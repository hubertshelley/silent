mod conn;
mod core;
/// The `silent` library.
#[warn(missing_docs)]
mod error;
mod handler;
mod log;
mod route;
mod rt;
mod service;

pub use crate::core::{request::Request, response::Response};
pub use error::SilentError;
pub use handler::Handler;
pub use log::logger;
pub use route::Route;
pub use service::Server;

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
