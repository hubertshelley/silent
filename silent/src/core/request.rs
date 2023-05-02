use crate::core::path_param::PathParam;
use crate::core::req_body::ReqBody;
use hyper::Request as HyperRequest;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Request {
    req: HyperRequest<ReqBody>,
    pub path_params: HashMap<String, PathParam>,
}

impl Default for Request {
    fn default() -> Self {
        Self::empty()
    }
}

impl Request {
    pub fn empty() -> Self {
        Self {
            req: HyperRequest::builder()
                .method("GET")
                .body(().into())
                .unwrap(),
            path_params: HashMap::new(),
        }
    }

    pub(crate) fn set_path_params(&mut self, key: String, value: PathParam) {
        self.path_params.insert(key, value);
    }

    pub fn path_params(&self) -> &HashMap<String, PathParam> {
        &self.path_params
    }

    pub(crate) fn split_url(self) -> (Self, String) {
        let url = self.uri().to_string();
        (self, url)
    }
}

impl From<HyperRequest<ReqBody>> for Request {
    fn from(req: HyperRequest<ReqBody>) -> Self {
        Self {
            req,
            ..Self::default()
        }
    }
}

impl Deref for Request {
    type Target = HyperRequest<ReqBody>;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}

impl DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.req
    }
}
