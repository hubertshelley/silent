use crate::error::{ExceptionHandler, ExceptionHandlerWrapper};
use crate::route::handler_match::{Match, RouteMatched};
use crate::route::Route;
#[cfg(feature = "session")]
use crate::session::SessionMiddleware;
#[cfg(feature = "template")]
use crate::templates::TemplateMiddleware;
#[cfg(feature = "scheduler")]
use crate::Scheduler;
#[cfg(feature = "grpc")]
use crate::{grpc::GrpcHandler, Handler};
use crate::{
    Configs, MiddleWareHandler, MiddlewareResult, Request, Response, SilentError, StatusCode,
};
#[cfg(feature = "session")]
use async_session::{Session, SessionStore};
use chrono::Utc;
use http::{header, HeaderValue};
use mime::Mime;
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
#[cfg(feature = "scheduler")]
use tokio::sync::Mutex;

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
    #[cfg(feature = "grpc")]
    pub(crate) grpc: Option<GrpcHandler>,
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
            #[cfg(feature = "grpc")]
            grpc: None,
        }
    }

    #[cfg(feature = "grpc")]
    pub fn grpc(&mut self, grpc: GrpcHandler) {
        self.grpc = Some(grpc);
    }

    #[cfg(feature = "grpc")]
    pub fn with_grpc(mut self, grpc: GrpcHandler) -> Self {
        self.grpc = Some(grpc);
        self
    }

    pub fn push(&mut self, route: Route) {
        self.middlewares.clone_from(&route.middlewares);
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

impl RootRoute {
    async fn handle_request(
        &self,
        mut req: Request,
        path: String,
    ) -> Result<Response, SilentError> {
        let configs = self.configs.clone().unwrap_or_default();
        let mut pre_res = Response::empty();
        let mut root_middlewares = vec![];
        for middleware in self.middlewares.iter() {
            if middleware.match_req(&req).await {
                root_middlewares.push(middleware);
            }
        }
        for middleware in root_middlewares.clone() {
            match middleware.pre_request(&mut req, &mut pre_res).await? {
                MiddlewareResult::Continue => {}
                MiddlewareResult::Break(res) => return Ok(res),
                MiddlewareResult::Error(err) => return Err(err),
            }
        }
        if req.content_type() == Some(Mime::from_str("application/grpc").unwrap())
            || req.headers().get(header::UPGRADE) == Some(&HeaderValue::from_static("h2c"))
        {
            #[cfg(feature = "grpc")]
            {
                return if let Some(grpc) = self.grpc.clone() {
                    grpc.call(req).await
                } else {
                    Err(SilentError::business_error(
                        StatusCode::NOT_IMPLEMENTED,
                        "grpc handler not set".to_string(),
                    ))
                };
            }
            #[cfg(not(feature = "grpc"))]
            {
                return Err(SilentError::business_error(
                    StatusCode::NOT_IMPLEMENTED,
                    "grpc not enabled".to_string(),
                ));
            }
        }
        match self.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(route) => match route.handler.get(req.method()) {
                None => {
                    return Err(SilentError::business_error(
                        StatusCode::METHOD_NOT_ALLOWED,
                        "method not allowed".to_string(),
                    ))
                }
                Some(handler) => {
                    pre_res.configs = configs.clone();
                    let mut active_middlewares = vec![];
                    for middleware in route.middlewares.iter() {
                        if middleware.match_req(&req).await {
                            active_middlewares.push(middleware);
                        }
                    }
                    for middleware in active_middlewares.clone() {
                        match middleware.pre_request(&mut req, &mut pre_res).await? {
                            MiddlewareResult::Continue => {}
                            MiddlewareResult::Break(res) => return Ok(res),
                            MiddlewareResult::Error(err) => return Err(err),
                        }
                    }
                    #[cfg(feature = "cookie")]
                    {
                        *pre_res.cookies_mut() = req.cookies().clone();
                    }
                    #[cfg(feature = "session")]
                    let session = req.extensions().get::<Session>().cloned();
                    #[cfg(feature = "session")]
                    if let Some(session) = session {
                        pre_res.extensions.insert(session);
                    }
                    pre_res.copy_from_response(handler.call(req).await?);
                    active_middlewares.reverse();
                    for middleware in active_middlewares {
                        match middleware.after_response(&mut pre_res).await? {
                            MiddlewareResult::Continue => {}
                            MiddlewareResult::Break(res) => return Ok(res),
                            MiddlewareResult::Error(err) => return Err(err),
                        }
                    }
                }
            },
            RouteMatched::Unmatched => {
                return Err(SilentError::business_error(
                    StatusCode::NOT_FOUND,
                    "Server not found".to_string(),
                ))
            }
        };
        root_middlewares.reverse();
        for middleware in root_middlewares {
            match middleware.after_response(&mut pre_res).await? {
                MiddlewareResult::Continue => {}
                MiddlewareResult::Break(res) => return Ok(res),
                MiddlewareResult::Error(err) => return Err(err),
            }
        }
        Ok(pre_res)
    }
    pub async fn handle(&self, req: Request, peer_addr: SocketAddr) -> Response {
        tracing::debug!("{:?}", req);
        let exception_handler = self.exception_handler.clone();
        let (mut req, path) = req.split_url();
        let configs = self.configs.clone().unwrap_or_default();
        req.configs = configs.clone();
        if req.headers().get("x-real-ip").is_none() {
            req.headers_mut()
                .insert("x-real-ip", peer_addr.ip().to_string().parse().unwrap());
        }
        let method = req.method().clone();
        let url = req.uri().to_string().clone();
        let http_version = req.version();
        let start_time = Utc::now().time();
        #[cfg(feature = "scheduler")]
        req.extensions_mut().insert(self.scheduler.clone());
        let res = self.handle_request(req, path).await;
        let end_time = Utc::now().time();
        let req_time = end_time - start_time;
        match res {
            Ok(res) => {
                tracing::info!(
                    "{} {} {} {:?} {} {:?} {}",
                    peer_addr,
                    method,
                    url,
                    http_version,
                    res.status_code.as_u16(),
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
                    e.status_code().as_u16(),
                    0,
                    req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0,
                    e.to_string()
                );
                match exception_handler {
                    Some(handler) => handler.call(e, configs.clone()).await,
                    None => e.into(),
                }
            }
        }
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
