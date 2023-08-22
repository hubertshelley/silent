use crate::error::exception_handler_trait::ExceptionHandler;
use crate::{Response, SilentError};
use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

/// 处理器包装结构体
/// 包含
/// 请求类型: Option<Method>
/// 请求方法: Handler
/// 其中请求类型可为空，用来定义中间件
/// 请求方法不可为空，用来定义处理器
/// 处理器为返回值为 Into<Bytes> + From<Bytes>的异步函数或者闭包函数
pub struct ExceptionHandlerWrapper<F> {
    handler: F,
}

#[allow(dead_code)]
impl<F, T, Fut> ExceptionHandlerWrapper<F>
where
    Fut: Future<Output = T> + Send + 'static,
    F: Fn(SilentError) -> Fut,
    T: Into<Response>,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }

    pub async fn handle(&self, err: SilentError) -> T {
        (self.handler)(err).await
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

/// 为HandlerWrapper实现Handler
#[async_trait]
impl<F, T, Fut> ExceptionHandler for ExceptionHandlerWrapper<F>
where
    Fut: Future<Output = T> + Send + 'static,
    F: Fn(SilentError) -> Fut + Send + Sync + 'static,
    T: Into<Response>,
{
    async fn call(&self, err: SilentError) -> Response {
        self.handle(err).await.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StatusCode;

    async fn exception_handler(err: SilentError) -> SilentError {
        err
    }

    #[tokio::test]
    async fn handler_wrapper_match_req_works() {
        let handler_wrapper = ExceptionHandlerWrapper::new(exception_handler);
        assert_eq!(
            handler_wrapper
                .call(SilentError::business_error(StatusCode::OK, "".to_string()))
                .await
                .status_code,
            StatusCode::OK
        );
    }
}
