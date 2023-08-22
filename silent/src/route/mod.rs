use crate::handler::Handler;
use crate::middleware::MiddleWareHandler;
use crate::Method;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub(crate) mod handler_append;
mod handler_match;
mod root;
mod route_service;

pub use root::RootRoute;

pub use route_service::RouteService;

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
    #[allow(dead_code)]
    pub(crate) fn middleware_hook_first(&mut self, handler: Arc<dyn MiddleWareHandler>) {
        self.middlewares.insert(0, handler.clone());
        self.children
            .iter_mut()
            .for_each(|r| r.middleware_hook_first(handler.clone()));
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
