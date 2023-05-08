use crate::{Request, Response, Result};
use async_trait::async_trait;

#[async_trait]
pub trait MiddleWareHandler: Send + Sync + 'static {
    async fn match_req(&self, _req: &Request) -> bool {
        true
    }
    async fn pre_request(&self, _req: &mut Request, _res: &mut Response) -> Result<()> {
        Ok(())
    }
    async fn after_response(&self, _res: &mut Response) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Uri;

    struct MiddleWare {}

    #[async_trait]
    impl MiddleWareHandler for MiddleWare {
        async fn match_req(&self, req: &Request) -> bool {
            req.uri().path().ends_with("hello")
        }
    }

    #[tokio::test]
    async fn test_middleware() -> Result<()> {
        let middleware = MiddleWare {};
        let mut req = Request::empty();
        let mut res = Response::empty();
        *req.uri_mut() = "/hello".parse::<Uri>().unwrap();
        assert!(middleware.match_req(&req).await);
        *req.uri_mut() = "/hell".parse::<Uri>().unwrap();
        assert!(!middleware.match_req(&req).await);
        assert!(middleware.pre_request(&mut req, &mut res).await.is_ok());
        assert!(middleware.after_response(&mut res).await.is_ok());
        Ok(())
    }
}
