use std::future::Future;
use std::pin::Pin;

use hyper::body::Incoming;
use hyper::service::Service as HyperService;
use tower_service::Service;

use crate::log::error;

#[doc(hidden)]
#[derive(Clone)]
pub struct GrpcService {
    pub(crate) handler: axum::Router<()>,
}

impl GrpcService {
    #[inline]
    #[allow(dead_code)]
    pub fn new(handler: axum::Router<()>) -> Self {
        Self { handler }
    }
}

impl HyperService<hyper::Request<Incoming>> for GrpcService {
    type Response = axum::response::Response;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn call(&self, req: hyper::Request<Incoming>) -> Self::Future {
        let mut handler = self.handler.clone();
        Box::pin(async move {
            let res = handler
                .call(req)
                .await
                .map_err(|e| {
                    error!(error = ?e, "call grpc router failed: {}", e);
                })
                .unwrap();
            Ok(res)
        })
    }
}
