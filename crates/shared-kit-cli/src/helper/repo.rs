use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use reqwest::blocking::Client;
use shared_kit_common::log_warn;
use tempfile::TempDir;

use crate::components::progress::download_file_with_progress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRef {
    Branch(String),
    Tag(String),
    Commit(String),
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoPlatform {
    GitHub,
    GitLab,
    Gitea,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoInfo {
    pub platform: RepoPlatform,
    pub user: String,
    pub repo: String,
    pub r#ref: GitRef,
}

pub struct ExtractedRepo {
    pub root_dir: PathBuf,
    _tmp_dir: TempDir, // 保持生命周期，drop 时自动清理
}

impl RepoInfo {
    pub fn download_url(&self) -> String {
        let reference = match &self.r#ref {
            GitRef::Default => "main",
            GitRef::Branch(b) => b,
            GitRef::Tag(t) => t,
            GitRef::Commit(c) => c,
        };

        match self.platform {
            RepoPlatform::GitHub => {
                format!(
                    "https://github.com/{}/{}/archive/refs/heads/{}.zip",
                    self.user, self.repo, reference
                )
            }
            RepoPlatform::GitLab => {
                format!(
                    "https://gitlab.com/{}/{}/-/archive/{}/{}-{}.zip",
                    self.user, self.repo, reference, self.repo, reference
                )
            }
            RepoPlatform::Gitea => {
                format!("https://gitea.com/{}/{}/archive/{}.zip", self.user, self.repo, reference)
            }
            RepoPlatform::Other(ref domain) => {
                // 其他平台不支持直接下载zip，可以自定义处理或返回空串
                log_warn!("Warning: unsupported platform {}, fallback to empty url", domain);
                String::new()
            }
        }
    }
}

/// Parses a Git repository input string (supports full URLs and shorthand notation).
///
/// # Supported formats
///
/// ## ✅ Full URL formats (platform auto-detection)
/// - `https://github.com/user/repo` (default branch)
/// - `https://github.com/user/repo#branch` (specific branch)
/// - `https://github.com/user/repo@v1.0.0` (specific tag)
/// - `https://github.com/user/repo@<40-char-commit>` (specific commit)
/// - `https://gitlab.com/group/repo`
/// - `https://gitea.com/user/repo#branch`
///
/// ## ✅ Shorthand formats (default platform is GitHub)
/// - `user/repo` (default branch)
/// - `user/repo#branch`
/// - `user/repo@v1.2.3`
/// - `user/repo@0123456789abcdef0123456789abcdef01234567`
///
/// # Parameters
/// - `input`: The repository address string, either a full URL or GitHub-style shorthand.
///
/// # Returns
/// - `RepoInfo` containing platform, user, repo name, and ref (branch/tag/commit).
///
/// # Errors
/// - Returns an error if the input format is invalid.
///
/// # Example
/// ```
/// let info = parse_repo_input(&"user/repo#dev".to_string()).unwrap();
/// assert_eq!(info.repo, "repo");
/// assert_eq!(info.r#ref, GitRef::Branch("dev".to_string()));
/// ```
///
pub fn parse_repo_input(input: &String) -> anyhow::Result<RepoInfo> {
    // Try to parse URL form
    if input.starts_with("http://") || input.starts_with("https://") {
        parse_from_url(input)
    } else {
        parse_from_short(input)
    }
}

pub fn parse_from_url(input: &String) -> anyhow::Result<RepoInfo> {
    let raw: &str = input.as_str(); // or &input[..]
    let mut base = raw;
    let mut suffix: Option<(&str, &str)> = None;

    if let Some(pos) = raw.find('#') {
        base = &raw[..pos];
        suffix = Some(("#", &raw[pos + 1..]));
    } else if let Some(pos) = raw.find('@') {
        base = &raw[..pos];
        suffix = Some(("@", &raw[pos + 1..]));
    }

    let url = url::Url::parse(base)?;
    let host = url.host_str().ok_or_else(|| anyhow::anyhow!("Invalid URL: {}", raw))?;
    let segments: Vec<_> = url.path_segments().map_or(Vec::new(), |s| s.collect());

    if segments.len() < 2 {
        anyhow::bail!("URL path must contain user and repo: {}", url);
    }

    let user = segments[0].to_string();
    let repo = segments[1].trim_end_matches(".git").to_string();

    let platform = match host {
        "github.com" => RepoPlatform::GitHub,
        "gitlab.com" => RepoPlatform::GitLab,
        h if h.contains("gitea") => RepoPlatform::Gitea,
        h => RepoPlatform::Other(h.to_string()),
    };

    let r#ref = match suffix {
        Some(("#", val)) => GitRef::Branch(val.to_string()),
        Some(("@", val)) => {
            if is_probable_commit(val) {
                GitRef::Commit(val.to_string())
            } else {
                GitRef::Tag(val.to_string())
            }
        }
        _ => GitRef::Default,
    };

    Ok(RepoInfo { platform, user, repo, r#ref })
}

pub fn parse_from_short(input: &String) -> anyhow::Result<RepoInfo> {
    let re = shared_kit_common::regex::Regex::new(
        r"^(?P<user>[^/\s]+)/(?P<repo>[^\s@#]+)([@#](?P<ref>[^\s]+))?$",
    )?;
    let caps =
        re.captures(input).with_context(|| format!("Invalid short repo format: '{}'", input))?;

    let user = caps["user"].to_string();
    let repo = caps["repo"].to_string();

    let r#ref = match caps.name("ref") {
        Some(m) => {
            if input.contains('#') {
                GitRef::Branch(m.as_str().to_string())
            } else if is_probable_commit(m.as_str()) {
                GitRef::Commit(m.as_str().to_string())
            } else {
                GitRef::Tag(m.as_str().to_string())
            }
        }
        None => GitRef::Default,
    };

    Ok(RepoInfo { platform: RepoPlatform::GitHub, user, repo, r#ref })
}

fn is_probable_commit(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn find_root_dir(extract_dir: &Path) -> anyhow::Result<PathBuf> {
    let entries = fs::read_dir(extract_dir).context("Failed to read extract dir")?;
    for entry in entries {
        let entry = entry.context("Failed to read dir entry")?;
        if entry.path().is_dir() {
            return Ok(entry.path());
        }
    }
    anyhow::bail!("No extracted directory found in zip")
}

fn extract_zip(zip_path: &Path, extract_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(extract_dir).context("Failed to create extract dir")?;
    let zip_file = std::fs::File::open(zip_path).context("Failed to open zip file")?;
    let mut archive = zip::ZipArchive::new(zip_file).context("Failed to read zip archive")?;
    archive.extract(extract_dir).context("Failed to extract zip archive")
}

fn download_zip_to_path(url: &str, dest_path: &Path) -> anyhow::Result<()> {
    let client = Client::new();
    let resp =
        client.get(url).send().with_context(|| format!("Failed to send GET request to {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to download repo zip: HTTP {}", resp.status());
    }

    download_file_with_progress(resp, dest_path)
}

fn download_and_extract_zip(download_url: &str) -> anyhow::Result<ExtractedRepo> {
    let tmp_dir = tempfile::tempdir().context("Failed to create temp dir")?;
    let zip_path = tmp_dir.path().join("repo.zip");

    download_zip_to_path(download_url, &zip_path)?;
    let extract_dir = tmp_dir.path().join("extract");

    extract_zip(&zip_path, &extract_dir)?;
    let root_path = find_root_dir(&extract_dir)?;
    Ok(ExtractedRepo {
        root_dir: root_path,
        _tmp_dir: tmp_dir, // 保持生命周期直到结构体 drop
    })
}

pub fn resolve_repo_to_dir(url: &String) -> anyhow::Result<ExtractedRepo> {
    let repo_info = parse_repo_input(&url)?;
    let download_url = repo_info.download_url();

    if download_url.is_empty() {
        anyhow::bail!("Unsupported repo platform for direct zip download");
    }

    let res = download_and_extract_zip(&download_url)?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::write::{ExtendedFileOptions, FileOptions};

    use super::*;

    #[test]
    fn test_parse_from_short_branch() {
        let input = "user/repo#dev".to_string();
        let parsed = parse_repo_input(&input).unwrap();
        assert_eq!(parsed.user, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.r#ref, GitRef::Branch("dev".to_string()));
    }

    #[test]
    fn test_parse_from_short_tag() {
        let input = "user/repo@v1.0.0".to_string();
        let parsed = parse_repo_input(&input).unwrap();
        assert_eq!(parsed.r#ref, GitRef::Tag("v1.0.0".to_string()));
    }

    #[test]
    fn test_parse_from_short_commit() {
        let input = "user/repo@0123456789abcdef0123456789abcdef01234567".to_string();
        let parsed = parse_repo_input(&input).unwrap();
        assert_eq!(
            parsed.r#ref,
            GitRef::Commit("0123456789abcdef0123456789abcdef01234567".to_string())
        );
    }

    #[test]
    fn test_parse_from_url_github() {
        let input = "https://github.com/octocat/Hello-World.git".to_string();
        let parsed = parse_repo_input(&input).unwrap();
        assert_eq!(parsed.platform, RepoPlatform::GitHub);
        assert_eq!(parsed.user, "octocat");
        assert_eq!(parsed.repo, "Hello-World");
        assert_eq!(parsed.r#ref, GitRef::Default);
    }

    #[test]
    fn test_parse_from_url_with_ref() {
        let input = "https://github.com/octocat/Hello-World#dev".to_string();
        let parsed = parse_repo_input(&input).unwrap();
        assert_eq!(parsed.r#ref, GitRef::Branch("dev".to_string()));
    }

    #[test]
    fn test_download_url_generation_github() {
        let repo = RepoInfo {
            platform: RepoPlatform::GitHub,
            user: "octocat".to_string(),
            repo: "Hello-World".to_string(),
            r#ref: GitRef::Branch("main".to_string()),
        };
        let url = repo.download_url();
        assert_eq!(url, "https://github.com/octocat/Hello-World/archive/refs/heads/main.zip");
    }

    #[test]
    fn test_download_url_generation_gitlab() {
        let repo = RepoInfo {
            platform: RepoPlatform::GitLab,
            user: "gitlab-org".to_string(),
            repo: "gitlab".to_string(),
            r#ref: GitRef::Tag("v16.0".to_string()),
        };
        let url = repo.download_url();
        assert_eq!(url, "https://gitlab.com/gitlab-org/gitlab/-/archive/v16.0/gitlab-v16.0.zip");
    }

    #[test]
    fn test_extract_zip_and_find_root_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("sample.zip");

        // 创建一个临时 ZIP 文件
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options: FileOptions<'_, ExtendedFileOptions> = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

        // 添加一个目录
        zip.add_directory("test_dir/", options.clone()).unwrap();

        // 在目录中添加一个文件
        zip.start_file("test_dir/sample.txt", options).unwrap();
        zip.write_all(b"Hello, world!").unwrap();
        zip.finish().unwrap();

        let extract_dir = tmp.path().join("extract");
        extract_zip(&zip_path, &extract_dir).unwrap();

        // 验证解压后的根目录
        let root = find_root_dir(&extract_dir).unwrap();
        assert!(root.is_dir());

        // ✅ 修正路径拼接
        let sample_file = root.join("sample.txt");
        assert!(sample_file.exists());
        assert_eq!(std::fs::read_to_string(sample_file).unwrap(), "Hello, world!");
    }

    #[test]
    fn test_invalid_url_should_fail() {
        let input = "invalid_url".to_string();
        let result = parse_repo_input(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_download_url_should_fail() {
        let repo = RepoInfo {
            platform: RepoPlatform::Other("unknown.com".to_string()),
            user: "foo".to_string(),
            repo: "bar".to_string(),
            r#ref: GitRef::Default,
        };
        assert_eq!(repo.download_url(), "");
    }

    #[test]
    fn test_resolve_repo_to_dir_real_github() {
        let url = "https://github.com/octocat/Hello-World#master".to_string();
        let repo = resolve_repo_to_dir(&url).unwrap();
        assert!(repo.root_dir.exists());
        assert!(repo.root_dir.is_dir());
    }
}
