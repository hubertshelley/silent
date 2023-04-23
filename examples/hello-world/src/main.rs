use silent::{Route, Server};

fn main() {
    let route = Route {
        path: "".to_string(),
        handler: None,
        children: vec![],
        middlewares: vec![],
    };
    Server::new().bind_route(route).run();
}
