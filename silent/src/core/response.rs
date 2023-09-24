use crate::core::res_body::{full, ResBody};
use crate::{header, HeaderMap, Result, SilentError, StatusCode};
use bytes::Bytes;
#[cfg(feature = "cookie")]
use cookie::{Cookie, CookieJar};
use headers::{ContentType, Header, HeaderMapExt};
use http::Extensions;
use http_body::{Body, SizeHint};
use serde::Serialize;
use serde_json::Value;
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
    #[cfg(feature = "cookie")]
    pub(crate) cookies: CookieJar,
    pub(crate) extensions: Extensions,
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
            #[cfg(feature = "cookie")]
            cookies: CookieJar::default(),
            extensions: Extensions::default(),
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
    pub fn set_header(mut self, key: header::HeaderName, value: header::HeaderValue) -> Self {
        self.headers.insert(key, value);
        self
    }
    #[inline]
    /// 获取extensions
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }
    #[inline]
    /// 获取extensions_mut
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
    #[inline]
    /// 设置响应重定向
    pub fn redirect(url: &str) -> Result<Self> {
        let mut res = Self::empty();
        res.status_code = StatusCode::MOVED_PERMANENTLY;
        res.headers.insert(
            header::LOCATION,
            url.parse().map_err(|e| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("redirect error: {}", e),
                )
            })?,
        );
        Ok(res)
    }
    #[inline]
    /// 设置响应header
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    #[inline]
    /// 设置响应header
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    #[inline]
    /// 获取响应体长度
    pub fn content_length(&self) -> SizeHint {
        self.body.size_hint()
    }
    #[inline]
    /// 设置响应header
    pub fn set_typed_header<H>(&mut self, header: H)
    where
        H: Header,
    {
        self.headers.typed_insert(header);
    }

    #[inline]
    pub(crate) fn into_hyper(self) -> hyper::Response<ResBody> {
        #[cfg(feature = "cookie")]
        let Self {
            status_code,
            headers,
            body,
            cookies,
            ..
        } = self;
        #[cfg(not(feature = "cookie"))]
        let Self {
            status_code,
            headers,
            body,
            ..
        } = self;

        let mut res = hyper::Response::new(body);
        *res.headers_mut() = headers;
        #[cfg(feature = "cookie")]
        for cookie in cookies.delta() {
            if let Ok(hv) = cookie.encoded().to_string().parse() {
                res.headers_mut().append(header::SET_COOKIE, hv);
            }
        }
        // Default to a 404 if no response code was set
        *res.status_mut() = status_code;

        res
    }

    #[cfg(feature = "cookie")]
    /// Get `CookieJar` reference.
    #[inline]
    pub fn cookies(&self) -> &CookieJar {
        &self.cookies
    }
    #[cfg(feature = "cookie")]
    /// Get `CookieJar` mutable reference.
    #[inline]
    pub fn cookies_mut(&mut self) -> &mut CookieJar {
        &mut self.cookies
    }
    #[cfg(feature = "cookie")]
    /// Get `Cookie` from cookies.
    #[inline]
    pub fn cookie<T>(&self, name: T) -> Option<&Cookie<'static>>
    where
        T: AsRef<str>,
    {
        self.cookies.get(name.as_ref())
    }

    #[cfg(feature = "cookie")]
    /// move response to from another response
    pub fn from_response(&mut self, res: Response) {
        for (header_key, header_value) in res.headers.clone().into_iter() {
            if let Some(key) = header_key {
                self.headers_mut().insert(key, header_value);
            }
        }
        res.cookies.delta().for_each(|cookie| {
            self.cookies.add(cookie.clone());
        });
        self.status_code = res.status_code;
        self.extensions.extend(res.extensions);
        self.set_body(res.body);
    }

    #[cfg(not(feature = "cookie"))]
    /// move response to from another response
    pub fn from_response(&mut self, res: Response) {
        for (header_key, header_value) in res.headers.clone().into_iter() {
            if let Some(key) = header_key {
                self.headers_mut().insert(key, header_value);
            }
        }
        self.status_code = res.status_code;
        self.extensions.extend(res.extensions);
        self.set_body(res.body);
    }
}

impl<S: Serialize> From<S> for Response {
    fn from(value: S) -> Self {
        let mut res = Response::empty();
        let result: Bytes = match serde_json::to_value(&value).unwrap() {
            Value::String(value) => {
                if value.contains("html") {
                    res.set_typed_header(ContentType::html());
                } else {
                    res.set_typed_header(ContentType::text_utf8());
                }
                value.into_bytes().into()
            }
            _ => {
                res.set_typed_header(ContentType::json());
                serde_json::to_vec(&value).unwrap().into()
            }
        };
        res.set_body(full(result));
        res
    }
}
