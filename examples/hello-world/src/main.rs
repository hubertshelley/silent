use silent::{logger, Route, Server};

fn main() {
    logger::fmt::init();
    let route = Route {
        path: "".to_string(),
        handler: None,
        children: vec![Route {
            path: "1".to_string(),
            handler: None,
            children: vec![],
            middlewares: vec![],
        }],
        middlewares: vec![],
    };
    Server::new().bind_route(route).run();
}
