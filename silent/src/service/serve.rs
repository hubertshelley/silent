use crate::conn::SilentConnection;
use crate::route::Routes;
use crate::service::hyper_service::HyperHandler;
use std::net::SocketAddr;
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
    pub(crate) async fn call(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), hyper::Error> {
        self.conn
            .http1
            .serve_connection(stream, HyperHandler::new(peer_addr, self.routes.clone()))
            .with_upgrades()
            .await
    }
}
