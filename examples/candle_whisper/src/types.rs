use silent::prelude::{FilePart, FormData};
use silent::{SilentError, StatusCode};

#[derive(Debug)]
pub(crate) struct CreateTranscriptionsRequest {
    // The audio file object (not file name) to transcribe, in one of these formats: wav.
    pub(crate) file: FilePart,
    // ID of the model to use. Only whisper-large-v3 is currently available.
    pub(crate) model: String,
    // The language of the input audio. Supplying the input language in ISO-639-1 format will improve accuracy and latency.
    pub(crate) language: Option<String>,
    // An optional text to guide the model's style or continue a previous audio segment. The prompt should match the audio language.
    pub(crate) prompt: Option<String>,
    // The format of the transcript output, in one of these options: json, text, srt, verbose_json, or vtt.
    pub(crate) response_format: String,
    // The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use log probability to automatically increase the temperature until certain thresholds are hit.
    pub(crate) temperature: f32,
}

impl TryFrom<FormData> for CreateTranscriptionsRequest {
    type Error = SilentError;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let file = value
            .files
            .get("file")
            .cloned()
            .ok_or(SilentError::business_error(
                StatusCode::BAD_REQUEST,
                "file is required".to_string(),
            ))?;
        Ok(Self {
            file,
            model: value
                .fields
                .get("model")
                .cloned()
                .ok_or(SilentError::business_error(
                    StatusCode::BAD_REQUEST,
                    "model is required".to_string(),
                ))?,
            language: value.fields.get("language").cloned(),
            prompt: value.fields.get("prompt").cloned(),
            response_format: value
                .fields
                .get("response_format")
                .cloned()
                .unwrap_or("json".to_string()),
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
