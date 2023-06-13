use crate::core::res_body::ResBody;
use crate::route::Routes;
use crate::{Request, Response};
use hyper::body::Incoming;
use hyper::service::Service as HyperService;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

#[doc(hidden)]
#[derive(Clone)]
pub struct HyperHandler {
    pub(crate) remote_addr: SocketAddr,
    pub(crate) routes: Routes,
}

impl HyperHandler {
    #[inline]
    pub fn new(remote_addr: SocketAddr, routes: Routes) -> Self {
        Self {
            remote_addr,
            routes,
        }
    }
    /// Handle [`Request`] and returns [`Response`].
    #[inline]
    pub fn handle(&self, req: Request) -> impl Future<Output = Response> {
        let mut response = Response::empty();
        let Self {
            remote_addr,
            routes,
        } = self.clone();
        async move {
            match routes.clone().handle(req, remote_addr).await {
                Ok(res) => {
                    response = res;
                }
                Err((mes, code)) => {
                    tracing::error!("Failed to handle request: {:?}", mes);
                    response.set_body(mes.into());
                    response.set_status(code);
                }
            }
            response
        }
    }
}

impl HyperService<HyperRequest<Incoming>> for HyperHandler {
    type Response = HyperResponse<ResBody>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn call(&mut self, req: HyperRequest<Incoming>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = HyperRequest::from_parts(parts, body.into()).into();
        let response = self.handle(req);
        Box::pin(async move { Ok(response.await.into_hyper()) })
    }
}
