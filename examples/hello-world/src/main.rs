use silent::{logger, Route, Server};

fn main() {
    logger::fmt::init();
    let route = Route {
        path: "".to_string(),
        handler: Some("hello world".to_string()),
        children: vec![Route {
            path: "1".to_string(),
            handler: Some("hello world1".to_string()),
            children: vec![
                Route {
                    path: "1".to_string(),
                    handler: Some("hello world11".to_string()),
                    children: vec![],
                    middlewares: vec![],
                },
                Route {
                    path: "2".to_string(),
                    handler: Some("hello world12".to_string()),
                    children: vec![],
                    middlewares: vec![],
                },
            ],
            middlewares: vec![],
        }],
        middlewares: vec![],
    };
    Server::new()
        .bind("127.0.0.1:8001".parse().unwrap())
        .bind_route(route)
        .run();
}
