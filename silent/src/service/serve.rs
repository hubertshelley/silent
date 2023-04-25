use crate::conn::SilentConnection;
use crate::core::request::Request;
use crate::core::res_body::ResBody;
use crate::core::response::Response;
use crate::route::Routes;
use hyper::service::service_fn;
use hyper::Response as HyperResponse;
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
        tracing::info!("req: {:?}", self.routes);
        let service = service_fn(move |req| self.handle(req));
        self.conn.http1.serve_connection(stream, service).await
    }

    async fn handle(&self, req: Request) -> Result<HyperResponse<ResBody>, hyper::Error> {
        match self.routes.handle(req).await {
            Ok(res) => Ok(res.res),
            Err((mes, code)) => {
                tracing::error!("Failed to handle request: {:?}", mes);
                let res = Response::from(mes).set_status(code);
                Ok(res.res)
            }
        }
    }
}
