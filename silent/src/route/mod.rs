use crate::core::request::Request;
use crate::core::response::Response;
use crate::route::handler_match::{Match, Matched};
use hyper::StatusCode;

mod handler_match;

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub handler: Option<String>,
    pub children: Vec<Route>,
    pub middlewares: Vec<String>,
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

#[derive(Debug, Clone)]
pub struct Routes {
    pub children: Vec<Route>,
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
                println!("{:?}", route.middlewares);
                Ok(Response::from(route.handler.unwrap()))
            }
            Matched::Unmatched => Err((String::from("404"), StatusCode::NOT_FOUND)),
        }
    }
}
