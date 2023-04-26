use crate::core::request::Request;
use crate::core::response::Response;
use crate::handler::Handler;
use crate::route::handler_match::{Match, Matched};
use hyper::StatusCode;
use std::fmt::Display;
use std::sync::Arc;

mod handler_match;

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub handler: Option<Arc<dyn Handler>>,
    pub children: Vec<Route>,
    pub middlewares: Vec<Arc<dyn Handler>>,
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

impl Match for Route {
    fn handler_match(&self, path: &str) -> Matched {
        let (local_url, last_url) = Self::path_split(path);
        if self.path == local_url {
            if last_url.is_empty() {
                return Matched::Matched(self.clone());
            } else {
                for route in &self.children {
                    if let Matched::Matched(route) = route.handler_match(last_url) {
                        return Matched::Matched(route);
                    }
                }
            }
        }
        Matched::Unmatched
    }
}

impl Route {
    fn path_split(path: &str) -> (&str, &str) {
        let mut iter = path.splitn(2, '/');
        let local_url = iter.next().unwrap_or("");
        let last_url = iter.next().unwrap_or("");
        (local_url, last_url)
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

impl Match for Routes {
    fn handler_match(&self, path: &str) -> Matched {
        for route in &self.children {
            if let Matched::Matched(route) = route.handler_match(path) {
                return Matched::Matched(route);
            }
        }
        Matched::Unmatched
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
        println!("{:?}", req);
        match self.handler_match(req.uri().path()) {
            Matched::Matched(route) => {
                if route.handler.is_none() {
                    return Err((String::from("404"), StatusCode::NOT_FOUND));
                }
                match route.handler.unwrap().call(req).await {
                    Ok(res) => Ok(res),
                    Err(e) => Err((e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)),
                }
            }
            Matched::Unmatched => Err((String::from("404"), StatusCode::NOT_FOUND)),
        }
    }
}
