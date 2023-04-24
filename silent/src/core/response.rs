use crate::core::res_body::{full, ResBody};
use bytes::Bytes;
use hyper::Response as HyperResponse;
use std::ops::{Deref, DerefMut};

pub struct Response {
    pub res: HyperResponse<ResBody>,
}

impl Response {
    #[allow(dead_code)]
    pub fn set_status(&mut self, status: hyper::StatusCode) -> &mut Self {
        *self.status_mut() = status;
        self
    }
    #[allow(dead_code)]
    pub fn set_header(
        &mut self,
        key: hyper::header::HeaderName,
        value: hyper::header::HeaderValue,
    ) -> &mut Self {
        self.headers_mut().insert(key, value);
        self
    }
}

impl<T: Into<Bytes>> From<T> for Response {
    fn from(chunk: T) -> Self {
        Self {
            res: HyperResponse::new(full(chunk)),
        }
    }
}

impl Deref for Response {
    type Target = HyperResponse<ResBody>;

    fn deref(&self) -> &Self::Target {
        &self.res
    }
}

impl DerefMut for Response {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.res
    }
}
