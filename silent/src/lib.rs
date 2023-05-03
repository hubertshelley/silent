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
pub(crate) use handler::{Handler, HandlerWrapper};
pub use hyper::Method;

pub mod prelude {
    pub use crate::core::{path_param::PathParam, request::Request, response::Response};
    pub use crate::error::SilentError;
    pub use crate::log::{logger, Level};
    pub use crate::route::handler_append::HandlerAppend;
    pub use crate::route::Route;
    pub use crate::service::Server;
    pub use hyper::Method;
}

/// The main entry point for the library.
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
