use crate::conn::SilentConnection;
use crate::core::res_body::ResBody;
use crate::core::response::Response;
use crate::route::Routes;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use std::sync::Arc;
use tokio::net::TcpStream;

pub(crate) struct Serve {
    pub(crate) routes: Routes,
    pub(crate) conn: Arc<SilentConnection>,
}

impl Serve {
    pub(crate) fn new(routes: Routes, conn: Arc<SilentConnection>) -> Self {
        Self { routes, conn }
    }
    pub(crate) async fn call(&self, stream: TcpStream) -> Result<(), hyper::Error> {
        let service = service_fn(move |req| self.handle(req));
        self.conn
            .http1
            .serve_connection(stream, service)
            .with_upgrades()
            .await
    }

    async fn handle(
        &self,
        req: HyperRequest<Incoming>,
    ) -> Result<HyperResponse<ResBody>, hyper::Error> {
        let (parts, body) = req.into_parts();
        let req = HyperRequest::from_parts(parts, body.into()).into();
        match self.routes.handle(req).await {
            Ok(res) => Ok(res.res),
            Err((mes, code)) => {
                tracing::error!("Failed to handle request: {:?}", mes);
                let mut res = Response::from(mes);
                res.set_status(code);
                Ok(res.res)
            }
        }
    }
}
