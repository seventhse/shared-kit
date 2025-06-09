// matcher_group.rs (decoupled from TemplateItem)

use crate::helper::matcher::{FilterMatcher, MatchResult, PatternSpec};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ResolvedVar {
    pub placeholder: String,
    pub replacement: String,
    pub includes: Vec<PatternSpec>,
    pub excludes: Vec<PatternSpec>,
}

#[derive(Debug)]
pub struct TemplateVarMatcher {
    pub placeholder: String,
    pub replacement: String,
    pub matcher: FilterMatcher,
}

#[derive(Debug)]
pub struct MatcherGroup {
    pub global: FilterMatcher,
    pub variables: Vec<TemplateVarMatcher>,
}

impl MatcherGroup {
    /// Construct a matcher group from a set of global patterns and resolved variables.
    pub fn from_resolved(
        global_includes: Vec<PatternSpec>,
        global_excludes: Vec<PatternSpec>,
        resolved_vars: Vec<ResolvedVar>,
    ) -> anyhow::Result<Self> {
        let global = FilterMatcher::new(global_includes, global_excludes)?;

        let mut variables = Vec::new();
        for var in resolved_vars {
            let matcher = FilterMatcher::new(var.includes, var.excludes)?;
            variables.push(TemplateVarMatcher {
                placeholder: var.placeholder,
                replacement: var.replacement,
                matcher,
            });
        }

        Ok(MatcherGroup { global, variables })
    }

    /// Return matched placeholders with status per path.
    pub fn vars_for_path_detailed<'a>(&'a self, path: &Path) -> Vec<(&'a str, MatchResult)> {
        self.variables.iter().map(|v| (v.placeholder.as_str(), v.matcher.matches(path))).collect()
    }

    /// Return matched variable placeholders only.
    pub fn matched_vars_for_path<'a>(&'a self, path: &Path) -> Vec<&'a str> {
        self.vars_for_path_detailed(path)
            .into_iter()
            .filter_map(|(ph, res)| if res == MatchResult::Matched { Some(ph) } else { None })
            .collect()
    }

    /// Replace all applicable variables in file content based on file path.
    pub fn apply_var_replacements(&self, content: &str, path: &Path) -> String {
        let mut result = content.to_string();
        for var in &self.variables {
            if var.matcher.matches(path) == MatchResult::Matched {
                result = result.replace(&var.placeholder, &var.replacement);
            }
        }
        result
    }

    /// Filter only files that pass the global matcher.
    pub fn filter_matched_paths<'a>(&self, files: impl Iterator<Item = &'a Path>) -> Vec<&'a Path> {
        files.filter(|p| self.global.matches(p) == MatchResult::Matched).collect()
    }
}
