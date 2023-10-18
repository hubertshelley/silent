use crate::error::{ExceptionHandler, ExceptionHandlerWrapper};
use crate::route::handler_match::{Match, RouteMatched};
use crate::route::Route;
#[cfg(feature = "session")]
use crate::session::SessionMiddleware;
#[cfg(feature = "template")]
use crate::templates::TemplateMiddleware;
use crate::{MiddleWareHandler, Request, Response, SilentError, StatusCode};
#[cfg(feature = "session")]
use async_session::{Session, SessionStore};
use chrono::Utc;
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct RootRoute {
    pub(crate) children: Vec<Route>,
    pub(crate) middlewares: Vec<Arc<dyn MiddleWareHandler>>,
    pub(crate) exception_handler: Option<Arc<dyn ExceptionHandler>>,
    #[cfg(feature = "session")]
    pub(crate) session_set: bool,
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
        }
    }

    pub fn push(&mut self, mut route: Route) {
        self.middlewares
            .iter()
            .cloned()
            .for_each(|m| route.middleware_hook(m.clone()));
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
        F: Fn(SilentError) -> Fut + Send + Sync + 'static,
        T: Into<Response>,
    {
        self.exception_handler = Some(ExceptionHandlerWrapper::new(handler).arc());
        self
    }
}

impl RootRoute {
    pub async fn handle(
        &self,
        req: Request,
        peer_addr: SocketAddr,
    ) -> Result<Response, SilentError> {
        tracing::debug!("{:?}", req);
        let exception_handler = self.exception_handler.clone();
        let (mut req, path) = req.split_url();
        if req.headers().get("x-real-ip").is_none() {
            req.headers_mut()
                .insert("x-real-ip", peer_addr.ip().to_string().parse().unwrap());
        }
        let method = req.method().clone();
        let url = req.uri().to_string().clone();
        let http_version = req.version();
        let start_time = Utc::now().time();
        let res = match self.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(route) => match route.handler.get(req.method()) {
                None => Err(SilentError::business_error(
                    StatusCode::METHOD_NOT_ALLOWED,
                    "method not allowed".to_string(),
                )),
                Some(handler) => {
                    let mut pre_res = Response::empty();
                    let mut active_middlewares = vec![];
                    for (i, middleware) in route.middlewares.iter().enumerate() {
                        if middleware.match_req(&req).await {
                            active_middlewares.push(i);
                        }
                    }
                    for i in active_middlewares.clone() {
                        route.middlewares[i]
                            .pre_request(&mut req, &mut pre_res)
                            .await?
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
                    match handler.call(req).await {
                        Ok(res) => {
                            pre_res.from_response(res);
                            active_middlewares.reverse();
                            for i in active_middlewares {
                                route.middlewares[i].after_response(&mut pre_res).await?
                            }
                            Ok(pre_res)
                        }
                        Err(e) => Err(e),
                    }
                }
            },
            RouteMatched::Unmatched => Err(SilentError::business_error(
                StatusCode::NOT_FOUND,
                "Server not found".to_string(),
            )),
        };
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
                Ok(res)
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
                    Some(handler) => Ok(handler.call(e).await),
                    None => Err(e),
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
