use crate::core::request::Request;
use crate::core::response::Response;
use crate::error::SilentError;
use async_trait::async_trait;
// use bytes::Bytes;
// use std::cell::RefCell;
// use std::future::Future;
// use std::pin::Pin;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
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

// struct HandlerWrapper<Out, F>
//     where
//         F: FnOnce(Request) -> Pin<Box<dyn Future<Output=Result<Out, SilentError>> + Send>> + Send + Sync + 'static,
//         Out: Into<Bytes> + From<Bytes> + Send + Sync + 'static,
// {
//     handler: F,
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn it_works() {
//         let handler = HandlerWrapper::<_, _> {
//             handler: |req| Box::pin(async move {
//                 Ok::<_, SilentError>("Hello World".into())
//             }),
//         };
//
//         assert_eq!(
//             (handler.handler)(Request::empty()).await.unwrap(),
//             "Hello World".into()
//         );
//     }
// }
