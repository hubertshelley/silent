mod hyper_service;
mod serve;

use crate::conn::SilentConnection;
use crate::route::RouteService;
use crate::service::serve::Serve;
use crate::Configs;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;

pub struct Server {
    addr: SocketAddr,
    conn: Arc<SilentConnection>,
    shutdown_callback: Option<Box<dyn Fn() + Send + Sync>>,
    configs: Option<Configs>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            addr: ([127, 0, 0, 1], 8000).into(),
            conn: Arc::new(SilentConnection::default()),
            shutdown_callback: None,
            configs: None,
        }
    }

    #[inline]
    pub fn set_configs(&mut self, configs: Configs) -> &mut Self {
        self.configs = Some(configs);
        self
    }

    #[inline]
    pub fn with_configs(mut self, configs: Configs) -> Self {
        self.configs = Some(configs);
        self
    }

    #[inline]
    pub fn bind(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = addr;
        self
    }

    pub fn set_shutdown_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.shutdown_callback = Some(Box::new(callback));
        self
    }

    pub async fn serve<S>(&self, service: S)
    where
        S: RouteService,
    {
        let Self { conn, .. } = self;
        tracing::info!("Listening on http{}{}", "://", self.addr);
        let listener = TcpListener::bind(self.addr).await.unwrap();
        let mut root_route = service.route();
        root_route.set_configs(self.configs.clone());
        #[cfg(feature = "session")]
        root_route.check_session();

        loop {
            #[cfg(unix)]
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("failed to install signal handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = async {
                let _ = std::future::pending::<()>().await;
            };
            tokio::select! {
                _ = signal::ctrl_c() => {
                    if let Some(ref callback) = self.shutdown_callback { callback() };
                    break;
                }
                _ = terminate => {
                    if let Some(ref callback) = self.shutdown_callback { callback() };
                    break;
                }
                s = listener.accept() =>{
                    match s{
                        Ok((stream, peer_addr)) => {
                            tracing::info!("Accepting from: {}", stream.peer_addr().unwrap());
                            let routes = root_route.clone();
                            let conn = conn.clone();
                            tokio::task::spawn(async move {
                                if let Err(err) = Serve::new(routes, conn).call(stream,peer_addr).await {
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

    pub fn run<S>(&self, service: S)
    where
        S: RouteService,
    {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.serve(service));
    }
}
