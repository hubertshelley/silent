use crate::error::exception_handler_trait::ExceptionHandler;
use crate::{Configs, Response, SilentError};
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
    F: Fn(SilentError, Configs) -> Fut,
    T: Into<Response>,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }

    pub async fn handle(&self, err: SilentError, configs: Configs) -> T {
        (self.handler)(err, configs).await
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
    F: Fn(SilentError, Configs) -> Fut + Send + Sync + 'static,
    T: Into<Response>,
{
    async fn call(&self, err: SilentError, configs: Configs) -> Response {
        self.handle(err, configs).await.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StatusCode;

    async fn exception_handler(err: SilentError, _configs: Configs) -> SilentError {
        err
    }

    #[tokio::test]
    async fn handler_wrapper_match_req_works() {
        let handler_wrapper = ExceptionHandlerWrapper::new(exception_handler);
        let configs = Configs::default();
        assert_eq!(
            handler_wrapper
                .call(
                    SilentError::business_error(StatusCode::OK, "".to_string()),
                    configs
                )
                .await
                .status,
            StatusCode::OK
        );
    }
}
