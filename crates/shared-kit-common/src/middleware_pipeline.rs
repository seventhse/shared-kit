use std::sync::Arc;

pub trait PipelineContext: Send + Sync {}

pub trait Middleware<C: PipelineContext, R>: Send + Sync {
    fn handle(&self, ctx: C, next: Arc<dyn Fn(C) -> R + Send + Sync + 'static>) -> R;
}

/// Represents a pipeline of middleware components.
///
/// Middleware components are executed in sequence, with each component having the ability to pass control to the next.
///
/// # Example
/// ```
/// use std::sync::Arc;
///
/// struct TestContext;
/// impl PipelineContext for TestContext {}
///
/// struct TestMiddleware;
/// impl Middleware<TestContext, String> for TestMiddleware {
///     fn handle(&self, ctx: TestContext, next: Arc<dyn Fn(TestContext) -> String + Send + Sync + 'static>) -> String {
///         let mut result = next(ctx);
///         result.push_str(" -> TestMiddleware");
///         result
///     }
/// }
///
/// let pipeline = MiddlewarePipeline::new()
///     .add(TestMiddleware)
///     .finalize(|_ctx| "Terminal".to_string());
///
/// let result = pipeline(TestContext);
/// assert_eq!(result, "Terminal -> TestMiddleware");
/// ```
pub struct MiddlewarePipeline<C: PipelineContext, R> {
    middlewares: Vec<Arc<dyn Middleware<C, R>>>,
}

impl<C: PipelineContext, R> MiddlewarePipeline<C, R> {
    /// Creates a new, empty middleware pipeline.
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    /// Adds a middleware to the pipeline.
    pub fn add<M: Middleware<C, R> + 'static>(mut self, mw: M) -> Self {
        self.middlewares.push(Arc::new(mw));
        self
    }

    /// Adds an optional middleware to the pipeline.
    pub fn add_option<M: Middleware<C, R> + 'static>(mut self, mw: Option<M>) -> Self {
        if let Some(mw) = mw {
            self.middlewares.push(Arc::new(mw));
        }
        self
    }

    /// Finalizes the pipeline, returning a function that executes the middleware chain.
    pub fn finalize<F>(self, terminal: F) -> impl Fn(C) -> R + Send + Sync + 'static
    where
        F: Fn(C) -> R + Send + Sync + 'static,
        C: PipelineContext + 'static,
        R: 'static,
    {
        fn build_chain<C, R>(
            middlewares: &[Arc<dyn Middleware<C, R>>],
            terminal: Arc<dyn Fn(C) -> R + Send + Sync + 'static>,
            index: usize,
        ) -> Arc<dyn Fn(C) -> R + Send + Sync + 'static>
        where
            C: PipelineContext + 'static,
            R: 'static,
        {
            if index == middlewares.len() {
                terminal
            } else {
                let mw = middlewares[index].clone();
                let next = build_chain(middlewares, terminal.clone(), index + 1);
                Arc::new(move |ctx| mw.handle(ctx, next.clone()))
            }
        }

        let terminal = Arc::new(terminal);
        let chain = build_chain(&self.middlewares, terminal, 0);

        move |ctx| chain(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestContext;
    impl PipelineContext for TestContext {}

    struct TestMiddleware;
    impl Middleware<TestContext, String> for TestMiddleware {
        fn handle(&self, ctx: TestContext, next: Arc<dyn Fn(TestContext) -> String + Send + Sync + 'static>) -> String {
            let mut result = next(ctx);
            result.push_str(" -> TestMiddleware");
            result
        }
    }

    #[test]
    fn test_pipeline_execution() {
        let pipeline = MiddlewarePipeline::new()
            .add(TestMiddleware)
            .finalize(|_ctx| "Terminal".to_string());

        let result = pipeline(TestContext);
        assert_eq!(result, "Terminal -> TestMiddleware");
    }

    #[test]
    fn test_optional_middleware() {
        let pipeline = MiddlewarePipeline::new()
            .add_option(Some(TestMiddleware))
            .add_option(None::<TestMiddleware>)
            .finalize(|_ctx| "Terminal".to_string());

        let result = pipeline(TestContext);
        assert_eq!(result, "Terminal -> TestMiddleware");
    }

    #[test]
    fn test_empty_pipeline() {
        let pipeline = MiddlewarePipeline::new().finalize(|_ctx| "Terminal".to_string());

        let result = pipeline(TestContext);
        assert_eq!(result, "Terminal");
    }
}