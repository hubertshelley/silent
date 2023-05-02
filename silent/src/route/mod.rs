use crate::core::request::Request;
use crate::core::response::Response;
use crate::handler::Handler;
use crate::route::handler_match::{Match, RouteMatched};
use crate::Method;
use hyper::StatusCode;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

pub(crate) mod handler_append;
mod handler_match;

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub handler: HashMap<Method, Arc<dyn Handler>>,
    pub children: Vec<Route>,
    pub middlewares: Vec<Arc<dyn Handler>>,
    special_match: bool,
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut path = self.path.clone();
        if path.is_empty() {
            path = "/".to_string();
        }
        for route in &self.children {
            write!(f, "{}", route)?;
        }
        write!(f, "{}", path)
    }
}

impl Route {
    pub fn new(path: &str) -> Self {
        let mut paths = path.splitn(2, '/');
        let first_path = paths.next().unwrap_or("");
        let last_path = paths.next().unwrap_or("");
        let route = Route {
            path: first_path.to_string(),
            handler: HashMap::new(),
            children: Vec::new(),
            middlewares: Vec::new(),
            special_match: first_path.starts_with('<') && first_path.ends_with('>'),
        };
        if last_path.is_empty() {
            route
        } else {
            route.append(Route::new(last_path))
        }
    }
    pub fn append(mut self, route: Route) -> Self {
        self.children.push(route);
        self
    }
}

#[derive(Clone)]
pub struct Routes {
    pub children: Vec<Route>,
}

impl Display for Routes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self
            .children
            .iter()
            .map(|route| route.to_string())
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

    pub async fn handle(&self, req: Request) -> Result<Response, (String, StatusCode)> {
        tracing::debug!("{:?}", req);
        let (mut req, path) = req.split_url();
        match self.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(route) => match route.handler.get(req.method()) {
                None => Err((String::from("405"), StatusCode::METHOD_NOT_ALLOWED)),
                Some(handler) => {
                    let mut pre_res = Response::empty();
                    for middleware in &route.middlewares {
                        if let Err(e) = middleware.middleware_call(&mut req, &mut pre_res).await {
                            return Err((e.to_string(), StatusCode::INTERNAL_SERVER_ERROR));
                        }
                    }
                    tracing::debug!("pre_res: {:?}", pre_res);
                    match handler.call(req).await {
                        Ok(res) => Ok(pre_res.set_body(res.res.into_body())),
                        Err(e) => Err((e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)),
                    }
                }
            },
            RouteMatched::Unmatched => Err((String::from("404"), StatusCode::NOT_FOUND)),
        }
    }
}
