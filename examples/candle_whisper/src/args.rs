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

    /// The input to be processed, in wav format, will default to `jfk.wav`. Alternatively
    /// this can be set to sample:jfk, sample:gb1, ... to fetch a sample from the following
    /// repo: https://huggingface.co/datasets/Narsil/candle_demo/
    #[arg(long)]
    pub(crate) input: String,

    /// The seed to use when generating random samples.
    #[arg(long, default_value_t = 299792458)]
    pub(crate) seed: u64,

    /// Enable tracing (generates a trace-timestamp.json file).
    #[arg(long)]
    tracing: bool,

    #[arg(long)]
    quantized: bool,

    /// Language.
    #[arg(long)]
    pub(crate) language: Option<String>,

    /// Task, when no task is specified, the input tokens contain only the sot token which can
    /// improve things when in no-timestamp mode.
    #[arg(long)]
    pub(crate) task: Option<Task>,

    /// Timestamps mode, this is not fully implemented yet.
    #[arg(long)]
    pub(crate) timestamps: bool,

    /// Print the full DecodingResult structure rather than just the text.
    #[arg(long)]
    pub(crate) verbose: bool,
    #[arg(long, default_value_t = 0.0)]
    pub(crate) temperature: f64,
}
