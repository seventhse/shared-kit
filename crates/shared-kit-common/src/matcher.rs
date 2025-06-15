use std::sync::Arc;

use globset::Glob;
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::lazy_cache;

lazy_cache!(REGEX_CACHE: String => Regex);
lazy_cache!(GLOB_CACHE: String => Glob);

#[derive(Error, Debug)]
pub enum MatcherError {
    #[error("Invalid regex: {value}")]
    NewRegex {
        value: String,
        #[source]
        source: regex::Error,
    },

    #[error("Invalid glob: {value}")]
    NewGlob {
        value: String,
        #[source]
        source: globset::Error,
    },
}

/// Retrieves a cached or newly created regex pattern.
///
/// # Arguments
///
/// * `val` - The regex pattern as a string.
///
/// # Returns
///
/// Returns an `Arc<Regex>` if successful, or a `MatcherError` if the regex is invalid.
fn get_regex(val: &str) -> Result<Arc<Regex>, MatcherError> {
    {
        let cache = REGEX_CACHE.read();
        if let Some(rex) = cache.get(val) {
            return Ok(Arc::clone(rex));
        }
    }

    let new_rex = Arc::new(
        Regex::new(val)
            .map_err(|e| MatcherError::NewRegex { value: val.to_string(), source: e })?,
    );

    let mut cache = REGEX_CACHE.write();
    cache.insert(val.to_string(), Arc::clone(&new_rex));

    Ok(new_rex)
}

/// Retrieves a cached or newly created glob pattern.
///
/// # Arguments
///
/// * `val` - The glob pattern as a string.
///
/// # Returns
///
/// Returns an `Arc<Glob>` if successful, or a `MatcherError` if the glob is invalid.
fn get_glob(val: &str) -> Result<Arc<Glob>, MatcherError> {
    {
        let cache = GLOB_CACHE.read();
        if let Some(glob) = cache.get(val) {
            return Ok(Arc::clone(glob));
        }
    }

    let new_glob =
        Glob::new(&val).map_err(|e| MatcherError::NewGlob { value: val.to_string(), source: e })?;
    let new_glob = Arc::new(new_glob);

    let mut cache = GLOB_CACHE.write();
    cache.insert(val.to_string(), Arc::clone(&new_glob));

    Ok(new_glob)
}

/// Represents a pattern kind, either a glob or a regex.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternKind {
    Glob(String),
    Regex(String),
}

/// Represents a pattern specification with optional custom data.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PatternSpec<T>
where
    T: Clone,
{
    kind: PatternKind,
    custom_data: Option<T>,
    style: MatcherStyle,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatcherStyle {
    Strict,
    Loose,
}

impl PatternKind {
    /// Parses a string into a `PatternKind`.
    ///
    /// # Arguments
    ///
    /// * `val` - The pattern string, prefixed with `regex:` for regex patterns.
    ///
    /// # Returns
    ///
    /// Returns a `PatternKind` enum.
    pub fn parse(val: &str) -> PatternKind {
        match val.strip_prefix("regex:") {
            Some(val) => PatternKind::Regex(String::from(val)),
            None => PatternKind::Glob(String::from(val)),
        }
    }
}

impl<T> PatternSpec<T>
where
    T: Clone,
{
    /// Creates a new `PatternSpec`.
    pub fn new(val: &str, custom_data: Option<T>, match_style: Option<MatcherStyle>) -> Self {
        Self {
            kind: PatternKind::parse(val),
            custom_data,
            style: match_style.unwrap_or(MatcherStyle::Loose),
        }
    }

    /// Checks if the pattern matches a given path.
    pub fn is_match(&self, path: &str) -> Result<bool, MatcherError> {
        match &self.kind {
            PatternKind::Glob(pattern) => {
                let glob = get_glob(pattern)?;
                let matched = glob.compile_matcher().is_match(path);
                match self.style {
                    MatcherStyle::Strict => Ok(matched),
                    MatcherStyle::Loose => Ok(matched || path.contains(pattern)),
                }
            }
            PatternKind::Regex(pattern) => {
                let regex = get_regex(pattern)?;
                let matched = regex.is_match(path);
                match self.style {
                    MatcherStyle::Strict => Ok(matched),
                    MatcherStyle::Loose => Ok(matched || path.contains(pattern)),
                }
            }
        }
    }
}

/// Builds a `Matcher` with include and exclude patterns.
#[derive(Debug, Clone)]
pub struct MatcherBuilder<T>
where
    T: Clone,
{
    includes: Vec<PatternSpec<T>>,
    excludes: Vec<PatternSpec<T>>,
    default_style: MatcherStyle,
}

impl<T> MatcherBuilder<T>
where
    T: Clone,
{
    /// Creates a new `MatcherBuilder`.
    ///
    /// # Returns
    ///
    /// Returns a new `MatcherBuilder` instance.
    pub fn new() -> Self {
        Self { includes: vec![], excludes: vec![], default_style: MatcherStyle::Loose }
    }

    /// Sets the default match style for the builder.
    ///
    /// # Arguments
    ///
    /// * `default_style` - The default matcher style (strict or loose).
    ///
    /// # Returns
    ///
    /// Returns the updated `MatcherBuilder` instance.
    pub fn with_match_style(mut self, default_style: MatcherStyle) -> Self {
        self.default_style = default_style;
        self
    }

    /// Adds an include pattern to the builder.
    ///
    /// # Arguments
    ///
    /// * `pattern_spec` - The pattern specification to include.
    ///
    /// # Returns
    ///
    /// Returns the updated `MatcherBuilder` instance.
    pub fn with_include(mut self, pattern_spec: PatternSpec<T>) -> Self {
        self.includes.push(pattern_spec);
        self
    }

    /// Adds an include pattern string to the builder.
    ///
    /// # Arguments
    ///
    /// * `pattern_str` - The pattern string to include.
    /// * `custom_data` - Optional custom data associated with the pattern.
    ///
    /// # Returns
    ///
    /// Returns the updated `MatcherBuilder` instance.
    pub fn with_include_str(mut self, pattern_str: &str, custom_data: Option<T>) -> Self {
        let pattern_spec = PatternSpec::new(pattern_str, custom_data, Some(self.default_style));
        self.includes.push(pattern_spec);
        self
    }

    pub fn with_include_strs<S, I>(mut self, patterns: I, custom_data: Option<T>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for pattern_str in patterns {
            let pattern_spec = PatternSpec::new(
                pattern_str.as_ref(),
                custom_data.clone(),
                Some(self.default_style),
            );
            self.includes.push(pattern_spec);
        }
        self
    }

    pub fn with_include_strs_opt<S, I>(
        mut self,
        patterns: Option<I>,
        custom_data: Option<T>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if let Some(patterns) = patterns {
            for pattern_str in patterns {
                let pattern_spec = PatternSpec::new(
                    pattern_str.as_ref(),
                    custom_data.clone(),
                    Some(self.default_style),
                );
                self.includes.push(pattern_spec);
            }
        }

        self
    }

    /// Adds an exclude pattern to the builder.
    ///
    /// # Arguments
    ///
    /// * `pattern_spec` - The pattern specification to exclude.
    ///
    /// # Returns
    ///
    /// Returns the updated `MatcherBuilder` instance.
    pub fn with_exclude(mut self, pattern_spec: PatternSpec<T>) -> Self {
        self.excludes.push(pattern_spec);
        self
    }

    /// Adds an exclude pattern string to the builder.
    ///
    /// # Arguments
    ///
    /// * `pattern_str` - The pattern string to exclude.
    /// * `custom_data` - Optional custom data associated with the pattern.
    ///
    /// # Returns
    ///
    /// Returns the updated `MatcherBuilder` instance.
    pub fn with_exclude_str(mut self, pattern_str: &str, custom_data: Option<T>) -> Self {
        let pattern_spec = PatternSpec::new(pattern_str, custom_data, Some(self.default_style));
        self.excludes.push(pattern_spec);
        self
    }

    pub fn with_exclude_strs<S, I>(mut self, patterns: I, custom_data: Option<T>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for pattern_str in patterns {
            let pattern_spec = PatternSpec::new(
                pattern_str.as_ref(),
                custom_data.clone(),
                Some(self.default_style),
            );
            self.excludes.push(pattern_spec);
        }
        self
    }

    pub fn with_exclude_strs_opt<S, I>(
        mut self,
        patterns: Option<I>,
        custom_data: Option<T>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if let Some(patterns) = patterns {
            for pattern_str in patterns {
                let pattern_spec = PatternSpec::new(
                    pattern_str.as_ref(),
                    custom_data.clone(),
                    Some(self.default_style),
                );
                self.excludes.push(pattern_spec);
            }
        }

        self
    }

    /// Builds the `Matcher` instance.
    ///
    /// # Returns
    ///
    /// Returns a new `Matcher` instance.
    pub fn build(self) -> Matcher<T> {
        Matcher { includes: self.includes, excludes: self.excludes }
    }
}

/// Represents a matcher with include and exclude patterns.
#[derive(Debug, Clone)]
pub struct Matcher<T>
where
    T: Clone,
{
    includes: Vec<PatternSpec<T>>,
    excludes: Vec<PatternSpec<T>>,
}

impl<T> Matcher<T>
where
    T: Clone,
{
    /// Checks if a path matches any include or exclude patterns.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to match against.
    ///
    /// # Returns
    ///
    /// Returns a `MatcherResult` indicating the match status, or a `MatcherError` if an error occurs.
    pub fn is_match(&self, path: &str) -> Result<MatcherResult<T>, MatcherError> {
        for pattern in &self.excludes {
            if pattern.is_match(path)? {
                return Ok(MatcherResult::InExclude(pattern.custom_data.clone()));
            }
        }

        for pattern in &self.includes {
            if pattern.is_match(path)? {
                return Ok(MatcherResult::Matched(pattern.custom_data.clone()));
            }
        }

        Ok(MatcherResult::NoMatched)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MatcherResult<T>
where
    T: Clone,
{
    Matched(Option<T>),
    InExclude(Option<T>),
    NoMatched,
}

impl<T> MatcherResult<T>
where
    T: Clone,
{
    /// Checks if the result is a match.
    pub fn is_matched(&self) -> bool {
        matches!(self, MatcherResult::Matched(_))
    }

    /// Checks if the result is in the exclude list.
    pub fn is_in_exclude(&self) -> bool {
        matches!(self, MatcherResult::InExclude(_))
    }

    /// Checks if the result is not matched.
    pub fn is_no_matched(&self) -> bool {
        matches!(self, MatcherResult::NoMatched)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_kind_parse() {
        assert_eq!(PatternKind::parse("regex:.*"), PatternKind::Regex(".*".to_string()));
        assert_eq!(PatternKind::parse("*.txt"), PatternKind::Glob("*.txt".to_string()));
    }

    #[test]
    fn test_pattern_spec_is_match_strict() {
        let spec = PatternSpec::new("file.txt", None::<()>, Some(MatcherStyle::Strict));
        assert!(spec.is_match("file.txt").unwrap());
        assert!(!spec.is_match("other_file.txt").unwrap());
    }

    #[test]
    fn test_pattern_spec_is_match_loose() {
        let spec = PatternSpec::new("file", None::<()>, Some(MatcherStyle::Loose));
        assert!(spec.is_match("file.txt").unwrap());
        assert!(spec.is_match("other_file.txt").unwrap());
        assert!(!spec.is_match("random.rs").unwrap());
    }

    #[test]
    fn test_matcher_with_include_and_exclude() {
        let matcher = MatcherBuilder::new()
            .with_include_str("*.txt", None::<()>)
            .with_exclude_str("secret.txt", None::<()>)
            .build();

        assert!(matcher.is_match("file.txt").unwrap().is_matched());
        assert!(matcher.is_match("secret.txt").unwrap().is_in_exclude());
        assert!(matcher.is_match("random.rs").unwrap().is_no_matched());
    }

    #[test]
    fn test_matcher_with_regex_and_glob() {
        let matcher = MatcherBuilder::new()
            .with_include_str("regex:^file\\d+\\.txt$", None::<()>)
            .with_include_str("*.log", None::<()>)
            .build();

        assert!(matcher.is_match("file123.txt").unwrap().is_matched());
        assert!(matcher.is_match("error.log").unwrap().is_matched());
        assert!(matcher.is_match("random.rs").unwrap().is_no_matched());
    }

    #[test]
    fn test_matcher_style_interaction() {
        let matcher = MatcherBuilder::new()
            .with_include(PatternSpec::new("file", None::<()>, Some(MatcherStyle::Loose)))
            .with_exclude(PatternSpec::new("file.txt", None::<()>, Some(MatcherStyle::Strict)))
            .build();

        assert!(matcher.is_match("file.txt").unwrap().is_in_exclude());
        assert!(matcher.is_match("file123.txt").unwrap().is_matched());
        assert!(matcher.is_match("random.rs").unwrap().is_no_matched());
    }

    #[test]
    fn test_matcher_empty_patterns() {
        let matcher = MatcherBuilder::<()>::new().build();

        assert!(matcher.is_match("file.txt").unwrap().is_no_matched());
        assert!(matcher.is_match("random.rs").unwrap().is_no_matched());
    }
}
