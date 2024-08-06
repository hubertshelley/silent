use std::error::Error as StdError;
use std::net::SocketAddr;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpStream;

use crate::route::RootRoute;
use crate::service::hyper_service::HyperServiceHandler;

pub(crate) struct Serve<E = TokioExecutor> {
    pub(crate) routes: RootRoute,
    pub(crate) builder: Builder<E>,
}

impl Serve {
    pub(crate) fn new(routes: RootRoute) -> Self {
        Self {
            routes,
            builder: Builder::new(TokioExecutor::new()),
        }
    }
    pub(crate) async fn call(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let io = TokioIo::new(stream);
        self.builder
            .serve_connection_with_upgrades(
                io,
                HyperServiceHandler::new(peer_addr, self.routes.clone()),
            )
            .await
    }
}
