/// Handler module
mod handler_trait;
mod handler_wrapper;
#[cfg(feature = "static")]
mod handler_wrapper_static;

pub use handler_trait::Handler;
pub use handler_wrapper::HandlerWrapper;
#[cfg(feature = "static")]
pub use handler_wrapper_static::static_handler;
