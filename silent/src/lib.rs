/// The `silent` library.
#[warn(missing_docs)]
mod error;
mod handler;
mod route;
mod service;

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
