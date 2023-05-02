use serde::{Deserialize, Serialize};
use silent::prelude::*;

async fn hello_world_1(_req: Request) -> Result<i32, SilentError> {
    Ok(1)
}

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
    logger::fmt().with_max_level(Level::DEBUG).init();
    let route = Route::new("1")
        .get(hello_world)
        .append(Route::new("11").get(hello_world_1))
        .append(Route::new("12").post(hello_world_2));
    Server::new()
        .bind("127.0.0.1:8001".parse().unwrap())
        .bind_route(route)
        .run();
}
