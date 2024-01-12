mod handlers;
mod pcm_decode;
mod types;

use crate::args::Args;
use crate::handlers::handle1;
use clap::Parser;
use silent::prelude::*;

mod args;
mod decoder;
mod device;
mod model;
mod multilingual;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let args = Args::parse();
    let mut configs = Configs::default();
    handle1(args).unwrap();
    // let route = Route::new("").get(|_req| async { Ok("hello world") });
    // Server::new().run(route);
}
