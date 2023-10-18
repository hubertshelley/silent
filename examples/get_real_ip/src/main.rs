use silent::prelude::*;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|req| async move { Ok(req.remote().to_string()) });
    Server::new()
        .bind("0.0.0.0:8000".parse().unwrap())
        .run(route);
}
