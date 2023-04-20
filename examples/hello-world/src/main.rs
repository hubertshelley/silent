use silent::{Route, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let route = Route {
        path: "".to_string(),
        handler: None,
        children: vec![],
        middlewares: vec![],
    };
    Server::new().append(route).run().await;
    Ok(())
}
