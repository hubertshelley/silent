/// The `silent` library.
#[warn(missing_docs)]
mod error;
// mod service;
// mod route;
mod handler;

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
