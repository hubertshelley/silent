use crate::handler::Handler;

pub struct Route {
    pub path: String,
    pub handler: Option<Handler>,
    pub children: Vec<Route>,
    pub middlewares: Vec<String>,
}

impl Route{

}
