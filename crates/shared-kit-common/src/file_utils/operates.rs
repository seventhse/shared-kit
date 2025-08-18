use std::{fs, io::Write, path::Path};

use crate::file_utils::error::FileError;

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
pub fn write_file(target: &Path, content: &str) -> Result<(), FileError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| FileError::CreateDir { path: parent.to_path_buf(), source: e })?;
    }

    let mut file = fs::File::create(target)
        .map_err(|e| FileError::CreateFile { path: target.to_path_buf(), source: e })?;

    file.write_all(content.as_bytes())
        .map_err(|e| FileError::WriteFile { path: target.to_path_buf(), source: e })?;

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
pub fn read_file(origin: &Path) -> Result<String, FileError> {
    fs::read_to_string(origin)
        .map_err(|e| FileError::ReadFile { path: origin.to_path_buf(), source: e })
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        write_file(&test_file, "Hello, world!").unwrap();
        let content = read_file(&test_file).unwrap();

        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_write_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let nested_file = temp_dir.path().join("nested/dir/test.txt");

        write_file(&nested_file, "Nested content").unwrap();
        let content = read_file(&nested_file).unwrap();

        assert_eq!(content, "Nested content");
    }

    #[test]
    fn test_read_non_existent_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_file = temp_dir.path().join("non_existent.txt");

        let result = read_file(&non_existent_file);
        assert!(result.is_err());
    }
}
