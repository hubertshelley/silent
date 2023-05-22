use crate::handler::handler_trait::Handler;
use crate::{Request, Response, Result};
use async_trait::async_trait;
use bytes::Bytes;
use serde::Serialize;
use serde_json::Value;
use std::future::Future;

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

enum ResponseType<T> {
    Serialize(T),
    Response(Response),
}

impl<T> From<Response> for ResponseType<T> {
    fn from(res: Response) -> Self {
        ResponseType::Response(res)
    }
}

impl<T> From<T> for ResponseType<T>
where
    T: Serialize + Send,
{
    fn from(value: T) -> Self {
        ResponseType::Serialize(value)
    }
}

impl<F, T, Fut> HandlerWrapper<F>
where
    Fut: Future<Output = Result<T>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut,
    T: Serialize + Send,
{
    pub fn new(handler: F) -> Self {
        HandlerWrapper { handler }
    }

    pub async fn handle(&self, req: Request) -> Result<HandlerResult> {
        let result = (self.handler)(req).await?;
        let result: ResponseType<T> = result.into();
        match result {
            ResponseType::Serialize(result) => {
                let result = serde_json::to_value(&result)?;
                match result {
                    Value::String(value) => {
                        let bts: Bytes = value.into_bytes().into();
                        Ok(bts.into())
                    }
                    _ => {
                        let bts: Bytes = serde_json::to_vec(&result)?.into();
                        Ok(bts.into())
                    }
                }
            }
            ResponseType::Response(res) => Ok(res.into()),
        }
    }
}

pub enum HandlerResult {
    Bytes(Bytes),
    Response(Response),
}

impl From<Bytes> for HandlerResult {
    fn from(bytes: Bytes) -> Self {
        HandlerResult::Bytes(bytes)
    }
}

impl From<Response> for HandlerResult {
    fn from(res: Response) -> Self {
        HandlerResult::Response(res)
    }
}

/// 为HandlerWrapper实现Handler
#[async_trait]
impl<F, T, Fut> Handler for HandlerWrapper<F>
where
    Fut: Future<Output = Result<T>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    T: Serialize + Send,
{
    async fn call(&self, req: Request) -> Result<Response> {
        let res = self.handle(req).await?;
        match res {
            HandlerResult::Bytes(result) => Ok(Response::from(result)),
            HandlerResult::Response(res) => Ok(res),
        }
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
        let result = handler_wrapper.handle(Request::empty()).await.unwrap();
        match result {
            HandlerResult::Bytes(result) => {
                assert_eq!(result, "Hello World".to_string().into_bytes());
            }
            HandlerResult::Response(_) => {
                panic!("should be bytes")
            }
        }
    }

    #[tokio::test]
    async fn handler_wrapper_struct_works() {
        let handler_wrapper = HandlerWrapper::new(hello_world_2);
        let hello = HelloHandler {
            name: "Hello World".to_string(),
        };
        let result = handler_wrapper.handle(Request::empty()).await.unwrap();
        match result {
            HandlerResult::Bytes(result) => {
                assert_eq!(result, serde_json::to_string(&hello).unwrap());
            }
            HandlerResult::Response(_) => {
                panic!("should be bytes")
            }
        }
    }

    // #[tokio::test]
    // async fn handler_wrapper_response_works() {
    //     let handler_wrapper = HandlerWrapper::new(|res| async {
    //         Ok(Response::empty())
    //     });
    //     let result = handler_wrapper.handle(Request::empty()).await.unwrap();
    //     match result {
    //         HandlerResult::Bytes(result) => {
    //             panic!("should be bytes")
    //         }
    //         HandlerResult::Response(_) => { panic!("should be bytes") }
    //     }
    // }
}
