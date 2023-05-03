/// The `silent` library.
#[warn(missing_docs)]
mod conn;
mod core;
mod error;
mod handler;
mod log;
mod route;
mod rt;
mod service;

pub use crate::core::{request::Request, response::Response};
pub use error::SilentError;
pub use handler::Handler;
pub(crate) use handler::HandlerWrapper;
pub use hyper::{header::HeaderName, header::HeaderValue, Method, StatusCode};

pub mod prelude {
    pub use crate::core::{path_param::PathParam, request::Request, response::Response};
    pub use crate::error::SilentError;
    pub use crate::handler::Handler;
    pub use crate::log::{logger, Level};
    pub use crate::route::handler_append::{HandlerAppend, HtmlHandlerAppend};
    pub use crate::route::Route;
    pub use crate::service::Server;
    pub use hyper::{header::HeaderName, header::HeaderValue, Method, StatusCode};
}
