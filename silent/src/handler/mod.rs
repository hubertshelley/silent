/// Handler module
mod handler_trait;
mod handler_wrapper;
mod handler_wrapper_html;
#[cfg(feature = "static")]
mod handler_wrapper_static;

pub use handler_trait::Handler;
pub(crate) use handler_wrapper::HandlerWrapper;
pub(crate) use handler_wrapper_html::HandlerWrapperHtml;
#[cfg(feature = "static")]
pub use handler_wrapper_static::static_handler;
