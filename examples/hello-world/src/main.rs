use serde::{Deserialize, Serialize};
use silent::{logger, HandlerWrapper, Method, Request, Route, Server, SilentError};
use std::sync::Arc;

async fn hello_world<'a>(_req: Request) -> Result<&'a str, SilentError> {
    Ok("Hello World")
}

#[derive(Debug, Serialize, Deserialize)]
struct HelloHandler {
    name: String,
}

async fn hello_world_2(_req: Request) -> Result<HelloHandler, SilentError> {
    Ok(HelloHandler {
        name: "Hello World".to_string(),
    })
}

fn main() {
    logger::fmt::init();
    let route = Route {
        path: "".to_string(),
        handler: Some(Arc::new(HandlerWrapper::new(
            Some(Method::GET),
            hello_world,
        ))),
        children: vec![Route {
            path: "".to_string(),
            handler: Some(Arc::new(HandlerWrapper::new(
                Some(Method::GET),
                hello_world_2,
            ))),
            children: vec![],
            middlewares: vec![],
        }],
        middlewares: vec![],
    };
    Server::new()
        .bind("127.0.0.1:8001".parse().unwrap())
        .bind_route(route)
        .run();
}
