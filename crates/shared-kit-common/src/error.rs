use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("file utils error: {0}")]
    FileUtils(#[from] crate::file_utils::error::FileError),

    #[error("matcher error: {0}")]
    Matcher(#[from] crate::matcher::MatcherError),
}
