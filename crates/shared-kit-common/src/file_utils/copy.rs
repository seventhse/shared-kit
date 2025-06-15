use std::{
    fs,
    path::{Path, PathBuf},
};

use path_clean::PathClean;

use crate::{
    file_utils::{
        error::FileError,
        operates::{read_file, write_file},
    },
    middleware_pipeline::PipelineContext,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTransformKind {
    Skip,
    Transform(String),
    Rename(String),
    Overwrite { new_content: String, new_name: String },
    NoChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTransformContext {
    pub origin: PathBuf,
    pub target: PathBuf,
    pub content: String,
}
impl PipelineContext for FileTransformContext {}

/// Recursively copies a directory's contents to a target path, optionally transforming file contents.
///
/// # Arguments
///
/// * `origin` - Source directory path.
/// * `target` - Destination directory path.
/// * `callback` - Optional callback that determines how files are transformed or skipped.
///
/// # Behavior
///
/// - Preserves directory structure.
/// - Skips files if `FileTransformKind::Skip` is returned from the callback.
/// - Renames files if `FileTransformKind::Rename(String)` is returned.
/// - Overwrites files with new content and name if `FileTransformKind::Overwrite { new_content, new_name }` is returned.
/// - Transforms file content if `FileTransformKind::Transform(String)` is returned.
/// - Leaves files unchanged if `FileTransformKind::NoChange` is returned.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
///
/// let transform = |ctx: FileTransformContext| {
///     if ctx.content.contains("old") {
///         FileTransformKind::Transform(ctx.content.replace("old", "new"))
///     } else {
///         FileTransformKind::NoChange
///     }
/// };
///
/// copy_directory_with_transform(
///     &PathBuf::from("./src"),
///     &PathBuf::from("./dst"),
///     Some(&transform),
/// )?;
/// ```
pub fn copy_directory_with_transform<F>(
    origin: &PathBuf,
    target: &PathBuf,
    callback: Option<&F>,
) -> Result<(), FileError>
where
    F: Fn(FileTransformContext) -> FileTransformKind + Send + Sync + 'static,
{
    if !origin.is_dir() || !origin.exists() {
        return Err(FileError::NotDirectory(origin.display().to_string()));
    }

    let entries: Vec<_> = fs::read_dir(origin)
        .map_err(|e| FileError::ReadDir { path: origin.to_path_buf(), source: e })?
        .collect::<Result<_, _>>()
        .map_err(|e| FileError::ReadDir { path: origin.clone(), source: e })?;

    if entries.is_empty() {
        fs::create_dir(target)
            .map_err(|e| FileError::CreateDir { path: target.clone(), source: e })?;
        return Ok(());
    }

    for entry in entries {
        let entry = entry;
        let path = entry.path();
        let relative_path = path.strip_prefix(origin).unwrap();
        let target_path = target.join(relative_path);
        if path.is_dir() {
            copy_directory_with_transform(&path, &target_path, callback)?;
        } else if path.is_file() {
            copy_with_transform(&path, &target_path, callback)?;
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
/// * `callback` - Optional callback that determines how the file is transformed or skipped.
///
/// # Behavior
///
/// - Skips the file if `FileTransformKind::Skip` is returned.
/// - Renames the file if `FileTransformKind::Rename(String)` is returned.
/// - Overwrites the file with new content and name if `FileTransformKind::Overwrite { new_content, new_name }` is returned.
/// - Transforms the file content if `FileTransformKind::Transform(String)` is returned.
/// - Leaves the file unchanged if `FileTransformKind::NoChange` is returned.
///
/// # Examples
///
/// ```rust
/// let transform = |ctx: FileTransformContext| {
///     if ctx.content.contains("ignore") {
///         FileTransformKind::Skip
///     } else {
///         FileTransformKind::Transform(ctx.content.to_string())
///     }
/// };
///
/// copy_with_transform(Path::new("a.txt"), Path::new("b.txt"), Some(&transform))?;
/// ```
pub fn copy_with_transform<F>(
    origin: &Path,
    target: &Path,
    callback: Option<&F>,
) -> Result<(), FileError>
where
    F: Fn(FileTransformContext) -> FileTransformKind + Send + Sync + 'static,
{
    let content = read_file(&origin)?;

    let transform_result = match callback {
        Some(cb) => {
            let context = FileTransformContext {
                origin: origin.to_path_buf().clone(),
                target: target.to_path_buf().clean(),
                content: content.clone(),
            };
            cb(context)
        }
        None => FileTransformKind::NoChange,
    };

    match transform_result {
        FileTransformKind::Skip => {
            return Ok(());
        }
        FileTransformKind::Rename(new_name) => {
            let new_target = target.with_file_name(new_name);
            write_file(&new_target, &content)?
        }
        FileTransformKind::Transform(new_content) => write_file(target, &new_content)?,
        FileTransformKind::Overwrite { new_content, new_name } => {
            eprintln!("new_name: {},new_content: {}", new_name, new_content);
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
    use tempfile::TempDir;

    #[test]
    fn test_copy_directory_with_transform() {
        let temp_dir = TempDir::new().unwrap();
        let origin_dir = temp_dir.path().join("origin");
        let target_dir = temp_dir.path().join("target");

        fs::create_dir_all(&origin_dir).unwrap();
        fs::write(origin_dir.join("file1.txt"), "content1").unwrap();
        fs::write(origin_dir.join("file2.txt"), "content2").unwrap();

        copy_directory_with_transform::<fn(FileTransformContext) -> FileTransformKind>(
            &origin_dir,
            &target_dir,
            None,
        )
        .unwrap();

        assert!(target_dir.join("file1.txt").exists());
        assert!(target_dir.join("file2.txt").exists());
    }

    #[test]
    fn test_copy_with_transform_skip() {
        let temp_dir = TempDir::new().unwrap();
        let origin_file = temp_dir.path().join("origin.txt");
        let target_file = temp_dir.path().join("target.txt");

        fs::write(&origin_file, "content").unwrap();

        let callback = |ctx: FileTransformContext| {
            if ctx.content.contains("content") {
                FileTransformKind::Skip
            } else {
                FileTransformKind::NoChange
            }
        };

        copy_with_transform(&origin_file, &target_file, Some(&callback)).unwrap();

        assert!(!target_file.exists());
    }

    #[test]
    fn test_copy_with_transform_rename() {
        let temp_dir = TempDir::new().unwrap();
        let origin_file = temp_dir.path().join("origin.txt");
        let target_file = temp_dir.path().join("target.txt");

        fs::write(&origin_file, "content").unwrap();

        let callback = |ctx: FileTransformContext| {
            FileTransformKind::Rename(format!(
                "renamed_{}.txt",
                ctx.origin.file_stem().unwrap().to_string_lossy()
            ))
        };

        copy_with_transform(&origin_file, &target_file, Some(&callback)).unwrap();

        assert!(temp_dir.path().join("renamed_origin.txt").exists());
    }

    #[test]
    fn test_copy_with_transform_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let origin_file = temp_dir.path().join("origin.txt");
        let target_file = temp_dir.path().join("target.txt");

        fs::write(&origin_file, "content").unwrap();

        let callback = |ctx: FileTransformContext| {
            let content = format!("new content from {}", ctx.origin.display());
            FileTransformKind::Overwrite {
                new_content: content.clone(),
                new_name: "new_name.txt".to_string(),
            }
        };

        copy_with_transform(&origin_file, &target_file, Some(&callback)).unwrap();

        assert!(temp_dir.path().join("new_name.txt").exists());
        let new_content = read_file(&temp_dir.path().join("new_name.txt")).unwrap();
        assert_eq!(new_content, format!("new content from {}", origin_file.display()));
    }

    #[test]
    fn test_copy_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let origin_dir = temp_dir.path().join("empty_origin");
        let target_dir = temp_dir.path().join("empty_target");

        fs::create_dir_all(&origin_dir).unwrap();

        copy_directory_with_transform::<fn(FileTransformContext) -> FileTransformKind>(
            &origin_dir,
            &target_dir,
            None,
        )
        .unwrap();

        assert!(target_dir.exists());
        assert!(fs::read_dir(&target_dir).unwrap().next().is_none());
    }

    #[test]
    fn test_copy_directory_non_existent() {
        let temp_dir = TempDir::new().unwrap();
        let origin_dir = temp_dir.path().join("non_existent_origin");
        let target_dir = temp_dir.path().join("target");

        let result = copy_directory_with_transform::<fn(FileTransformContext) -> FileTransformKind>(
            &origin_dir,
            &target_dir,
            None,
        );
        assert!(result.is_err());
    }
}
