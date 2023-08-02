use silent::prelude::*;

struct Temp {
    name: String,
}

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|_req| async {
        let temp = Temp {
            name: "world".to_string(),
        };
        Ok(temp)
    });
    Server::new().bind_route(route).run();
}
