use crate::{Response, SilentError};
use async_trait::async_trait;

#[async_trait]
pub trait ExceptionHandler: Send + Sync + 'static {
    async fn call(&self, err: SilentError) -> Response;
}
