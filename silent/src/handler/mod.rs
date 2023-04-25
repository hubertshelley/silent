// use std::future::Future;
// use std::pin::Pin;
// use crate::core::request::Request;
// use crate::error::SilentError;
//
// pub struct Handler<T> {
//     pub handler: Pin<Box<dyn Future<Output=T> + 'static>>,
//     pub middlewares: Vec<String>,
// }
//
// impl<T> Handler<T> {
//     pub(crate) async fn call(&self, req: Request) -> Result<Vec<&str>, SilentError> {
//         let _ = req;
//         println!("{:?}", self.middlewares);
//         let data = vec!["foo", "bar"];
//         Ok(data)
//     }
// }
