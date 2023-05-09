#[cfg(feature = "server")]
mod conn;
/// The `silent` library.
#[warn(missing_docs)]
mod core;
mod error;
mod handler;
mod log;
mod middleware;
mod route;
#[cfg(feature = "server")]
mod rt;
#[cfg(feature = "server")]
mod service;

pub use crate::core::{request::Request, response::Response};
pub use crate::middleware::MiddleWareHandler;
pub use error::SilentError;
pub use error::SilentResult as Result;
pub use handler::Handler;
pub(crate) use handler::HandlerWrapper;
pub use hyper::{header, Method, StatusCode};

pub mod prelude {
    pub use crate::core::{
        path_param::PathParam, request::Request, res_body::full, response::Response,
    };
    pub use crate::error::{SilentError, SilentResult as Result};
    #[cfg(feature = "static")]
    pub use crate::handler::static_handler;
    pub use crate::handler::Handler;
    pub use crate::log::{logger, Level};
    pub use crate::middleware::MiddleWareHandler;
    pub use crate::route::handler_append::{HandlerAppend, HandlerGetter, HtmlHandlerAppend};
    pub use crate::route::Route;
    #[cfg(feature = "server")]
    pub use crate::service::Server;
    pub use hyper::{header, Method, StatusCode};
}
