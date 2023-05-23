/// Handler module
mod handler_trait;
mod handler_wrapper;
mod handler_wrapper_response;
#[cfg(feature = "static")]
mod handler_wrapper_static;

pub use handler_trait::Handler;
pub use handler_wrapper::HandlerWrapper;
pub use handler_wrapper_response::HandlerWrapperResponse;
#[cfg(feature = "static")]
pub use handler_wrapper_static::static_handler;
