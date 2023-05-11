use async_trait::async_trait;
use silent::{MiddleWareHandler, Request, Response, Result};
use std::sync::Arc;

struct Middleware;

#[async_trait]
impl MiddleWareHandler for Middleware {
    async fn pre_request(&self, req: &mut Request, _res: &mut Response) -> Result<()> {
        println!("{}", req.uri());
        Ok(())
    }
    async fn after_response(&self, res: &mut Response) -> Result<()> {
        println!("{}", res.status());
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn get_middleware() -> Arc<dyn MiddleWareHandler> {
    Arc::new(Middleware {})
}
