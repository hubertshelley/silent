use crate::StatusCode;
use std::backtrace::Backtrace;
use std::io;
use thiserror::Error;

/// SilentError is the error type for the `silent` library.
#[derive(Error, Debug)]
pub enum SilentError {
    /// IO 错误
    #[error("io error")]
    IOError(#[from] io::Error),
    /// IO 错误
    #[error("io error")]
    TungsteniteError(#[from] tokio_tungstenite::tungstenite::Error),
    /// 反序列化 错误
    #[error("serde_json error `{0}`")]
    SerdeJsonError(#[from] serde_json::Error),
    /// 反序列化 错误
    #[error("serde de error `{0}`")]
    SerdeDeError(#[from] serde::de::value::Error),
    /// Hyper 错误
    #[error("the data for key `{0}` is not available")]
    HyperError(#[from] hyper::Error),
    /// 上传文件读取 错误
    #[error("upload file read error `{0}`")]
    FileEmpty(#[from] multer::Error),
    /// Body为空 错误
    #[error("body is empty")]
    BodyEmpty,
    /// Json为空 错误
    #[error("json is empty")]
    JsonEmpty,
    /// Json为空 错误
    #[error("content-type is error")]
    ContentTypeError,
    /// Params为空 错误
    #[error("params is empty")]
    ParamsEmpty,
    /// Params为空 错误
    #[error("params not found")]
    ParamsNotFound,
    /// websocket错误
    #[error("websocket error: {0}")]
    WsError(String),
    /// 业务错误
    #[error("business error: {msg} ({code})")]
    BusinessError {
        /// 错误码
        code: StatusCode,
        /// 错误信息
        msg: String,
    },
}

pub type SilentResult<T> = Result<T, SilentError>;

impl SilentError {
    pub fn trace(&self) -> Backtrace {
        Backtrace::capture()
    }
}
