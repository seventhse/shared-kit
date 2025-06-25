use std::{fmt::Debug, sync::Arc};

use crate::error::Error;

pub trait Matcher: Send + Sync + Debug {
    fn is_match(&self, path: &str) -> bool;
}

#[derive(Debug)]
pub struct GlobMatcher {
    pub raw: String,
    pub inner: Arc<dyn Matcher>,
}

pub trait PatternCompiler: Send + Sync + Debug {
    fn compile(&self, pattern: &str) -> Result<GlobMatcher, Error>;
}
