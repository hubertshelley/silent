// https://github.com/openai/whisper/blob/main/whisper/model.py/rgs
// TODO:
// - Batch size greater than 1.
// - More token filters (SuppressBlanks, ApplyTimestampRules).

#[cfg(feature = "accelerate")]
extern crate accelerate_src;

#[cfg(feature = "mkl")]
extern crate intel_mkl_src;

use crate::args::Args;
use crate::decoder::Decoder;
use crate::device::device;
use crate::model::Model;
use crate::multilingual;
use anyhow::{Error as E, Result};
use candle_core::{self as candle, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as m, audio, Config};
use clap::ValueEnum;
use silent::{Request, Response, Result as SilentResult, SilentError, StatusCode};
use std::path::PathBuf;
use tokenizers::Tokenizer;

use crate::pcm_decode::pcm_decode;
use crate::types::{CreateTranscriptionRequest, CreateTranscriptionResponse};

pub fn token_id(tokenizer: &Tokenizer, token: &str) -> candle::Result<u32> {
    match tokenizer.token_to_id(token) {
        None => candle::bail!("no token-id for {token}"),
        Some(id) => Ok(id),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WhisperModel {
    tokenizer: Tokenizer,
    model: Model,
    config: Config,
    mel_filters: Vec<f32>,
    device: candle::Device,
}
fn init_model(args: Args) -> Result<WhisperModel> {
    let device = device(args.cpu)?;
    let model_id = args.model_id;
    let (config_filename, tokenizer_filename, weights_filename) = {
        let config = PathBuf::from(format!("{model_id}/config.json"));
        let tokenizer = PathBuf::from(format!("{model_id}/tokenizer.json"));
        let model = PathBuf::from(format!("{model_id}/model.safetensors"));
        (config, tokenizer, model)
    };

    let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

    let mel_bytes = match config.num_mel_bins {
        80 => include_bytes!("melfilters.bytes").as_slice(),
        128 => include_bytes!("melfilters128.bytes").as_slice(),
        nmel => anyhow::bail!("unexpected num_mel_bins {nmel}"),
    };
    let mut mel_filters = vec![0f32; mel_bytes.len() / 4];
    <byteorder::LittleEndian as byteorder::ByteOrder>::read_f32_into(mel_bytes, &mut mel_filters);

    let model = {
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], m::DTYPE, &device)? };
        Model::Normal(m::model::Whisper::load(&vb, config.clone())?)
    };
    Ok(WhisperModel {
        tokenizer,
        model,
        config,
        mel_filters,
        device,
    })
}

fn handle(req: CreateTranscriptionRequest) -> CreateTranscriptionResponse {
    CreateTranscriptionResponse::new(vec![], req.response_format.clone())
}

async fn create_transcription(mut req: Request) -> SilentResult<Response> {
    let req = req.form_data().await?.try_into().map_err(|e| {
        SilentError::business_error(
            StatusCode::BAD_REQUEST,
            format!("failed to parse request: {}", e),
        )
    })?;
    let res = handle(req);
    Ok(res.into())
}
pub(crate) fn handle1(args: Args) -> Result<()> {
    let mut whisper_model = init_model(args.clone())?;
    let input = PathBuf::from(args.input);

    let pcm_data = pcm_decode(input);

    let mel = audio::pcm_to_mel(&whisper_model.config, &pcm_data, &whisper_model.mel_filters);
    let mel_len = mel.len();
    let mel = Tensor::from_vec(
        mel,
        (
            1,
            whisper_model.config.num_mel_bins,
            mel_len / whisper_model.config.num_mel_bins,
        ),
        &whisper_model.device,
    )?;

    let language_token = match (args.model.is_multilingual(), args.language) {
        (true, None) => Some(multilingual::detect_language(
            &mut whisper_model.model,
            &whisper_model.tokenizer,
            &mel,
        )?),
        (false, None) => None,
        (true, Some(language)) => {
            match token_id(&whisper_model.tokenizer, &format!("<|{language}|>")) {
                Ok(token_id) => Some(token_id),
                Err(_) => anyhow::bail!("language {language} is not supported"),
            }
        }
        (false, Some(_)) => {
            anyhow::bail!("a language cannot be set for non-multilingual models")
        }
    };
    let mut dc = Decoder::new(
        whisper_model.model.clone(),
        whisper_model.tokenizer.clone(),
        args.seed,
        &whisper_model.device,
        language_token,
        args.task,
        args.timestamps,
        args.verbose,
        args.temperature,
    )?;
    let s = dc.run(&mel)?;
    println!("done: {:?}", s);
    Ok(())
}
