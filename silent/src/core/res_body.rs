// use bytes::Bytes;
// use http_body_util::{BodyExt, Full};
//
// pub type ResBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;
//
// pub fn full<T: Into<Bytes>>(chunk: T) -> ResBody {
//     Full::new(chunk.into())
//         .map_err(|never| match never {})
//         .boxed()
// }
