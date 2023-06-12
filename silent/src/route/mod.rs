use crate::core::request::Request;
use crate::core::response::Response;
use crate::handler::Handler;
use crate::middleware::MiddleWareHandler;
use crate::route::handler_match::{Match, RouteMatched};
use crate::{header, Method, SilentError, StatusCode};
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
        route.middlewares.append(&mut self.middlewares.clone());
        self.children.push(route);
        self
    }
    pub fn hook(mut self, handler: impl MiddleWareHandler + 'static) -> Self {
        self.middlewares.push(Arc::new(handler));
        self
    }
}

#[derive(Clone, Default)]
pub struct Routes {
    pub children: Vec<Route>,
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
        Self { children: vec![] }
    }

    pub fn add(&mut self, route: Route) {
        self.children.push(route);
    }

    pub async fn handle(
        &self,
        req: Request,
        peer_addr: SocketAddr,
    ) -> Result<Response, (String, StatusCode)> {
        tracing::debug!("{:?}", req);
        let (mut req, path) = req.split_url();
        let method = req.method().clone();
        let url = req.uri().to_string().clone();
        let http_version = req.version();
        let start_time = Utc::now().time();
        let res = match self.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(route) => match route.handler.get(req.method()) {
                None => Err((String::from("405"), StatusCode::METHOD_NOT_ALLOWED)),
                Some(handler) => {
                    let mut pre_res = Response::empty();
                    let mut active_middlewares = vec![];
                    for (i, middleware) in route.middlewares.iter().enumerate() {
                        if middleware.match_req(&req).await {
                            active_middlewares.push(i);
                        }
                    }
                    for i in active_middlewares.clone() {
                        if let Err(e) = route.middlewares[i]
                            .pre_request(&mut req, &mut pre_res)
                            .await
                        {
                            return error_result_handler(e);
                        }
                    }
                    match handler.call(req).await {
                        Ok(res) => {
                            for (header_key, header_value) in res.headers.clone().into_iter() {
                                if let Some(key) = header_key {
                                    pre_res = pre_res.set_header(key, header_value);
                                }
                            }
                            pre_res.status_code = res.status_code;
                            pre_res.set_body(res.body);
                            for i in active_middlewares {
                                if let Err(e) =
                                    route.middlewares[i].after_response(&mut pre_res).await
                                {
                                    return error_result_handler(e);
                                }
                            }
                            Ok(pre_res)
                        }
                        Err(e) => error_result_handler(e),
                    }
                }
            },
            RouteMatched::Unmatched => Err((String::from("404"), StatusCode::NOT_FOUND)),
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
            Err((msg, code)) => {
                tracing::error!(
                    "{} \"{} {} {:?}\" {} {:?} {} {}",
                    peer_addr,
                    method,
                    url,
                    http_version,
                    code,
                    0,
                    req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0,
                    msg
                );
                Err((msg, code))
            }
        }
    }
}

fn error_result_handler(e: SilentError) -> Result<Response, (String, StatusCode)> {
    let _ = e.trace();
    match e {
        SilentError::BusinessError { code, msg } => Err((msg, code)),
        _ => Err((e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)),
    }
}
