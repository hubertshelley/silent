use silent::{Route, Server};

#[tokio::main]
async fn main() {
    let route = Route {
        path: "".to_string(),
        handler: None,
        children: vec![],
        middlewares: vec![],
    };
    Server::new().append(route).run().await;
}
