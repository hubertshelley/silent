use crate::core::res_body::{full, ResBody};
use bytes::Bytes;
use hyper::Response as HyperResponse;
use std::ops::{Deref, DerefMut};

/// 响应体
/// ```
/// use silent::Response;
/// let req = Response::empty();
/// ```
#[derive(Debug)]
pub struct Response {
    pub(crate) res: HyperResponse<ResBody>,
}

impl Response {
    /// 创建空响应体
    pub fn empty() -> Self {
        Response::from(Bytes::new())
    }
    /// 设置响应状态
    pub fn set_status(&mut self, status: hyper::StatusCode) {
        *self.res.status_mut() = status;
    }
    /// 设置响应body
    pub fn set_body(mut self, body: ResBody) -> Self {
        *self.res.body_mut() = body;
        self
    }
    /// 设置响应header
    pub fn set_header(
        mut self,
        key: hyper::header::HeaderName,
        value: hyper::header::HeaderValue,
    ) -> Self {
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
