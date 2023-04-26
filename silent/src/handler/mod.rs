use crate::core::request::Request;
use crate::core::response::Response;
use crate::error::SilentError;
use async_trait::async_trait;
// use std::future::Future;

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

// struct HandlerWrapper {
//     handler: Box<dyn FnOnce(Request) -> dyn Future<Result<dyn <Into<Bytes>>, SilentError>> + Send + Sync + 'static>,
// }
