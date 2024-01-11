mod handlers;
mod pcm_decode;
mod types;

use silent::prelude::*;

mod args;
mod device;
mod model;
mod multilingual;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|_req| async { Ok("hello world") });
    Server::new().run(route);
}
