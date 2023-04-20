use std::io;
use thiserror::Error;

/// SilentError is the error type for the `silent` library.
#[derive(Error, Debug)]
pub enum SilentError {
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    HyperError(#[from] hyper::Error),
}
