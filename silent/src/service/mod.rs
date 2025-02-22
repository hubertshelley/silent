mod hyper_service;
mod serve;

use crate::Configs;
#[cfg(feature = "scheduler")]
use crate::Scheduler;
use crate::core::listener::Listener;
use crate::route::RouteService;
use crate::service::serve::Serve;
use std::net::SocketAddr;
use tokio::net::{TcpListener, UnixListener};
use tokio::signal;
use tokio::task::JoinSet;

pub struct Server {
    addr: Option<SocketAddr>,
    path: Option<String>,
    listener: Option<Listener>,
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
            addr: None,
            path: None,
            listener: None,
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
    pub fn bind(mut self, addr: SocketAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    #[inline]
    pub fn bind_unix<P: Into<String>>(mut self, path: P) -> Self {
        self.path = Some(path.into());
        self
    }

    #[inline]
    pub fn listen<T: Into<Listener>>(mut self, listener: T) -> Self {
        self.listener = Some(listener.into());
        self
    }

    pub fn set_shutdown_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.shutdown_callback = Some(Box::new(callback));
        self
    }

    pub async fn serve<S>(self, service: S)
    where
        S: RouteService,
    {
        let Self {
            listener,
            configs,
            addr,
            path,
            ..
        } = self;

        let listener = match listener {
            None => match (addr, path.clone()) {
                (None, None) => Listener::TokioListener(
                    TcpListener::bind("127.0.0.1:0")
                        .await
                        .expect("failed to listen"),
                ),
                (Some(addr), _) => Listener::TokioListener(
                    TcpListener::bind(addr)
                        .await
                        .unwrap_or_else(|_| panic!("failed to listen {}", addr)),
                ),
                (None, Some(path)) => {
                    let _ = tokio::fs::remove_file(&path).await;
                    Listener::TokioUnixListener(
                        UnixListener::bind(path.clone())
                            .unwrap_or_else(|_| panic!("failed to listen {}", path)),
                    )
                }
            },
            Some(listener) => listener,
        };
        tracing::info!("listening on: {:?}", listener.local_addr().unwrap());
        let mut root_route = service.route();
        root_route.set_configs(configs.clone());
        #[cfg(feature = "session")]
        root_route.check_session();
        #[cfg(feature = "cookie")]
        root_route.check_cookie();
        #[cfg(feature = "scheduler")]
        let scheduler = root_route.scheduler.clone();
        #[cfg(feature = "scheduler")]
        tokio::spawn(async move {
            Scheduler::schedule(scheduler).await;
        });
        let mut join_set = JoinSet::new();
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
                            join_set.spawn(async move {
                                if let Err(err) = Serve::new(routes).call(stream,peer_addr).await {
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

    pub fn run<S>(self, service: S)
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
