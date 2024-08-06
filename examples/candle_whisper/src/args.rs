use crate::model::WhichModel;
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum Task {
    Transcribe,
    Translate,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Run on CPU rather than on GPU.
    #[arg(long)]
    pub(crate) cpu: bool,

    #[arg(long)]
    pub(crate) model_id: String,

    /// The model to use, check out available models:
    /// https://huggingface.co/models?search=whisper
    #[arg(long)]
    revision: Option<String>,

    /// The model to be used, can be tiny, small, medium.
    #[arg(long, default_value = "tiny.en")]
    pub(crate) model: WhichModel,

    /// The seed to use when generating random samples.
    #[arg(long, default_value_t = 299792458)]
    pub(crate) seed: u64,

    #[arg(long)]
    quantized: bool,
}
