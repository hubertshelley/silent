use crate::{Request, Response, SilentError};
use async_trait::async_trait;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn match_req(&self, _req: &Request) -> bool {
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
