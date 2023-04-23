// use crate::handler::Handler;
#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub handler: Option<String>,
    pub children: Vec<Route>,
    pub middlewares: Vec<String>,
}

impl Route {}

#[derive(Debug, Clone)]
pub struct Routes {
    pub children: Vec<Route>,
}

impl Routes {
    pub fn new() -> Self {
        Self { children: vec![] }
    }

    pub fn add(&mut self, route: Route) {
        self.children.push(route);
    }
}
