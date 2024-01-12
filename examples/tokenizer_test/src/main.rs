use anyhow::{Error as E, Result};
use std::path::PathBuf;
use tokenizers::Tokenizer;
use tracing::Level;

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let tokenizer_filename =
        PathBuf::from("/Users/hubertshelley/Documents/whisper-large-v3/tokenizer.json".to_string());
    let tokenizer = Tokenizer::from_file(tokenizer_filename)
        .map_err(E::msg)
        .expect("tokenizer error");
}
