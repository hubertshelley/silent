/// Handler module
mod handler_trait;
mod handler_wrapper;
mod handler_wrapper_html;
mod handler_wrapper_static;

pub use handler_trait::Handler;
pub(crate) use handler_wrapper::HandlerWrapper;
pub(crate) use handler_wrapper_html::HandlerWrapperHtml;
pub use handler_wrapper_static::static_handler;
