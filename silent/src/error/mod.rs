use std::io;
use thiserror::Error;

/// SilentError is the error type for the `silent` library.
#[derive(Error, Debug)]
pub enum SilentError {
    /// IO 错误
    #[error("io error")]
    IOError(#[from] io::Error),
    /// IO 错误
    #[error("serde_json error")]
    SerdeJsonError(#[from] serde_json::Error),
    /// Hyper 错误
    #[error("the data for key `{0}` is not available")]
    HyperError(#[from] hyper::Error),
}
