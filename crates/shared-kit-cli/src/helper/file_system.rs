use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum FileTransformKind {
    Skip,
    Replace(String),
    Rename(String),
    Overwrite { new_content: String, new_name: String },
    NoChange,
}

pub type TransformCallback = dyn Fn(&str, &Path) -> FileTransformKind;

/// Recursively counts the number of files (not directories) under a given path.
///
/// # Arguments
///
/// * `path` - The root directory path to start counting from.
///
/// # Returns
///
/// Returns the total number of files found, or an error if any directory cannot be read.
///
/// # Examples
///
/// ```rust
/// let count = pre_count_files(&PathBuf::from("./some_folder")).unwrap();
/// println!("Total files: {}", count);
/// ```
pub fn pre_count_files(path: &PathBuf) -> anyhow::Result<usize> {
    fn count_recursive(path: &Path, count: &mut usize) -> anyhow::Result<()> {
        for entry in fs::read_dir(path)
            .map_err(|e| anyhow::anyhow!("Failed to read dir '{}': {}", path.display(), e))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                *count += 1;
            } else if path.is_dir() {
                count_recursive(&path, count)?;
            }
        }
        Ok(())
    }

    let mut count = 0;
    count_recursive(path, &mut count)?;
    Ok(count)
}

/// Writes the given content to the target file, creating parent directories if needed.
///
/// # Arguments
///
/// * `target` - The destination file path.
/// * `content` - The string content to write.
///
/// # Returns
///
/// Returns `Ok(())` if the write succeeds, or an error otherwise.
///
/// # Examples
///
/// ```rust
/// write_file(Path::new("./output.txt"), "Hello, world!")?;
/// ```
pub fn write_file(target: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    let mut file = fs::File::create(target)
        .with_context(|| format!("Failed to create target file: {}", target.display()))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to target file: {}", target.display()))?;

    Ok(())
}

/// Reads the entire content of a file into a `String`.
///
/// # Arguments
///
/// * `origin` - Path to the source file to read.
///
/// # Returns
///
/// A `String` containing the file content, or an error if reading fails.
///
/// # Examples
///
/// ```rust
/// let content = read_file(Path::new("./input.txt"))?;
/// println!("{}", content);
/// ```
pub fn read_file(origin: &Path) -> anyhow::Result<String> {
    let mut content = String::new();

    fs::File::open(origin)
        .with_context(|| format!("Failed to open source file: {}", origin.display()))?
        .read_to_string(&mut content)
        .with_context(|| format!("Failed to read from source file: {}", origin.display()))?;

    Ok(content)
}

/// Recursively copies a directory's contents to a target path, optionally transforming file contents.
///
/// # Arguments
///
/// * `origin` - Source directory path.
/// * `target` - Destination directory path.
/// * `callback` - Optional callback that returns either a modified string or skip signal.
///
/// # Behavior
///
/// - Preserves directory structure.
/// - Skips files if `FileTransformKind::Skip` is returned from the callback.
/// - Replaces file content if `FileTransformKind::Replace(String)` is returned.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
///
/// let transform = |content: &str, path: &Path| {
///     FileTransformKind::Replace(content.replace("old", "new"))
/// };
///
/// copy_directory_with_replace(
///     &PathBuf::from("./src"),
///     &PathBuf::from("./dst"),
///     Some(&transform),
/// )?;
/// ```
pub fn copy_directory_with_replace(
    origin: &PathBuf,
    target: &PathBuf,
    callback: Option<&TransformCallback>,
) -> anyhow::Result<()> {
    if !origin.is_dir() {
        let err_msg = format!("Source path is not a directory: {}", origin.display());
        error_msg!("{}", &err_msg);
        anyhow::bail!(err_msg);
    }

    for entry in fs::read_dir(origin)
        .with_context(|| format!("Failed to read directory: {}", origin.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(origin).unwrap();
        let target_path = target.join(relative_path);

        if path.is_dir() {
            copy_directory_with_replace(&path, &target_path, callback)?;
        } else if path.is_file() {
            copy_with_replace(&path, &target_path, callback)
                .with_context(|| format!("Failed to copy file: {}", path.display()))?;
        }
    }

    Ok(())
}

/// Copies a single file, optionally transforming or skipping its content.
///
/// # Arguments
///
/// * `origin` - Path to the source file.
/// * `target` - Destination file path.
/// * `callback` - Optional callback that transforms or skips the content.
///
/// # Returns
///
/// Returns `Ok(())` if the operation succeeds, or an error otherwise.
///
/// # Examples
///
/// ```rust
/// let transform = |content: &str, _path: &Path| {
///     if content.contains("ignore") {
///         FileTransformKind::Skip
///     } else {
///         FileTransformKind::Replace(content.to_string())
///     }
/// };
///
/// copy_with_replace(Path::new("a.txt"), Path::new("b.txt"), Some(&transform))?;
/// ```
pub fn copy_with_replace(
    origin: &Path,
    target: &Path,
    callback: Option<&TransformCallback>,
) -> anyhow::Result<()> {
    let content = read_file(origin)?;

    let transform_result = match callback {
        Some(cb) => cb(&content, origin),
        None => FileTransformKind::NoChange,
    };

    match transform_result {
        FileTransformKind::Skip => {
            info_msg!("Skipped file: {}", origin.display());
            return Ok(());
        }
        FileTransformKind::Rename(new_name) => {
            let new_target = target.with_file_name(new_name);
            write_file(&new_target, &content)?
        }
        FileTransformKind::Replace(new_content) => write_file(target, &new_content)?,
        FileTransformKind::Overwrite { new_content, new_name } => {
            let new_target = target.with_file_name(new_name);
            write_file(&new_target, &new_content)?;
        }
        FileTransformKind::NoChange => {
            write_file(target, &content)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn create_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    /// Test that `pre_count_files` returns 0 for an empty directory.
    #[test]
    fn test_pre_count_files_empty() {
        let dir = tempdir().unwrap();
        assert_eq!(pre_count_files(&dir.path().to_path_buf()).unwrap(), 0);
    }

    /// Test that `pre_count_files` correctly counts nested files.
    #[test]
    fn test_pre_count_files_nested() {
        let dir = tempdir().unwrap();
        create_file(&dir.path().join("a.txt"), "a");
        create_file(&dir.path().join("sub/b.txt"), "b");
        assert_eq!(pre_count_files(&dir.path().to_path_buf()).unwrap(), 2);
    }

    /// Test that `pre_count_files` fails on a nonexistent path.
    #[test]
    fn test_pre_count_files_invalid_path() {
        let path = PathBuf::from("nonexistent_dir_should_fail");
        assert!(pre_count_files(&path).is_err());
    }

    /// Test writing and reading back a file.
    #[test]
    fn test_write_and_read_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let content = "Hello, world!";
        write_file(&file_path, content).unwrap();
        let read_back = read_file(&file_path).unwrap();
        assert_eq!(read_back, content);
    }

    /// Test that `copy_with_replace` performs content replacement.
    #[test]
    fn test_copy_with_replace_replace() {
        let dir = tempdir().unwrap();
        let origin = dir.path().join("origin.txt");
        let target = dir.path().join("target.txt");

        create_file(&origin, "original");

        let transform = |_: &str, _: &Path| FileTransformKind::Replace("replaced".to_string());
        copy_with_replace(&origin, &target, Some(&transform)).unwrap();

        let result = fs::read_to_string(&target).unwrap();
        assert_eq!(result, "replaced");
    }

    /// Test that `copy_with_replace` skips the file when `Skip` is returned.
    #[test]
    fn test_copy_with_replace_skip() {
        let dir = tempdir().unwrap();
        let origin = dir.path().join("origin.txt");
        let target = dir.path().join("target.txt");

        create_file(&origin, "original");

        let transform = |_: &str, _: &Path| FileTransformKind::Skip;
        copy_with_replace(&origin, &target, Some(&transform)).unwrap();

        assert!(!target.exists());
    }

    /// Test that `copy_directory_with_replace` recursively copies and transforms files.
    #[test]
    fn test_copy_directory_with_replace_basic() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        let dst = dir.path().join("dst");

        create_file(&src.join("a.txt"), "foo");
        create_file(&src.join("nested/b.txt"), "bar");

        let transform = |_: &str, _: &Path| FileTransformKind::Replace("baz".to_string());

        copy_directory_with_replace(&src, &dst, Some(&transform)).unwrap();

        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "baz");
        assert_eq!(fs::read_to_string(dst.join("nested/b.txt")).unwrap(), "baz");
    }

    /// Test `copy_directory_with_replace` when source is not a directory.
    #[test]
    fn test_copy_directory_with_replace_invalid_source() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("not_a_dir.txt");
        create_file(&file, "invalid");

        let result = copy_directory_with_replace(&file, &dir.path().join("out"), None);
        assert!(result.is_err());
    }

    /// Test `read_file` with invalid file path.
    #[test]
    fn test_read_file_invalid() {
        let path = PathBuf::from("nonexistent_file.txt");
        assert!(read_file(&path).is_err());
    }

    /// Test that `copy_with_replace` handles Overwrite variant.
    #[test]
    fn test_copy_with_replace_overwrite() {
        let dir = tempdir().unwrap();
        let origin = dir.path().join("origin.txt");
        let target = dir.path().join("target.txt");
        let new_name = "renamed.txt";
        create_file(&origin, "original");
        let transform = |_: &str, _: &Path| FileTransformKind::Overwrite {
            new_content: "overwritten".to_string(),
            new_name: new_name.to_string(),
        };
        copy_with_replace(&origin, &target, Some(&transform)).unwrap();
        let renamed_path = target.with_file_name(new_name);
        assert_eq!(fs::read_to_string(renamed_path).unwrap(), "overwritten");
    }

    /// Test that `copy_with_replace` handles Rename variant.
    #[test]
    fn test_copy_with_replace_rename() {
        let dir = tempdir().unwrap();
        let origin = dir.path().join("origin.txt");
        let target = dir.path().join("target.txt");
        let new_name = "renamed.txt";
        create_file(&origin, "original");
        let transform = |_: &str, _: &Path| FileTransformKind::Rename(new_name.to_string());
        copy_with_replace(&origin, &target, Some(&transform)).unwrap();
        let renamed_path = target.with_file_name(new_name);
        assert_eq!(fs::read_to_string(renamed_path).unwrap(), "original");
    }

    /// Test that `copy_with_replace` works with callback None (NoChange).
    #[test]
    fn test_copy_with_replace_no_callback() {
        let dir = tempdir().unwrap();
        let origin = dir.path().join("origin.txt");
        let target = dir.path().join("target.txt");
        create_file(&origin, "original");
        copy_with_replace(&origin, &target, None).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "original");
    }

    /// Test that `copy_directory_with_replace` creates target directories automatically.
    #[test]
    fn test_copy_directory_with_replace_creates_target_dirs() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        let dst = dir.path().join("dst/nested/dir");
        create_file(&src.join("a.txt"), "foo");
        copy_directory_with_replace(&src, &dst, None).unwrap();
        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "foo");
    }
}
