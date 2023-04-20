// use crate::handler::Handler;
#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub handler: Option<String>,
    pub children: Vec<Route>,
    pub middlewares: Vec<String>,
}

impl Route {}
