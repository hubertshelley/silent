mod hyper_service;
mod serve;

use crate::conn::SilentConnection;
use crate::error::ExceptionHandlerWrapper;
use crate::route::{Route, Routes};
use crate::service::serve::Serve;
#[cfg(feature = "session")]
use crate::session::SessionMiddleware;
#[cfg(feature = "template")]
use crate::templates::TemplateMiddleware;
use crate::{Response, SilentError};
#[cfg(feature = "session")]
use async_session::SessionStore;
use std::cell::OnceCell;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::RwLock;

pub struct Server {
    pub routes: Arc<RwLock<Routes>>,
    addr: SocketAddr,
    conn: Arc<SilentConnection>,
    rt: OnceCell<Runtime>,
    shutdown_callback: Option<Box<dyn Fn() + Send + Sync>>,
    #[cfg(feature = "session")]
    session_set: bool,
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
            rt: OnceCell::new(),
            shutdown_callback: None,
            #[cfg(feature = "session")]
            session_set: false,
        }
    }

    pub fn bind(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = addr;
        self
    }

    #[cfg(feature = "session")]
    pub fn set_session_store<S: SessionStore>(&mut self, session: S) -> &mut Self {
        self.rt
            .get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
            })
            .block_on(self.routes.write())
            .hook_first(SessionMiddleware::new(session));
        self.session_set = true;
        self
    }

    #[cfg(feature = "template")]
    pub fn set_template_dir(&mut self, dir: impl Into<String>) -> &mut Self {
        self.rt
            .get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
            })
            .block_on(self.routes.write())
            .hook(TemplateMiddleware::new(dir.into().as_str()));
        self
    }

    pub fn set_shutdown_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.shutdown_callback = Some(Box::new(callback));
        self
    }

    pub fn set_exception_handler<F, T, Fut>(&mut self, handler: F) -> &mut Self
    where
        Fut: Future<Output = T> + Send + 'static,
        F: Fn(SilentError) -> Fut + Send + Sync + 'static,
        T: Into<Response>,
    {
        self.rt
            .get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
            })
            .block_on(self.routes.write())
            .set_exception_handler(ExceptionHandlerWrapper::new(handler).arc());
        self
    }

    pub fn bind_route(&mut self, route: Route) -> &mut Self {
        self.rt
            .get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
            })
            .block_on(self.routes.write())
            .add(route);
        self
    }

    pub async fn bind_route_async(&mut self, route: Route) -> &mut Self {
        self.routes.write().await.add(route);
        self
    }

    pub async fn serve(&self) {
        #[cfg(feature = "session")]
        let session_set = self.session_set;
        let Self { conn, routes, .. } = self;
        #[cfg(feature = "session")]
        if !session_set {
            routes
                .write()
                .await
                .hook_first(SessionMiddleware::default());
        };
        tracing::info!("Listening on {}", self.addr);
        let listener = TcpListener::bind(self.addr).await.unwrap();

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
                let _ = std::future::pending::<()>();
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
                                let routes = routes.read().await.clone();
                                let conn = conn.clone();
                                self.rt.get_or_init(||tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
            ).spawn(async move {
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

    pub fn runtime(&self) -> &Runtime {
        self.rt.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        })
    }

    pub fn set_runtime(self, rt: Runtime) -> Self {
        self.rt.set(rt).unwrap();
        self
    }

    pub fn run(&self) {
        self.rt
            .get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
            })
            .block_on(self.serve());
    }
}
