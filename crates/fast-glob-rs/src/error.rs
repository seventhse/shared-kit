use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// TaskBuilder 结束时发现 include 列表为空
    #[error("task at root `{0}` has no include patterns")] // Display message
    EmptyInclude(PathBuf),

    #[error("brace expansion depth exceeded for pattern `{0}`")]
    BraceDepth(String),
}

pub type IResult<T> = Result<T, Error>;
