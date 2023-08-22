use crate::route::{RootRoute, Route};

pub trait RouteService {
    fn route(self) -> RootRoute;
}

impl RouteService for RootRoute {
    fn route(self) -> RootRoute {
        Self { ..self }
    }
}

impl RouteService for Route {
    fn route(self) -> RootRoute {
        let mut root = RootRoute::new();
        root.push(self);
        root
    }
}
