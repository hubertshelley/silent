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
        if self.path == path {
            return Matched::Matched(self.clone());
        }
        for route in &self.children {
            if let Matched::Matched(route) = route.handler_match(path) {
                return Matched::Matched(route);
            }
        }
        Matched::Unmatched
    }
}

impl Route {}

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
                println!("{:?}", route.middlewares);
                Ok(Response::from("Hello World"))
            }
            Matched::Unmatched => Err((String::from("404"), StatusCode::NOT_FOUND)),
        }
    }
}
