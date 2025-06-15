use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to read directory: {path}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read directory entry in '{path}'")]
    ReadDirEntry {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create parent directory '{path}'")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create target file '{path}'")]
    CreateFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write to target file '{path}'")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read from source file '{path}'")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Source path is not a directory: {0}")]
    NotDirectory(String),
}
