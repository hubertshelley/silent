use crate::core::res_body::{full, ResBody};
use crate::{HeaderMap, StatusCode};
use bytes::Bytes;
use headers::{Header, HeaderMapExt};
use std::fmt;
use std::fmt::{Display, Formatter};

/// 响应体
/// ```
/// use silent::Response;
/// let req = Response::empty();
/// ```
pub struct Response {
    /// The HTTP status code.
    pub(crate) status_code: StatusCode,
    /// The HTTP headers.
    pub(crate) headers: HeaderMap,
    pub(crate) body: ResBody,
}

impl fmt::Debug for Response {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "HTTP/1.1 {}\n{:?}", self.status_code, self.headers)
    }
}

impl Display for Response {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Response {
    /// 创建空响应体
    pub fn empty() -> Self {
        Self {
            status_code: StatusCode::OK,
            headers: HeaderMap::new(),
            body: ResBody::None,
        }
    }
    /// 设置响应状态
    pub fn set_status(&mut self, status: StatusCode) {
        self.status_code = status;
    }
    /// 设置响应body
    pub fn set_body(&mut self, body: ResBody) {
        self.body = body;
    }
    /// 设置响应header
    pub fn set_header(
        mut self,
        key: hyper::header::HeaderName,
        value: hyper::header::HeaderValue,
    ) -> Self {
        self.headers.insert(key, value);
        self
    }
    /// 设置响应header
    pub fn set_typed_header<H>(&mut self, header: H)
    where
        H: Header,
    {
        self.headers.typed_insert(header);
    }

    #[inline]
    pub(crate) fn into_hyper(self) -> hyper::Response<ResBody> {
        let Self {
            status_code,
            headers,
            body,
        } = self;

        let mut res = hyper::Response::new(body);
        *res.headers_mut() = headers;
        // Default to a 404 if no response code was set
        *res.status_mut() = status_code;

        res
    }
}

impl<T: Into<Bytes>> From<T> for Response {
    fn from(chunk: T) -> Self {
        let mut res = Response::empty();
        res.set_body(full(chunk.into()));
        res
    }
}
