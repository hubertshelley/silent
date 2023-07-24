use crate::core::request::Request;
use crate::core::response::Response;
use crate::handler::Handler;
use crate::middleware::MiddleWareHandler;
use crate::route::handler_match::{Match, RouteMatched};
use crate::{header, Method, SilentError, StatusCode};
#[cfg(feature = "session")]
use async_session::Session;
use chrono::Utc;
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

pub(crate) mod handler_append;
mod handler_match;

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub handler: HashMap<Method, Arc<dyn Handler>>,
    pub children: Vec<Route>,
    pub middlewares: Vec<Arc<dyn MiddleWareHandler>>,
    special_match: bool,
    create_path: String,
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn get_route_str(pre_fix: String, route: &Route) -> String {
            let space_pre_fix = format!("    {}", pre_fix);
            let mut route_strs: Vec<String> = route
                .children
                .iter()
                .filter(|r| !r.handler.is_empty() || !r.children.is_empty())
                .map(|r| get_route_str(space_pre_fix.clone(), r))
                .collect();
            if !route.handler.is_empty() || !route.children.is_empty() {
                let methods: Vec<String> = route.handler.keys().map(|m| m.to_string()).collect();
                let methods_str = if methods.is_empty() {
                    "".to_string()
                } else {
                    format!("({})", methods.join(","))
                };
                route_strs.insert(0, format!("{}{}{}", pre_fix, route.path, methods_str));
            }
            route_strs.join("\n")
        }
        write!(f, "{}", get_route_str("".to_string(), self))
    }
}

impl Route {
    pub fn new(path: &str) -> Self {
        let path = path.trim_start_matches('/');
        let mut paths = path.splitn(2, '/');
        let first_path = paths.next().unwrap_or("");
        let last_path = paths.next().unwrap_or("");
        let route = Route {
            path: first_path.to_string(),
            handler: HashMap::new(),
            children: Vec::new(),
            middlewares: Vec::new(),
            special_match: first_path.starts_with('<') && first_path.ends_with('>'),
            create_path: path.to_string(),
        };
        if last_path.is_empty() {
            route
        } else {
            route.append(Route::new(last_path))
        }
    }
    pub fn append(mut self, mut route: Route) -> Self {
        self.middlewares
            .iter()
            .cloned()
            .for_each(|m| route.middleware_hook(m.clone()));
        self.children.push(route);
        self
    }
    pub fn hook(mut self, handler: impl MiddleWareHandler + 'static) -> Self {
        self.middleware_hook(Arc::new(handler));
        self
    }
    pub(crate) fn middleware_hook(&mut self, handler: Arc<dyn MiddleWareHandler>) {
        self.middlewares.push(handler.clone());
        self.children
            .iter_mut()
            .for_each(|r| r.middleware_hook(handler.clone()));
    }
}

#[derive(Clone, Default)]
pub struct Routes {
    pub children: Vec<Route>,
    middlewares: Vec<Arc<dyn MiddleWareHandler>>,
}

impl fmt::Debug for Routes {
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

impl Routes {
    pub fn new() -> Self {
        Self {
            children: vec![],
            middlewares: vec![],
        }
    }

    pub fn add(&mut self, mut route: Route) {
        self.middlewares
            .iter()
            .cloned()
            .for_each(|m| route.middleware_hook(m.clone()));
        self.children.push(route);
    }

    pub fn hook(&mut self, handler: impl MiddleWareHandler + 'static) {
        let handler = Arc::new(handler);
        self.children
            .iter_mut()
            .for_each(|r| r.middleware_hook(handler.clone()));
    }

    pub async fn handle(
        &self,
        req: Request,
        peer_addr: SocketAddr,
    ) -> Result<Response, SilentError> {
        tracing::debug!("{:?}", req);
        let (mut req, path) = req.split_url();
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
                    "{} \"{} {} {:?}\" {} {:?} {}",
                    peer_addr,
                    method,
                    url,
                    http_version,
                    res.status_code,
                    res.headers.get(header::CONTENT_LENGTH),
                    req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0
                );
                Ok(res)
            }
            Err(e) => {
                tracing::error!(
                    "{} \"{} {} {:?}\" {} {:?} {} {}",
                    peer_addr,
                    method,
                    url,
                    http_version,
                    e.status_code(),
                    0,
                    req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0,
                    e.to_string()
                );
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MiddlewareTest;

    impl MiddleWareHandler for MiddlewareTest {}

    #[test]
    fn middleware_tree_test() {
        let route = Route::new("api")
            .hook(MiddlewareTest {})
            .append(Route::new("test"));
        assert_eq!(route.children[0].middlewares.len(), 1)
    }
}
