use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use hyper::body::Incoming;
use hyper::service::Service as HyperService;
use hyper::{Request as HyperRequest, Response as HyperResponse};

use crate::core::{adapt::RequestAdapt, res_body::ResBody};
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

impl HyperService<HyperRequest<Incoming>> for HyperServiceHandler {
    type Response = HyperResponse<ResBody>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn call(&self, req: HyperRequest<Incoming>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = HyperRequest::from_parts(parts, body.into()).tran_to_request();
        let response = self.handle(req);
        Box::pin(async move { Ok(response.await.into_hyper()) })
    }
}
