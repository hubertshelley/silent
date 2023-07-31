use silent::prelude::*;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|_req| async { Response::redirect("https://www.baidu.com") });
    Server::new().bind_route(route).run();
}
