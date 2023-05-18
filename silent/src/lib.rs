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
#[cfg(feature = "ws")]
mod ws;

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
    pub use crate::log::*;
    pub use crate::middleware::MiddleWareHandler;
    #[cfg(feature = "ws")]
    pub use crate::route::handler_append::WSHandlerAppend;
    pub use crate::route::handler_append::{HandlerAppend, HandlerGetter};
    pub use crate::route::Route;
    #[cfg(feature = "server")]
    pub use crate::service::Server;
    #[cfg(feature = "ws")]
    pub use crate::ws::{
        FnOnClose, FnOnConnect, FnOnNoneResultFut, FnOnReceive, FnOnSend, FnOnSendFut,
    };
    #[cfg(feature = "ws")]
    pub use crate::ws::{HandlerWrapperWebSocket, Message, WebSocket, WebSocketParts};
    pub use hyper::{header, upgrade, Method, StatusCode};
}
