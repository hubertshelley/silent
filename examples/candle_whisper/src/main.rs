mod handlers;
mod pcm_decode;
mod types;

use crate::args::Args;
use crate::handlers::{create_transcription, init_model};
use clap::Parser;
use silent::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

mod args;
mod decoder;
mod device;
mod model;
mod multilingual;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let args = Args::parse();
    let mut configs = Configs::default();
    let whisper_model = init_model(args.clone()).expect("failed to initialize model");
    configs.insert(Arc::new(Mutex::new(whisper_model)));
    let route = Route::new("/v1/audio/transcriptions").post(create_transcription);
    Server::new()
        .with_configs(configs)
        .bind("0.0.0.0:8000".parse().unwrap())
        .run(route);
}
