/// The `silent` library.
#[cfg(feature = "server")]
mod conn;
#[warn(missing_docs)]
mod core;
mod error;
mod handler;
mod log;
pub mod middleware;
mod route;
#[cfg(feature = "server")]
mod rt;
#[cfg(feature = "security")]
mod security;
#[cfg(feature = "server")]
mod service;
#[cfg(feature = "session")]
mod session;
#[cfg(feature = "sse")]
mod sse;
#[cfg(feature = "template")]
mod templates;
#[cfg(feature = "ws")]
mod ws;

pub use crate::core::{configs::Configs, request::Request, response::Response};
pub use crate::middleware::{middlewares, MiddleWareHandler};
pub use error::SilentError;
pub use error::SilentResult as Result;
pub use handler::Handler;
pub use handler::HandlerWrapper;
pub use headers::*;
pub use hyper::{header, Method, StatusCode};

pub mod prelude {
    pub use crate::core::{
        configs::Configs, path_param::PathParam, request::Request, res_body::full,
        res_body::stream_body, response::Response,
    };
    pub use crate::error::{SilentError, SilentResult as Result};
    #[cfg(feature = "static")]
    pub use crate::handler::static_handler;
    pub use crate::handler::Handler;
    pub use crate::handler::HandlerWrapper;
    pub use crate::log::*;
    pub use crate::middleware::MiddleWareHandler;
    #[cfg(feature = "ws")]
    pub use crate::route::handler_append::WSHandlerAppend;
    pub use crate::route::handler_append::{HandlerAppend, HandlerGetter};
    pub use crate::route::{Route, RouteService};
    #[cfg(feature = "security")]
    pub use crate::security::{argon2, pbkdf2};
    #[cfg(feature = "server")]
    pub use crate::service::Server;
    #[cfg(feature = "sse")]
    pub use crate::sse::{sse_reply, SSEEvent};
    #[cfg(feature = "template")]
    pub use crate::templates::*;
    #[cfg(feature = "ws")]
    pub use crate::ws::{
        FnOnClose, FnOnConnect, FnOnNoneResultFut, FnOnReceive, FnOnSend, FnOnSendFut,
    };
    #[cfg(feature = "ws")]
    pub use crate::ws::{Message, WebSocket, WebSocketHandler, WebSocketParts};
    #[cfg(feature = "session")]
    pub use async_session::{Session, SessionStore};
    #[cfg(feature = "cookie")]
    pub use cookie::{time as CookieTime, Cookie};
    pub use headers::*;
    pub use hyper::{header, upgrade, Method, StatusCode};
}
