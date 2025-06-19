use std::{path::PathBuf, sync::Arc};

use indicatif::ProgressBar;
use shared_kit_common::{
    file_utils::{
        copy::{FileTransformContext, FileTransformKind},
        path::to_relative_path,
    },
    matcher::{Matcher, MatcherResult},
    middleware_pipeline::Middleware,
};

#[derive(Debug, Clone)]
pub struct FileMatcherItem {
    pub pattern_val: String,
    pub replace_val: String,
    pub includes: Vec<String>,
}

pub struct FileTransformMiddleware {
    origin: PathBuf,
    matcher: Arc<Matcher<FileMatcherItem>>,
}
impl FileTransformMiddleware {
    pub fn new(origin: PathBuf, matcher: Arc<Matcher<FileMatcherItem>>) -> Self {
        Self { origin, matcher }
    }
}
impl Middleware<FileTransformContext, FileTransformKind> for FileTransformMiddleware {
    fn handle(
        &self,
        ctx: FileTransformContext,
        next: std::sync::Arc<
            dyn Fn(FileTransformContext) -> FileTransformKind + Send + Sync + 'static,
        >,
    ) -> FileTransformKind {
        let relative_path = to_relative_path(&self.origin, &ctx.origin);

        if let Ok(relative_path) = relative_path {
            let result = self.matcher.is_match(&relative_path.to_string_lossy());
            if let Ok(matcher_result) = result {
                match matcher_result {
                    MatcherResult::Matched(data) => {
                        if data.is_some() {
                            let file_match = data.unwrap();
                            let new_context = ctx
                                .content
                                .replace(&file_match.pattern_val, &file_match.replace_val);
                            return FileTransformKind::Transform(new_context);
                        }
                        return next(ctx);
                    }
                    MatcherResult::InExclude(_) => {
                        return FileTransformKind::Skip;
                    }
                    MatcherResult::NoMatched => {
                        return next(ctx);
                    }
                }
            }
        }

        next(ctx)
    }
}

pub struct FileProgressMiddleware {
    origin_dir: PathBuf,
    pb: Arc<ProgressBar>,
}
impl FileProgressMiddleware {
    pub fn new(origin: PathBuf, pb: Arc<ProgressBar>) -> Self {
        Self { origin_dir: origin, pb: pb }
    }
}
impl Middleware<FileTransformContext, FileTransformKind> for FileProgressMiddleware {
    fn handle(
        &self,
        ctx: FileTransformContext,
        next: Arc<dyn Fn(FileTransformContext) -> FileTransformKind + Send + Sync + 'static>,
    ) -> FileTransformKind {
        let relative_path = ctx.origin.strip_prefix(&self.origin_dir).unwrap();
        self.pb.set_message(format!("{}", relative_path.display()));
        self.pb.inc(1);

        next(ctx)
    }
}
