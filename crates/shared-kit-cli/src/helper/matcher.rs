// matcher.rs

use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::{
    fmt::{self, Display},
    path::Path,
};

/// A unified pattern specification (either glob or regex).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSpec {
    Glob(String),
    Regex(String),
}

impl Display for PatternSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternSpec::Glob(s) => write!(f, "glob:{}", s),
            PatternSpec::Regex(s) => write!(f, "regex:{}", s),
        }
    }
}

impl PatternSpec {
    pub fn parse(raw: &str) -> Self {
        if let Some(r) = raw.strip_prefix("regex:") {
            PatternSpec::Regex(r.to_string())
        } else {
            PatternSpec::Glob(raw.to_string())
        }
    }

    pub fn from_option_vec(value: Option<Vec<String>>) -> Vec<PatternSpec> {
        value.unwrap_or_default().into_iter().map(|path| PatternSpec::parse(&path)).collect()
    }
}

/// A compiled matcher combining glob and regex rules.
#[derive(Debug)]
pub struct Matcher {
    globset: Option<GlobSet>,
    regexes: Vec<Regex>,
}

impl Matcher {
    pub fn new(patterns: &[PatternSpec]) -> anyhow::Result<Self> {
        let mut glob_builder = GlobSetBuilder::new();
        let mut regexes = Vec::new();
        let mut has_glob = false;

        for pattern in patterns {
            match pattern {
                PatternSpec::Glob(pat) => {
                    glob_builder.add(Glob::new(pat)?);
                    has_glob = true;
                }
                PatternSpec::Regex(pat) => {
                    regexes.push(Regex::new(pat)?);
                }
            }
        }

        let globset = if has_glob { Some(glob_builder.build()?) } else { None };
        Ok(Matcher { globset, regexes })
    }

    pub fn matches(&self, path: &Path) -> bool {
        if let Some(path_str) = path.to_str() {
            if let Some(gs) = &self.globset {
                if gs.is_match(path) {
                    return true;
                }
            }
            self.regexes.iter().any(|r| r.is_match(path_str))
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchResult {
    Matched,
    Excluded,
    NotIncluded,
}

/// Handles include and exclude filtering.
#[derive(Debug)]
pub struct FilterMatcher {
    includes: Option<Matcher>,
    excludes: Option<Matcher>,
}

impl FilterMatcher {
    pub fn new(includes: Vec<PatternSpec>, excludes: Vec<PatternSpec>) -> anyhow::Result<Self> {
        let includes = if includes.is_empty() { None } else { Some(Matcher::new(&includes)?) };

        let excludes = if excludes.is_empty() { None } else { Some(Matcher::new(&excludes)?) };

        Ok(Self { includes, excludes })
    }

    pub fn matches(&self, path: &Path) -> MatchResult {
        if let Some(ex) = &self.excludes {
            if ex.matches(path) {
                return MatchResult::Excluded;
            }
        }

        if let Some(inc) = &self.includes {
            if inc.matches(path) { MatchResult::Matched } else { MatchResult::NotIncluded }
        } else {
            MatchResult::Matched
        }
    }
}
