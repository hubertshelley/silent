use crate::route::RootRoute;
use crate::service::hyper_service::HyperServiceHandler;
use crate::Handler;
use hyper::rt::Executor;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};

pub(crate) struct Serve<S, E = TokioExecutor>
where
    S: Handler,
{
    pub(crate) routes: S,
    pub(crate) builder: Builder<E>,
}

impl<S> Serve<S>
where
    S: Handler + Clone,
{
    pub(crate) fn new(routes: S) -> Self {
        Self {
            routes,
            builder: Builder::new(TokioExecutor::new()),
        }
    }
    pub(crate) async fn call<T: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static>(
        &self,
        stream: T,
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
