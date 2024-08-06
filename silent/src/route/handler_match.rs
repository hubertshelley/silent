use super::{RootRoute, Route};
use crate::core::path_param::PathParam;
use crate::Request;

pub(crate) enum RouteMatched {
    Matched(Route),
    Unmatched,
}

pub(crate) trait Match {
    fn handler_match(&self, req: &mut Request, path: &str) -> RouteMatched;
}

pub(crate) trait RouteMatch: Match {
    fn get_path(&self) -> &str;
    /// 最终匹配
    fn last_matched(&self, req: &mut Request, last_url: &str) -> RouteMatched;
    fn path_split(path: &str) -> (&str, &str) {
        let mut iter = path.splitn(2, '/');
        let local_url = iter.next().unwrap_or("");
        let last_url = iter.next().unwrap_or("");
        (local_url, last_url)
    }
}

enum SpecialPath {
    String(String),
    Int(String),
    I64(String),
    I32(String),
    U64(String),
    U32(String),
    UUid(String),
    Path(String),
    FullPath(String),
}

impl<'a> From<&'a str> for SpecialPath {
    fn from(value: &str) -> Self {
        // 去除首尾的尖括号
        let value = &value[1..value.len() - 1];
        let mut type_str = value.splitn(2, ':');
        let key = type_str.next().unwrap_or("");
        let path_type = type_str.next().unwrap_or("");
        println!("key: {}, path_type: {}", key, path_type);
        match path_type {
            "**" => SpecialPath::FullPath(key.to_string()),
            "*" => SpecialPath::Path(key.to_string()),
            "full_path" => SpecialPath::FullPath(key.to_string()),
            "path" => SpecialPath::Path(key.to_string()),
            "str" => SpecialPath::String(key.to_string()),
            "int" => SpecialPath::Int(key.to_string()),
            "i64" => SpecialPath::I64(key.to_string()),
            "i32" => SpecialPath::I32(key.to_string()),
            "u64" => SpecialPath::U64(key.to_string()),
            "u32" => SpecialPath::U32(key.to_string()),
            "uuid" => SpecialPath::UUid(key.to_string()),
            _ => SpecialPath::String(key.to_string()),
        }
    }
}

impl Match for Route {
    fn handler_match(&self, req: &mut Request, path: &str) -> RouteMatched {
        let (local_url, last_url) = if self.path.is_empty() {
            ("", path)
        } else {
            Self::path_split(path)
        };
        if !self.special_match {
            if self.path == local_url {
                self.last_matched(req, last_url)
            } else {
                RouteMatched::Unmatched
            }
        } else {
            match self.get_path().into() {
                SpecialPath::String(key) => match self.last_matched(req, last_url) {
                    RouteMatched::Matched(route) => {
                        req.set_path_params(key, local_url.to_string().into());
                        RouteMatched::Matched(route)
                    }
                    RouteMatched::Unmatched => RouteMatched::Unmatched,
                },
                SpecialPath::Int(key) => match local_url.parse::<i32>() {
                    Ok(value) => {
                        req.set_path_params(key, value.into());
                        self.last_matched(req, last_url)
                    }
                    Err(_) => RouteMatched::Unmatched,
                },
                SpecialPath::I64(key) => match local_url.parse::<i64>() {
                    Ok(value) => {
                        req.set_path_params(key, value.into());
                        self.last_matched(req, last_url)
                    }
                    Err(_) => RouteMatched::Unmatched,
                },
                SpecialPath::I32(key) => match local_url.parse::<i32>() {
                    Ok(value) => {
                        req.set_path_params(key, value.into());
                        self.last_matched(req, last_url)
                    }
                    Err(_) => RouteMatched::Unmatched,
                },
                SpecialPath::U64(key) => match local_url.parse::<u64>() {
                    Ok(value) => {
                        req.set_path_params(key, value.into());
                        self.last_matched(req, last_url)
                    }
                    Err(_) => RouteMatched::Unmatched,
                },
                SpecialPath::U32(key) => match local_url.parse::<u32>() {
                    Ok(value) => {
                        req.set_path_params(key, value.into());
                        self.last_matched(req, last_url)
                    }
                    Err(_) => RouteMatched::Unmatched,
                },
                SpecialPath::UUid(key) => match local_url.parse::<uuid::Uuid>() {
                    Ok(value) => {
                        req.set_path_params(key, value.into());
                        self.last_matched(req, last_url)
                    }
                    Err(_) => RouteMatched::Unmatched,
                },
                SpecialPath::Path(key) => {
                    req.set_path_params(key, PathParam::Path(local_url.to_string()));
                    self.last_matched(req, last_url)
                }
                SpecialPath::FullPath(key) => {
                    println!("SpecialPath::FullPath: path: {}", path);
                    req.set_path_params(key, PathParam::Path(path.to_string()));
                    match self.last_matched(req, last_url) {
                        RouteMatched::Matched(route) => {
                            println!("SpecialPath::FullPath: matched: {}", route.path);
                            RouteMatched::Matched(route)
                        }
                        RouteMatched::Unmatched => {
                            println!(
                                "SpecialPath::FullPath: Unmatched matched: {}",
                                self.handler.len()
                            );
                            match self.handler.is_empty() {
                                true => RouteMatched::Unmatched,
                                false => RouteMatched::Matched(self.clone()),
                            }
                        }
                    }
                }
            }
        }
    }
}

impl RouteMatch for Route {
    fn get_path(&self) -> &str {
        self.path.as_str()
    }

    fn last_matched(&self, req: &mut Request, last_url: &str) -> RouteMatched {
        if last_url.is_empty() && !self.handler.is_empty() {
            return RouteMatched::Matched(self.clone());
        } else {
            for route in &self.children {
                if let RouteMatched::Matched(route) = route.handler_match(req, last_url) {
                    return RouteMatched::Matched(route);
                }
            }
        }

        RouteMatched::Unmatched
    }
}

impl Match for RootRoute {
    fn handler_match(&self, req: &mut Request, path: &str) -> RouteMatched {
        tracing::debug!("path: {}", path);
        let mut path = path;
        // 去除路由开始的第一个斜杠
        if path.starts_with('/') {
            path = &path[1..];
        }
        for route in &self.children {
            if let RouteMatched::Matched(route) = route.handler_match(req, path) {
                return RouteMatched::Matched(route);
            }
        }
        RouteMatched::Unmatched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::HandlerAppend;
    use crate::SilentError;
    use bytes::Bytes;
    use http_body_util::BodyExt;

    async fn hello(_: Request) -> Result<String, SilentError> {
        Ok("hello".to_string())
    }

    async fn world<'a>(_: Request) -> Result<&'a str, SilentError> {
        Ok("world")
    }

    fn get_matched(routes: &RootRoute, req: Request) -> bool {
        let (mut req, path) = req.split_url();
        match routes.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(_) => true,
            RouteMatched::Unmatched => false,
        }
    }

    #[test]
    fn route_match_test() {
        let route = Route::new("hello").get(hello);
        let mut routes = RootRoute::new();
        routes.push(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/hello".parse().unwrap();
        assert!(get_matched(&routes, req));
    }

    #[test]
    fn multi_route_match_test() {
        let route = Route::new("hello/world").get(hello);
        let mut routes = RootRoute::new();
        routes.push(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/hello/world".parse().unwrap();
        assert!(get_matched(&routes, req));
    }

    #[test]
    fn multi_route_match_test_2() {
        let route = Route::new("")
            .get(hello)
            .append(Route::new("world").get(hello));
        let mut routes = RootRoute::new();
        routes.push(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/world".parse().unwrap();
        assert!(get_matched(&routes, req));
    }

    #[test]
    fn multi_route_match_test_3() {
        let route = Route::new("")
            .get(hello)
            .append(Route::new("<id:i64>").get(hello));
        let mut routes = RootRoute::new();
        routes.push(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/12345678909876543".parse().unwrap();
        let (mut req, path) = req.split_url();
        let matched = match routes.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(_) => {
                assert_eq!(
                    req.get_path_params::<i64>("id").unwrap(),
                    12345678909876543i64
                );
                true
            }
            RouteMatched::Unmatched => false,
        };
        assert!(matched)
    }

    #[test]
    fn special_route_match_test_2() {
        let route = Route::new("<path:**>")
            .get(hello)
            .append(Route::new("world").get(hello));
        let mut routes = RootRoute::new();
        routes.push(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/hello/world".parse().unwrap();
        let (mut req, path) = req.split_url();
        let matched = match routes.handler_match(&mut req, path.as_str()) {
            RouteMatched::Matched(_) => {
                assert_eq!(
                    req.get_path_params::<String>("path").unwrap(),
                    "hello/world".to_string()
                );
                true
            }
            RouteMatched::Unmatched => false,
        };
        assert!(matched)
    }

    #[tokio::test]
    async fn special_route_match_test_3() {
        let route = Route::new("<path:**>")
            .get(hello)
            .append(Route::new("world").get(world));
        let mut routes = RootRoute::new();
        routes.push(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/hello/world".parse().unwrap();
        assert_eq!(
            routes
                .handle(req, "127.0.0.1:8000".parse().unwrap())
                .await
                .body
                .frame()
                .await
                .unwrap()
                .unwrap()
                .data_ref()
                .unwrap(),
            &Bytes::from("world")
        );
    }

    #[tokio::test]
    async fn special_route_match_test_4() {
        let route = Route::new("<path:**>")
            .get(hello)
            .append(Route::new("world").get(world));
        let mut routes = RootRoute::new();
        routes.push(route);
        let mut req = Request::empty();
        *req.uri_mut() = "/hello/world1".parse().unwrap();
        assert_eq!(
            routes
                .handle(req, "127.0.0.1:8000".parse().unwrap())
                .await
                .body
                .frame()
                .await
                .unwrap()
                .unwrap()
                .data_ref()
                .unwrap(),
            &Bytes::from("hello")
        );
    }
}
