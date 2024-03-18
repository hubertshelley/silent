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
use silent::{Request, Response, Result as SilentResult, SilentError, StatusCode};
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

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

pub(crate) fn init_model(args: Args) -> Result<WhisperModel> {
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

fn handle(
    req: CreateTranscriptionRequest,
    whisper_model: WhisperModel,
) -> Result<CreateTranscriptionResponse> {
    let input = req.file.path().clone();

    let pcm_data = pcm_decode(input);
    let config = whisper_model.config.clone();
    let mel_filters = whisper_model.mel_filters.clone();
    let mel = audio::pcm_to_mel(&config, &pcm_data, &mel_filters);
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
    println!("loaded mel: {:?}", mel.dims());
    let mut model = whisper_model.model.clone();
    let tokenizer = whisper_model.tokenizer.clone();
    let device = whisper_model.device.clone();
    let language_token = match (req.model.is_multilingual(), req.language) {
        (true, None) => Some(multilingual::detect_language(&mut model, &tokenizer, &mel)?),
        (false, None) => None,
        (true, Some(language)) => match token_id(&tokenizer, &format!("<|{language}|>")) {
            Ok(token_id) => Some(token_id),
            Err(_) => anyhow::bail!("language {language} is not supported"),
        },
        (false, Some(_)) => {
            anyhow::bail!("a language cannot be set for non-multilingual models")
        }
    };
    println!("matched language: {:?}", language_token);
    let mut dc = Decoder::new(
        model,
        tokenizer,
        299792458,
        &device,
        language_token,
        None,
        req.response_format.has_timestamps(),
        req.response_format.is_verbose(),
        req.temperature,
    )?;
    let segments = dc.run(&mel)?;
    Ok(CreateTranscriptionResponse::new(
        segments,
        req.response_format.clone(),
    ))
}

pub(crate) async fn create_transcription(mut req: Request) -> SilentResult<Response> {
    let whisper_model = req
        .configs()
        .get::<Arc<Mutex<WhisperModel>>>()
        .unwrap()
        .lock()
        .await
        .clone();
    let req = req.form_data().await?.try_into().map_err(|e| {
        SilentError::business_error(
            StatusCode::BAD_REQUEST,
            format!("failed to parse request: {}", e),
        )
    })?;
    let res = handle(req, whisper_model).map_err(|e| {
        SilentError::business_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create transcription: {}", e),
        )
    })?;
    Ok(res.into())
}
