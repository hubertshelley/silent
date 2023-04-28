use crate::core::request::Request;
use crate::core::response::Response;
use crate::error::SilentError;
use async_trait::async_trait;
use bytes::Bytes;
use hyper::Method;
use serde::Serialize;
use std::future::Future;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn match_method(&self, _method: &Method) -> bool {
        true
    }

    async fn call(&self, _req: Request) -> Result<Response, SilentError> {
        Ok(Response::empty())
    }
    async fn middleware_call(
        &self,
        _req: &mut Request,
        _res: &mut Response,
    ) -> Result<(), SilentError> {
        Ok(())
    }
}

/// 处理器包装结构体
/// 包含
/// 请求类型: Option<Method>
/// 请求方法: Handler
/// 其中请求类型可为空，用来定义中间件
/// 请求方法不可为空，用来定义处理器
/// 处理器为返回值为 Into<Bytes> + From<Bytes>的异步函数或者闭包函数
pub struct HandlerWrapper<F> {
    pub(crate) method: Option<Method>,
    handler: F,
}

impl<F, T, Fut> HandlerWrapper<F>
where
    Fut: Future<Output = Result<T, SilentError>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut,
    T: Serialize + Send,
{
    pub fn new(method: Option<Method>, handler: F) -> Self {
        HandlerWrapper { method, handler }
    }

    pub async fn handle(&self, req: Request) -> Result<Bytes, SilentError> {
        let result = (self.handler)(req).await?;
        Ok(serde_json::to_vec(&result)?.into())
    }
}

/// 为HandlerWrapper实现Handler
#[async_trait]
impl<F, T, Fut> Handler for HandlerWrapper<F>
where
    Fut: Future<Output = Result<T, SilentError>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    T: Serialize + Send + 'static,
{
    async fn match_method(&self, method: &Method) -> bool {
        if self.method.is_none() {
            return false;
        }
        self.method.as_ref().unwrap() == method
    }

    async fn call(&self, req: Request) -> Result<Response, SilentError> {
        let res = self.handle(req).await?;
        Ok(Response::from(res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    async fn hello_world(_req: Request) -> Result<String, SilentError> {
        Ok("Hello World".into())
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct HelloHandler {
        name: String,
    }

    async fn hello_world_2(_req: Request) -> Result<HelloHandler, SilentError> {
        Ok(HelloHandler {
            name: "Hello World".to_string(),
        })
    }

    #[tokio::test]
    async fn handler_wrapper_match_method_works() {
        let handler_wrapper = HandlerWrapper::new(Some(Method::GET), hello_world);
        let req = Request::empty();
        let method = req.method();
        assert!(handler_wrapper.match_method(method).await);
    }

    #[tokio::test]
    async fn handler_wrapper_works() {
        let handler_wrapper = HandlerWrapper::new(None, hello_world);

        assert_eq!(
            handler_wrapper.handle(Request::empty()).await.unwrap(),
            "\"Hello World\"".to_string()
        );
    }

    #[tokio::test]
    async fn handler_wrapper_struct_works() {
        let handler_wrapper = HandlerWrapper::new(None, hello_world_2);
        let hello = HelloHandler {
            name: "Hello World".to_string(),
        };
        assert_eq!(
            handler_wrapper.handle(Request::empty()).await.unwrap(),
            serde_json::to_string(&hello).unwrap()
        );
    }
}
