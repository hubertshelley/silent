use super::{Route, Routes};
use crate::Request;

pub(crate) enum Matched {
    Matched(Route),
    Unmatched,
}

pub(crate) trait Match {
    fn handler_match(&self, req: &Request, path: &str) -> Matched;
}

pub(crate) trait RouteMatch: Match {
    fn get_path(&self) -> &str;
    fn path_match(&self, req: &Request, path: &str) -> bool {
        let self_path = self.get_path();
        if self_path.starts_with('<') && self_path.ends_with('>') {
            let _ = req;
            // todo: 路由特殊匹配待编写
            return false;
        }
        self_path == path
    }
    fn path_split(path: &str) -> (&str, &str) {
        let mut iter = path.splitn(2, '/');
        let local_url = iter.next().unwrap_or("");
        let last_url = iter.next().unwrap_or("");
        (local_url, last_url)
    }
}

impl Match for Route {
    fn handler_match(&self, req: &Request, path: &str) -> Matched {
        let (local_url, last_url) = Self::path_split(path);
        println!("handler_match: path: {}", path);
        println!(
            "handler_match: local_url: {}, last_url: {}",
            local_url, last_url
        );
        if self.path_match(req, local_url) {
            if last_url.is_empty() {
                return Matched::Matched(self.clone());
            } else {
                for route in &self.children {
                    if let Matched::Matched(route) = route.handler_match(req, last_url) {
                        return Matched::Matched(route);
                    }
                }
            }
        }
        Matched::Unmatched
    }
}

impl RouteMatch for Route {
    fn get_path(&self) -> &str {
        self.path.as_str()
    }
}

impl Match for Routes {
    fn handler_match(&self, req: &Request, path: &str) -> Matched {
        tracing::debug!("path: {}", path);
        let mut path = path;
        // 去除路由开始的第一个斜杠
        if path.starts_with('/') {
            path = &path[1..];
        }
        for route in &self.children {
            if let Matched::Matched(route) = route.handler_match(req, path) {
                return Matched::Matched(route);
            }
        }
        Matched::Unmatched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::HandlerAppend;
    use crate::SilentError;

    async fn hello(_: Request) -> Result<String, SilentError> {
        Ok("hello".to_string())
    }

    fn get_matched(routes: &Routes, req: &Request) -> bool {
        match routes.handler_match(req, req.uri().path()) {
            Matched::Matched(_) => true,
            Matched::Unmatched => false,
        }
    }

    #[test]
    fn route_match_test() {
        let route = Route::new("hello").get(hello);
        let mut routes = Routes::new();
        routes.add(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/hello".parse().unwrap();
        assert!(get_matched(&routes, &req));
    }

    #[test]
    fn multi_route_match_test() {
        let route = Route::new("hello")
            .get(hello)
            .append(Route::new("world").get(hello));
        let mut routes = Routes::new();
        routes.add(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/hello/world".parse().unwrap();
        assert!(get_matched(&routes, &req));
    }

    #[test]
    fn multi_route_match_test_2() {
        let route = Route::new("")
            .get(hello)
            .append(Route::new("world").get(hello));
        let mut routes = Routes::new();
        routes.add(route);
        let mut req = Request::empty();
        *req.uri_mut() = "//world".parse().unwrap();
        assert!(get_matched(&routes, &req));
    }
}
