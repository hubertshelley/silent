use crate::handler::handler_trait::Handler;
use crate::{Request, Response, Result};
use async_trait::async_trait;
use bytes::Bytes;
use std::future::Future;

/// 处理器包装结构体
/// 包含
/// 请求类型: Option<Method>
/// 请求方法: Handler
/// 其中请求类型可为空，用来定义中间件
/// 请求方法不可为空，用来定义处理器
/// 处理器为返回值为 Into<Bytes> + From<Bytes>的异步函数或者闭包函数
pub struct HandlerWrapperHtml<F> {
    handler: F,
}

impl<F, Fut> HandlerWrapperHtml<F>
where
    Fut: Future<Output = Result<&'static str>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }

    pub async fn handle(&self, req: Request) -> Result<Bytes> {
        let result = (self.handler)(req).await?;
        Ok(result.into())
    }
}

/// 为HandlerWrapper实现Handler
#[async_trait]
impl<F, Fut> Handler for HandlerWrapperHtml<F>
where
    Fut: Future<Output = Result<&'static str>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
{
    async fn call(&self, req: Request) -> Result<Response> {
        let res = self.handle(req).await?;
        Ok(Response::from(res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Request, Result};

    async fn hello_world(_req: Request) -> Result<&'static str> {
        Ok("Hello World")
    }

    #[tokio::test]
    async fn handler_wrapper_match_req_works() {
        let handler_wrapper = HandlerWrapperHtml::new(hello_world);
        let req = Request::empty();
        assert!(handler_wrapper.match_req(&req).await);
    }
}
