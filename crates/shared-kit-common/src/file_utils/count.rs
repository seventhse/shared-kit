use crate::file_utils::error::FileError;
use std::{
    fs,
    path::{Path, PathBuf},
};

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
pub fn pre_count_files(path: &PathBuf) -> Result<usize, FileError> {
    fn count_recursive(path: &Path, count: &mut usize) -> Result<(), FileError> {
        for entry in fs::read_dir(path)
            .map_err(|e| FileError::ReadDirEntry { path: path.display().to_string(), source: e })?
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_count_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();
        fs::write(test_dir.join("file1.txt"), "content").unwrap();
        fs::write(test_dir.join("file2.txt"), "content").unwrap();
        let sub_dir = test_dir.join("sub_dir");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("file3.txt"), "content").unwrap();

        let count = pre_count_files(&test_dir.to_path_buf()).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        let count = pre_count_files(&test_dir.to_path_buf()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_non_existent_directory() {
        let test_dir = PathBuf::from("./non_existent_dir");
        let result = pre_count_files(&test_dir);
        assert!(result.is_err());
    }
}
