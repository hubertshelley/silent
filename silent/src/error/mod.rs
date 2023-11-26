mod exception_handler_trait;
mod exception_handler_wrapper;

use crate::headers::ContentType;
use crate::{Response, StatusCode};
pub(crate) use exception_handler_trait::ExceptionHandler;
pub(crate) use exception_handler_wrapper::ExceptionHandlerWrapper;
use serde::Serialize;
use serde_json::Value;
use std::backtrace::Backtrace;
use std::io;
use thiserror::Error;

/// BoxedError
pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// SilentError is the error type for the `silent` library.
#[derive(Error, Debug)]
pub enum SilentError {
    /// IO 错误
    #[error("io error")]
    IOError(#[from] io::Error),
    /// IO 错误
    #[error("io error")]
    TungsteniteError(#[from] crate::tokio_tungstenite::tungstenite::Error),
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
    FileEmpty(#[from] crate::multer::Error),
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
    /// 配置不存在 错误
    #[error("config not found")]
    ConfigNotFound,
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
    pub fn business_error_obj<S>(code: StatusCode, msg: S) -> Self
    where
        S: Serialize,
    {
        let msg = serde_json::to_string(&msg).unwrap_or_default();
        Self::BusinessError { code, msg }
    }
    pub fn business_error(code: StatusCode, msg: String) -> Self {
        Self::BusinessError { code, msg }
    }
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BusinessError { code, .. } => *code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn message(&self) -> String {
        match self {
            Self::BusinessError { msg, .. } => msg.clone(),
            _ => self.to_string(),
        }
    }
    pub fn trace(&self) -> Backtrace {
        Backtrace::capture()
    }
}

impl From<SilentError> for Response {
    fn from(value: SilentError) -> Self {
        let mut res = Response::empty();
        res.set_status(value.status_code());
        if serde_json::from_str::<Value>(&value.message()).is_ok() {
            res.set_typed_header(ContentType::json());
        }
        res.set_body(value.message().into());
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;
    use http_body_util::BodyExt;
    use hyper::StatusCode;
    use serde_json::Value;

    #[derive(Serialize)]
    struct ResBody {
        code: u16,
        msg: String,
        data: Value,
    }

    #[tokio::test]
    async fn test_silent_error() {
        let res_body = ResBody {
            code: 400,
            msg: "bad request".to_string(),
            data: Value::Null,
        };
        let err = SilentError::business_error_obj(StatusCode::BAD_REQUEST, res_body);
        let mut res: Response = err.into();
        assert_eq!(res.status_code, StatusCode::BAD_REQUEST);
        println!("{:#?}", res.headers);
        println!(
            "{:#?}",
            res.body.frame().await.unwrap().unwrap().data_ref().unwrap()
        );
    }
}
