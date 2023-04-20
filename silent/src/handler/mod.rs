// use std::future::Future;
// use std::pin::Pin;
// use bytes::Bytes;
// use http_body_util::Full;
// use hyper::{body::Incoming as IncomingBody, Request, Response};
// use crate::error::SilentError;
//
// pub struct Handler<T> {
//     pub handler: Pin<Box<dyn Future<Output=T> + 'static>>,
//     pub middlewares: Vec<String>,
// }
//
// impl<T> Handler<T> {
//     pub(crate) async fn call(&self, req: Request<IncomingBody>) -> Result<Response<Full<Bytes>>, SilentError> {
//         println!("{:?}", self.middlewares);
//         Ok(Response::builder().body(Full::new(Bytes::from(self.handler(req)?.await))).unwrap())
//     }
// }
