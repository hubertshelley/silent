use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use hyper::service::Service as HyperService;
use hyper::{Request as HyperRequest, Response as HyperResponse};

use crate::core::{adapt::RequestAdapt, adapt::ResponseAdapt, res_body::ResBody};
use crate::prelude::ReqBody;
use crate::route::RootRoute;
use crate::{Request, Response};

#[doc(hidden)]
#[derive(Clone)]
pub struct HyperServiceHandler {
    pub(crate) remote_addr: SocketAddr,
    pub(crate) routes: RootRoute,
}

impl HyperServiceHandler {
    #[inline]
    pub fn new(remote_addr: SocketAddr, routes: RootRoute) -> Self {
        Self {
            remote_addr,
            routes,
        }
    }
    /// Handle [`Request`] and returns [`Response`].
    #[inline]
    pub fn handle(&self, req: Request) -> impl Future<Output = Response> {
        let Self {
            remote_addr,
            routes,
        } = self.clone();
        async move { routes.clone().handle(req, remote_addr).await }
    }
}

impl<B> HyperService<HyperRequest<B>> for HyperServiceHandler
where
    B: Into<ReqBody>,
{
    type Response = HyperResponse<ResBody>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn call(&self, req: HyperRequest<B>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = HyperRequest::from_parts(parts, body.into()).tran_to_request();
        let response = self.handle(req);
        Box::pin(async move {
            let res = response.await;
            Ok(ResponseAdapt::tran_from_response(res))
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_request() {
        // Arrange
        let remote_addr = "127.0.0.1:8080".parse().unwrap();
        let routes = RootRoute::new(); // Assuming RootRoute::new() creates a new instance of RootRoute
        let hsh = HyperServiceHandler::new(remote_addr, routes);
        let req = hyper::Request::builder().body(()).unwrap(); // Assuming Request::new() creates a new instance of Request

        // Act
        let _ = hsh.call(req).await;
    }
}
