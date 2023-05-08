use crate::{Request, Response, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn match_req(&self, _req: &Request) -> bool {
        true
    }

    async fn call(&self, _req: Request) -> Result<Response> {
        Ok(Response::empty())
    }
}
