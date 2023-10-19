use silent::middlewares::{Cors, CorsType};
use silent::prelude::*;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("")
        .hook(
            Cors::new()
                .origin(CorsType::Any)
                .methods(vec!["POST"])
                .headers(CorsType::Any)
                .credentials(true),
        )
        .get(|_req| async { Ok("hello world") })
        .post(|_req| async { Ok("hello world") })
        .put(|_req| async { Ok("hello world") })
        .patch(|_req| async { Ok("hello world") })
        .options(|_req| async { Ok("hello world") })
        .delete(|_req| async { Ok("hello world") });
    Server::new().run(route);
}
