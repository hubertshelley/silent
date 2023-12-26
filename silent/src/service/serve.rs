use crate::conn::support::TokioIo;
use crate::conn::SilentConnection;
use crate::route::RootRoute;
use crate::service::hyper_service::HyperServiceHandler;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;

pub(crate) struct Serve {
    pub(crate) routes: RootRoute,
    pub(crate) conn: Arc<SilentConnection>,
}

impl Serve {
    pub(crate) fn new(routes: RootRoute, conn: Arc<SilentConnection>) -> Self {
        Self { routes, conn }
    }
    pub(crate) async fn call(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), hyper::Error> {
        let io = TokioIo::new(stream);
        self.conn
            .http1
            .serve_connection(io, HyperServiceHandler::new(peer_addr, self.routes.clone()))
            .with_upgrades()
            .await
    }
}
