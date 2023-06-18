use crate::handler::handler_trait::Handler;
use crate::{Request, Response, Result};
use async_trait::async_trait;
use bytes::Bytes;
use serde::Serialize;
use serde_json::Value;
use std::future::Future;
use std::sync::Arc;

/// 处理器包装结构体
/// 包含
/// 请求类型: Option<Method>
/// 请求方法: Handler
/// 其中请求类型可为空，用来定义中间件
/// 请求方法不可为空，用来定义处理器
/// 处理器为返回值为 Into<Bytes> + From<Bytes>的异步函数或者闭包函数
pub struct HandlerWrapper<F> {
    handler: F,
}

impl<F, T, Fut> HandlerWrapper<F>
where
    Fut: Future<Output = Result<T>> + Send + 'static,
    F: Fn(Request) -> Fut,
    T: Serialize + Send,
{
    pub fn new(handler: F) -> Self {
        HandlerWrapper { handler }
    }

    pub async fn handle(&self, req: Request) -> Result<Bytes> {
        let result = (self.handler)(req).await?;
        let result = serde_json::to_value(&result)?;
        match result {
            Value::String(value) => Ok(value.into_bytes().into()),
            _ => Ok(serde_json::to_vec(&result)?.into()),
        }
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

/// 为HandlerWrapper实现Handler
#[async_trait]
impl<F, T, Fut> Handler for HandlerWrapper<F>
where
    Fut: Future<Output = Result<T>> + Send + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    T: Serialize + Send + 'static,
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
    use serde::{Deserialize, Serialize};

    async fn hello_world(_req: Request) -> Result<String> {
        Ok("Hello World".into())
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct HelloHandler {
        name: String,
    }

    async fn hello_world_2(_req: Request) -> Result<HelloHandler> {
        Ok(HelloHandler {
            name: "Hello World".to_string(),
        })
    }

    #[tokio::test]
    async fn handler_wrapper_match_req_works() {
        let handler_wrapper = HandlerWrapper::new(hello_world);
        let req = Request::empty();
        assert!(handler_wrapper.match_req(&req).await);
    }

    #[tokio::test]
    async fn handler_wrapper_works() {
        let handler_wrapper = HandlerWrapper::new(hello_world);

        assert_eq!(
            handler_wrapper.handle(Request::empty()).await.unwrap(),
            "Hello World".to_string()
        );
    }

    #[tokio::test]
    async fn handler_wrapper_struct_works() {
        let handler_wrapper = HandlerWrapper::new(hello_world_2);
        let hello = HelloHandler {
            name: "Hello World".to_string(),
        };
        assert_eq!(
            handler_wrapper.handle(Request::empty()).await.unwrap(),
            serde_json::to_string(&hello).unwrap()
        );
    }
}
