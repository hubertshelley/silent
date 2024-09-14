use silent::middlewares::Timeout;
use silent::prelude::*;
use std::time::Duration;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("")
        .hook(Timeout::new(Duration::from_secs(1)))
        .get(|_req| async {
            println!("sleeping for 2 seconds");
            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("done sleeping");
            Ok("hello world")
        });
    Server::new().run(route);
}
