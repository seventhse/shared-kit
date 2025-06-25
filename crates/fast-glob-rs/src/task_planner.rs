use std::{collections::HashMap, path::PathBuf};

use crate::{
    error::{Error, IResult},
    matcher::matcher::{GlobMatcher, PatternCompiler},
};

#[derive(Debug)]
pub struct Task {
    pub root: PathBuf,
    pub include: Vec<GlobMatcher>,
    pub exclude: Vec<GlobMatcher>,
    pub max_depth: Option<usize>,
}

#[derive(Debug)]
struct TaskBuilder {
    root: PathBuf,
    include_raw: Vec<String>,
    exclude_raw: Vec<String>,
    max_depth: Option<usize>,
}

impl TaskBuilder {
    fn new(root: PathBuf) -> Self {
        Self { root, include_raw: vec![], exclude_raw: vec![], max_depth: None }
    }

    fn finalize<C: PatternCompiler>(self, compiler: &C) -> IResult<Task> {
        if self.include_raw.is_empty() {
            return Err(Error::EmptyInclude(self.root));
        }

        let include =
            self.include_raw.iter().map(|p| compiler.compile(p)).collect::<IResult<Vec<_>>>()?;
        let exclude =
            self.exclude_raw.iter().map(|p| compiler.compile(p)).collect::<IResult<Vec<_>>>()?;

        Ok(Task { root: self.root, include, exclude, max_depth: self.max_depth })
    }
}

#[derive(Debug)]
struct NormalizedPattern {
    raw: String,
    is_negated: bool,
    prefix: PathBuf,
    max_depth: Option<usize>,
}

#[derive(Debug)]
pub struct PlannedTask {
    pub tasks: Vec<Task>,
    pub global_excludes: Vec<GlobMatcher>,
}

#[derive(Debug)]
pub struct TaskPlanner<C: PatternCompiler> {
    compiler: C,
}

impl<C: PatternCompiler> TaskPlanner<C> {
    #[inline]
    pub fn new(compiler: C) -> Self {
        Self { compiler }
    }

    pub fn plan(&self, includes: &[String], excludes: &[String]) -> IResult<PlannedTask> {
        let mut buckets: HashMap<PathBuf, TaskBuilder> = HashMap::new();
        let mut global_excludes_raw = Vec::<String>::new();

        let iter = includes.iter().map(|s| (s, false)).chain(excludes.iter().map(|s| (s, true)));

        for (raw, is_negated) in iter {
            let pat = Self::normalize_pattern(raw, is_negated);

            if pat.is_negated && Self::is_plain_directory(&pat.raw) {
                global_excludes_raw.push(pat.raw);
                continue;
            }

            let bucket = buckets
                .entry(pat.prefix.clone())
                .or_insert_with(|| TaskBuilder::new(pat.prefix.clone()));

            if pat.is_negated {
                bucket.exclude_raw.push(pat.raw);
            } else {
                bucket.include_raw.push(pat.raw);
            }

            bucket.max_depth = Self::merge_depth(bucket.max_depth, pat.max_depth);
        }

        let mut tasks = Vec::with_capacity(buckets.len());

        for (_, builder) in buckets {
            tasks.push(builder.finalize(&self.compiler)?);
        }

        let mut global_excludes = Vec::with_capacity(global_excludes_raw.len());

        for dir in global_excludes_raw {
            global_excludes.push(self.compiler.compile(&format!("{dir}/**"))?);
        }

        Ok(PlannedTask { tasks, global_excludes })
    }

    fn normalize_pattern(raw: &str, negated: bool) -> NormalizedPattern {
        let trimmed = raw.trim_start_matches("./").replace('\\', "/");
        let prefix = Self::static_prefix(&trimmed);
        let max_depth = Self::infer_depth(&trimmed);
        NormalizedPattern { raw: trimmed, is_negated: negated, prefix, max_depth }
    }

    fn static_prefix(pat: &str) -> PathBuf {
        let mut buf = String::new();
        let mut chars = pat.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                '*' | '?' | '[' | '{' => break,
                _ => {
                    buf.push(c);
                    chars.next();
                }
            }
        }
        let prefix_str = buf.trim_end_matches('/');
        if prefix_str.is_empty() {
            PathBuf::from(".") // 使用当前目录作为空前缀的占位符
        } else {
            PathBuf::from(prefix_str)
        }
    }

    fn is_plain_directory(pat: &str) -> bool {
        !pat.contains('*') && !pat.contains('?') && !pat.contains('[') && !pat.contains('{')
    }

    fn infer_depth(pat: &str) -> Option<usize> {
        if pat.contains("**") {
            return None;
        }
        Some(pat.split('/').count())
    }

    fn merge_depth(a: Option<usize>, b: Option<usize>) -> Option<usize> {
        match (a, b) {
            (Some(x), Some(y)) => Some(x.max(y)),
            (None, _) | (_, None) => None,
        }
    }
}

/* ---------------- Smoke test ---------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::matcher::Matcher;
    use std::sync::Arc;

    /// Replace this with your real glob parser + matcher.
    #[derive(Debug)]
    pub struct DummyCompiler;

    impl PatternCompiler for DummyCompiler {
        fn compile(&self, pattern: &str) -> IResult<GlobMatcher> {
            Ok(GlobMatcher {
                raw: pattern.to_owned(),
                inner: Arc::new(DummyMatcher { pat: pattern.to_owned() }),
            })
        }
    }

    #[derive(Debug)]
    struct DummyMatcher {
        pat: String,
    }

    impl Matcher for DummyMatcher {
        /// Always returns true – placeholder.
        fn is_match(&self, _path: &str) -> bool {
            println!("[DEBUG] matching {:?} (always true)", self.pat);
            true
        }
    }

    #[test]
    fn plan_basic() {
        let includes = vec!["**/**/*.ts".to_string(), "./src/**/*.js".to_string()];
        let excludes = vec!["node_modules".to_string(), "src/demo.js".to_string()];

        let planner = TaskPlanner::new(DummyCompiler);
        let plan = planner.plan(&includes, &excludes).unwrap();
        dbg!(&plan);
        assert_eq!(plan.tasks.len(), 2);
        assert!(!plan.global_excludes.is_empty());
    }

    #[test]
    fn empty_include_error() {
        let includes = vec![];
        let excludes = vec![];
        let planner = TaskPlanner::new(DummyCompiler);
        let err = planner.plan(&includes, &excludes).unwrap_err();
        assert!(matches!(err, Error::EmptyInclude(_)));
    }
}
