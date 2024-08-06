mod configs;
/// The `silent` library.
#[warn(missing_docs)]
mod core;
mod error;
#[cfg(feature = "grpc")]
mod grpc;
mod handler;
mod log;
pub mod middleware;
mod route;
#[cfg(feature = "scheduler")]
mod scheduler;
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
#[cfg(feature = "upgrade")]
mod ws;

// use silent_multer as multer;
#[cfg(feature = "multipart")]
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use multer;

pub use crate::configs::Configs;
pub use crate::core::{request::Request, response::Response};
pub use crate::middleware::{middlewares, MiddleWareHandler, MiddlewareResult};
pub use error::SilentError;
pub use error::SilentResult as Result;
pub use handler::Handler;
pub use handler::HandlerWrapper;
pub use headers;
pub use hyper::{header, Method, StatusCode};
#[cfg(feature = "scheduler")]
pub use scheduler::{ProcessTime, Scheduler, Task};

pub mod prelude {
    pub use crate::configs::Configs;
    #[cfg(feature = "multipart")]
    pub use crate::core::form::{FilePart, FormData};
    pub use crate::core::{
        path_param::PathParam, req_body::ReqBody, request::Request, res_body::full,
        res_body::stream_body, res_body::ResBody, response::Response,
    };
    pub use crate::error::{SilentError, SilentResult as Result};
    #[cfg(feature = "static")]
    pub use crate::handler::static_handler;
    pub use crate::handler::Handler;
    pub use crate::handler::HandlerWrapper;
    pub use crate::log::*;
    pub use crate::middleware::MiddleWareHandler;
    pub use crate::middleware::MiddlewareResult;
    #[cfg(feature = "upgrade")]
    pub use crate::route::handler_append::WSHandlerAppend;
    pub use crate::route::handler_append::{HandlerAppend, HandlerGetter};
    pub use crate::route::{RootRoute, Route, RouteService, RouterAdapt};
    #[cfg(feature = "scheduler")]
    pub use crate::scheduler::Task;
    #[cfg(feature = "security")]
    pub use crate::security::{argon2, pbkdf2};
    #[cfg(feature = "server")]
    pub use crate::service::Server;
    #[cfg(feature = "sse")]
    pub use crate::sse::{sse_reply, SSEEvent};
    #[cfg(feature = "template")]
    pub use crate::templates::*;
    #[cfg(feature = "upgrade")]
    pub use crate::ws::{
        FnOnClose, FnOnConnect, FnOnNoneResultFut, FnOnReceive, FnOnSend, FnOnSendFut,
    };
    #[cfg(feature = "upgrade")]
    pub use crate::ws::{Message, WebSocket, WebSocketHandler, WebSocketParts};
    #[cfg(feature = "session")]
    pub use async_session::{Session, SessionStore};
    #[cfg(feature = "cookie")]
    pub use cookie::{time as CookieTime, Cookie};
    pub use headers;
    pub use hyper::{header, upgrade, Method, StatusCode};
}
