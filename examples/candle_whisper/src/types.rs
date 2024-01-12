use crate::decoder::Segment;
use crate::model::WhichModel;
use serde::Deserialize;
use serde_json::json;
use silent::prelude::{FilePart, FormData};
use silent::{Response, SilentError, StatusCode};

#[derive(Debug, Deserialize, Clone)]
pub(crate) enum ResponseFormat {
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

#[derive(Debug, Clone)]
pub struct CreateTranscriptionRequest {
    // The audio file object (not file name) to transcribe, in one of these formats: wav.
    pub(crate) file: FilePart,
    // ID of the model to use. Only whisper-large-v3 is currently available.
    pub(crate) model: WhichModel,
    // The language of the input audio. Supplying the input language in ISO-639-1 format will improve accuracy and latency.
    pub(crate) language: Option<String>,
    // An optional text to guide the model's style or continue a previous audio segment. The prompt should match the audio language.
    pub(crate) prompt: Option<String>,
    // The format of the transcript output, in one of these options: json, text, srt, verbose_json, or vtt.
    pub response_format: ResponseFormat,
    // The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use log probability to automatically increase the temperature until certain thresholds are hit.
    pub(crate) temperature: f64,
}

impl TryFrom<&FormData> for CreateTranscriptionRequest {
    type Error = SilentError;

    fn try_from(value: &FormData) -> Result<Self, Self::Error> {
        let file = value
            .files
            .get("file")
            .cloned()
            .ok_or(SilentError::business_error(
                StatusCode::BAD_REQUEST,
                "file is required".to_string(),
            ))?;
        let model = serde_json::from_str(&value.fields.get("model").cloned().ok_or(
            SilentError::business_error(StatusCode::BAD_REQUEST, "model is required".to_string()),
        )?)?;
        let response_format = serde_json::from_str(
            &value
                .fields
                .get("response_format")
                .cloned()
                .unwrap_or("json".to_string()),
        )?;
        Ok(Self {
            file,
            model,
            language: value.fields.get("language").cloned(),
            prompt: value.fields.get("prompt").cloned(),
            response_format,
            temperature: value
                .fields
                .get("temperature")
                .cloned()
                .unwrap_or("0".to_string())
                .parse()
                .map_err(|_| {
                    SilentError::business_error(
                        StatusCode::BAD_REQUEST,
                        "temperature must be a float".to_string(),
                    )
                })?,
        })
    }
}
pub struct CreateTranscriptionResponse {
    segments: Vec<Segment>,
    format: ResponseFormat,
}

impl CreateTranscriptionResponse {
    pub fn new(segments: Vec<Segment>, format: ResponseFormat) -> Self {
        Self { segments, format }
    }
}

impl From<CreateTranscriptionResponse> for Response {
    fn from(value: CreateTranscriptionResponse) -> Self {
        match value.format {
            ResponseFormat::Json => json!({
                "text": value.segments.iter().map(|s| s.text()).collect::<Vec<_>>(),
            })
            .into(),
            ResponseFormat::Text => value
                .segments
                .iter()
                .map(|s| s.text())
                .collect::<Vec<_>>()
                .into(),
            ResponseFormat::Srt => !unimplemented!("Srt"),
            ResponseFormat::VerboseJson => json!({
                "text": value.segments.iter().map(|s| s.text()).collect::<Vec<_>>(),
            })
            .into(),
            ResponseFormat::Vtt => !unimplemented!("Vtt"),
        }
    }
}
