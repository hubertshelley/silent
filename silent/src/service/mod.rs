mod serve;

use crate::conn::SilentConnection;
use crate::route::{Route, Routes};
use crate::service::serve::Serve;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::RwLock;

pub struct Server {
    pub routes: Arc<RwLock<Routes>>,
    addr: SocketAddr,
    conn: Arc<SilentConnection>,
    rt: Runtime,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(Routes::new())),
            addr: ([127, 0, 0, 1], 8000).into(),
            conn: Arc::new(SilentConnection::default()),
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        }
    }

    pub fn bind(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = addr;
        self
    }

    pub fn bind_route(&mut self, route: Route) -> &mut Self {
        self.rt.block_on(self.routes.write()).add(route);
        self
    }

    pub async fn serve(&self) {
        let Self { conn, routes, .. } = self;
        tracing::info!("Listening on http://{}", self.addr);
        let listener = TcpListener::bind(self.addr).await.unwrap();
        // 捕获SIGINT信号
        let (tx, mut rx) = unbounded_channel();

        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
            .expect("Error setting Ctrl-C handler");
        loop {
            tokio::select! {
                _ = rx.recv() => {
                    tracing::info!("Received Ctrl-C, shutting down");
                    break;
                }
                s = listener.accept() =>{
                    match s{
                        Ok((stream, _)) => {
                            tracing::debug!("Accepting from: {}", stream.peer_addr().unwrap());
                            let routes = routes.read().await.clone();
                            let conn = conn.clone();
                            tokio::task::spawn(async move {
                                if let Err(err) = Serve::new(routes, conn).call(stream).await {
                                    tracing::error!("Failed to serve connection: {:?}", err);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "accept connection failed");
                        }
                    }
                }
            }
        }
    }

    pub fn run(&self) {
        self.rt.block_on(self.serve());
    }
}
