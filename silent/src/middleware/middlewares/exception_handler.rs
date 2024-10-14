use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;

use crate::{Configs, Handler, MiddleWareHandler, Next, Request, Response, Result};

/// ExceptionHandler 中间件
/// ```rust
/// use silent::prelude::*;
/// use silent::middlewares::{ExceptionHandler};
/// // Define a custom error handler function
/// let _ = ExceptionHandler::new(|res, _configs| async {res});
#[derive(Default, Clone)]
pub struct ExceptionHandler<F> {
    handler: Arc<F>,
}

impl<F, Fut, T> ExceptionHandler<F>
where
    Fut: Future<Output = Result<T>> + Send + 'static,
    F: Fn(Result<Response>, Configs) -> Fut + Send + Sync + 'static,
    T: Into<Response>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }
}

#[async_trait]
impl<F, Fut, T> MiddleWareHandler for ExceptionHandler<F>
where
    Fut: Future<Output = Result<T>> + Send + 'static,
    F: Fn(Result<Response>, Configs) -> Fut + Send + Sync + 'static,
    T: Into<Response>,
{
    async fn handle(&self, req: Request, next: &Next) -> Result<Response> {
        let configs = req.configs();
        self.handler.clone()(next.call(req).await, configs)
            .await
            .map(|r| r.into())
    }
}
