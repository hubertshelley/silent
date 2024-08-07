use crate::error::{ExceptionHandler, ExceptionHandlerWrapper};
#[cfg(feature = "grpc")]
use crate::grpc::GrpcHandler;
#[cfg(feature = "grpc")]
use crate::prelude::HandlerGetter;
use crate::route::handler_match::{Match, RouteMatched};
use crate::route::Route;
#[cfg(feature = "session")]
use crate::session::SessionMiddleware;
#[cfg(feature = "template")]
use crate::templates::TemplateMiddleware;
#[cfg(feature = "scheduler")]
use crate::Scheduler;
use crate::{Configs, Handler, MiddleWareHandler, Next, Request, Response, SilentError};
#[cfg(feature = "session")]
use async_session::SessionStore;
use async_trait::async_trait;
use chrono::Utc;
#[cfg(feature = "grpc")]
use http::Method;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
#[cfg(feature = "scheduler")]
use tokio::sync::Mutex;
#[cfg(feature = "grpc")]
use tonic::body::BoxBody;
#[cfg(feature = "grpc")]
use tonic::codegen::Service;
#[cfg(feature = "grpc")]
use tonic::server::NamedService;

#[derive(Clone, Default)]
pub struct RootRoute {
    pub(crate) children: Vec<Route>,
    pub(crate) middlewares: Vec<Arc<dyn MiddleWareHandler>>,
    pub(crate) exception_handler: Option<Arc<dyn ExceptionHandler>>,
    #[cfg(feature = "session")]
    pub(crate) session_set: bool,
    pub(crate) configs: Option<Configs>,
    #[cfg(feature = "scheduler")]
    pub(crate) scheduler: Arc<Mutex<Scheduler>>,
}

impl fmt::Debug for RootRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self
            .children
            .iter()
            .map(|route| format!("{:?}", route))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", path)
    }
}

impl RootRoute {
    pub fn new() -> Self {
        Self {
            children: vec![],
            middlewares: vec![],
            exception_handler: None,
            #[cfg(feature = "session")]
            session_set: false,
            configs: None,
            #[cfg(feature = "scheduler")]
            scheduler: Arc::new(Mutex::new(Scheduler::new())),
        }
    }

    #[cfg(feature = "grpc")]
    pub fn grpc<S>(&mut self, grpc: GrpcHandler<S>)
    where
        S: Service<http::Request<BoxBody>, Response = http::Response<BoxBody>> + NamedService,
        S: Clone + Send + 'static,
        S: Sync + Send + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        let path = grpc.path().to_string();
        let handler = Arc::new(grpc);
        let route = Route::new(path.as_str()).append(
            Route::new("<path:**>")
                .insert_handler(Method::POST, handler.clone())
                .insert_handler(Method::GET, handler),
        );
        self.push(route)
    }

    #[cfg(feature = "grpc")]
    pub fn with_grpc<S>(mut self, grpc: GrpcHandler<S>) -> Self
    where
        S: Service<http::Request<BoxBody>, Response = http::Response<BoxBody>> + NamedService,
        S: Clone + Send + 'static,
        S: Sync + Send + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        self.grpc(grpc);
        self
    }

    pub fn push(&mut self, route: Route) {
        self.middlewares.extend(route.root_middlewares.clone());
        self.children.push(route);
    }

    pub fn hook(&mut self, handler: impl MiddleWareHandler + 'static) {
        let handler = Arc::new(handler);
        self.middlewares.push(handler.clone());
        self.children
            .iter_mut()
            .for_each(|r| r.middleware_hook(handler.clone()));
    }
    #[allow(dead_code)]
    pub(crate) fn hook_first(&mut self, handler: impl MiddleWareHandler + 'static) {
        let handler = Arc::new(handler);
        self.middlewares.insert(0, handler.clone());
        self.children
            .iter_mut()
            .for_each(|r| r.middleware_hook_first(handler.clone()));
    }

    pub fn set_exception_handler<F, T, Fut>(mut self, handler: F) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
        F: Fn(SilentError, Configs) -> Fut + Send + Sync + 'static,
        T: Into<Response>,
    {
        self.exception_handler = Some(ExceptionHandlerWrapper::new(handler).arc());
        self
    }
    pub(crate) fn set_configs(&mut self, configs: Option<Configs>) {
        self.configs = configs;
    }
}

#[async_trait]
impl Handler for RootRoute {
    async fn call(&self, mut req: Request) -> Result<Response, SilentError> {
        tracing::debug!("{:?}", req);
        let exception_handler = self.exception_handler.clone();
        let configs = self.configs.clone().unwrap_or_default();
        req.configs = configs.clone();
        let method = req.method().clone();
        let url = req.uri().to_string().clone();
        let http_version = req.version();
        let peer_addr = req.remote();
        let start_time = Utc::now().time();
        #[cfg(feature = "scheduler")]
        req.extensions_mut().insert(self.scheduler.clone());

        let mut root_middlewares = vec![];
        for middleware in self.middlewares.iter().cloned() {
            if middleware.match_req(&req).await {
                root_middlewares.push(middleware);
            }
        }
        let (mut req, path) = req.split_url();
        let res = match self.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(route) => {
                let next = Next::build(Arc::new(route), root_middlewares);
                next.call(req).await
            }
            RouteMatched::Unmatched => Err(SilentError::NotFound),
        };
        let end_time = Utc::now().time();
        let req_time = end_time - start_time;
        Ok(match res {
            Ok(res) => {
                tracing::info!(
                    "{} {} {} {:?} {} {:?} {}",
                    peer_addr,
                    method,
                    url,
                    http_version,
                    res.status.as_u16(),
                    res.content_length().lower(),
                    req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0
                );
                res
            }
            Err(e) => {
                tracing::error!(
                    "{} {} {} {:?} {} {:?} {} {}",
                    peer_addr,
                    method,
                    url,
                    http_version,
                    e.status().as_u16(),
                    0,
                    req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0,
                    e.to_string()
                );
                match exception_handler {
                    Some(handler) => handler.call(e, configs.clone()).await,
                    None => e.into(),
                }
            }
        })
    }
}

impl RootRoute {
    #[cfg(feature = "session")]
    pub fn set_session_store<S: SessionStore>(&mut self, session: S) -> &mut Self {
        self.hook_first(SessionMiddleware::new(session));
        self.session_set = true;
        self
    }
    #[cfg(feature = "session")]
    pub fn check_session(&mut self) {
        if !self.session_set {
            self.hook_first(SessionMiddleware::default())
        }
    }

    #[cfg(feature = "template")]
    pub fn set_template_dir(&mut self, dir: impl Into<String>) -> &mut Self {
        self.hook(TemplateMiddleware::new(dir.into().as_str()));
        self
    }
}
