use crate::handler::handler_trait::Handler;
use crate::{Request, Response, Result};
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
pub struct HandlerWrapperResponse<F> {
    handler: F,
}

impl<F, Fut> HandlerWrapperResponse<F>
where
    Fut: Future<Output = Result<Response>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut,
{
    pub fn new(handler: F) -> Self {
        HandlerWrapperResponse { handler }
    }

    pub async fn handle(&self, req: Request) -> Result<Response> {
        (self.handler)(req).await
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

/// 为HandlerWrapper实现Handler
#[async_trait]
impl<F, Fut> Handler for HandlerWrapperResponse<F>
where
    Fut: Future<Output = Result<Response>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
{
    async fn call(&self, req: Request) -> Result<Response> {
        self.handle(req).await
    }
}

impl<F, Fut> From<F> for HandlerWrapperResponse<F>
where
    Fut: Future<Output = Result<Response>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
{
    fn from(handler: F) -> Self {
        Self { handler }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Request, Result, StatusCode};

    async fn hello_world(_req: Request) -> Result<Response> {
        Ok(Response::empty())
    }

    #[tokio::test]
    async fn handler_wrapper_match_req_works() {
        let handler_wrapper = HandlerWrapperResponse::new(hello_world);
        let req = Request::empty();
        assert!(handler_wrapper.match_req(&req).await);
        let res = handler_wrapper.call(req).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().status_code, StatusCode::OK);
    }
}
