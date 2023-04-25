use crate::core::req_body::ReqBody;
use hyper::Request as HyperRequest;

pub type Request = HyperRequest<ReqBody>;
