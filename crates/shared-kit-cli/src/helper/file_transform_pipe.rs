use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use indicatif::ProgressBar;

use crate::helper::file_system::FileTransformKind;

pub type TransformContext = (String, PathBuf);
pub type TransformNext = Arc<dyn Fn(TransformContext) -> FileTransformKind + Send + Sync>;
pub type MiddlewareFn =
    dyn Fn(TransformContext, TransformNext) -> FileTransformKind + Send + Sync + 'static;
pub type Middleware = Arc<MiddlewareFn>;

pub fn make_middleware<F>(f: F) -> Middleware
where
    F: Fn(TransformContext, TransformNext) -> FileTransformKind + Send + Sync + 'static,
{
    Arc::new(f)
}

#[derive(Default)]
pub struct FileTransformPipe {
    middlewares: Vec<Middleware>,
}

impl std::fmt::Debug for FileTransformPipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileTransformPipe")
            .field("middlewares_count", &self.middlewares.len())
            .finish()
    }
}

impl FileTransformPipe {
    #[inline]
    pub fn new() -> Self {
        Self { middlewares: vec![] }
    }

    pub fn add(mut self, middleware: Middleware) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn into_handler(
        self,
        final_handler: impl Fn(TransformContext) -> FileTransformKind + Send + Sync + 'static,
    ) -> impl Fn(&str, &Path) -> FileTransformKind + Send + Sync + 'static {
        let mut next: TransformNext = Arc::new(move |ctx| final_handler(ctx));

        for middleware in self.middlewares.into_iter().rev() {
            let curr = middleware.clone();
            let prev_next = next.clone();
            next = Arc::new(move |ctx| curr(ctx, prev_next.clone()));
        }

        move |content: &str, path: &Path| next((content.to_string(), path.to_path_buf()))
    }
}

pub fn copy_file_progress_middleware(pb: Arc<ProgressBar>, origin: PathBuf) -> Middleware {
    make_middleware(move |(_content, path), next| {
        let relative_path = path.strip_prefix(&origin).unwrap();
        pb.set_message(format!("{}", relative_path.display()));
        pb.inc(1);
        next((_content, path))
    })
}
