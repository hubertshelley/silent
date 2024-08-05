use std::error::Error as StdError;
use std::net::SocketAddr;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::service::hyper_service::HyperServiceHandler;
use crate::Handler;

pub(crate) struct Serve<S, E = TokioExecutor>
where
    S: Handler,
{
    pub(crate) routes: S,
    pub(crate) builder: Builder<E>,
    pub(crate) peer_addr: SocketAddr,
}

impl<S> Serve<S>
where
    S: Handler + Clone,
{
    pub(crate) fn new(routes: S, peer_addr: SocketAddr) -> Self {
        Self {
            routes,
            builder: Builder::new(TokioExecutor::new()),
            peer_addr,
        }
    }
    pub(crate) async fn call<T: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static>(
        &self,
        stream: T,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let io = TokioIo::new(stream);
        self.builder
            .serve_connection_with_upgrades(
                io,
                HyperServiceHandler::new(self.peer_addr, self.routes.clone()),
            )
            .await
    }
}
