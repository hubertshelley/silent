#[cfg(feature = "server")]
use crate::core::form::FilePart;
use crate::core::form::FormData;
use crate::core::path_param::PathParam;
use crate::core::req_body::ReqBody;
use crate::core::serde::from_str_multi_val;
use crate::header::CONTENT_TYPE;
use crate::SilentError;
#[cfg(feature = "cookie")]
use crate::{header, StatusCode};
#[cfg(feature = "cookie")]
use cookie::{Cookie, CookieJar};
use http_body_util::BodyExt;
use hyper::http::Extensions;
use hyper::Request as HyperRequest;
use mime::Mime;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use tokio::sync::OnceCell;
use url::form_urlencoded;

/// 请求体
/// ```
/// use silent::Request;
/// let req = Request::empty();
/// ```
#[derive(Debug)]
pub struct Request {
    req: HyperRequest<ReqBody>,
    path_params: HashMap<String, PathParam>,
    params: HashMap<String, String>,
    body: ReqBody,
    form_data: OnceCell<FormData>,
    json_data: OnceCell<Value>,
    #[cfg(feature = "cookie")]
    pub(crate) cookies: CookieJar,
}

impl Default for Request {
    fn default() -> Self {
        Self::empty()
    }
}

impl Request {
    /// 创建空请求体
    pub fn empty() -> Self {
        Self {
            req: HyperRequest::builder()
                .method("GET")
                .body(().into())
                .unwrap(),
            path_params: HashMap::new(),
            params: HashMap::new(),
            body: ReqBody::Empty,
            form_data: OnceCell::new(),
            json_data: OnceCell::new(),
            #[cfg(feature = "cookie")]
            cookies: CookieJar::default(),
        }
    }

    pub(crate) fn set_path_params(&mut self, key: String, value: PathParam) {
        self.path_params.insert(key, value);
    }

    /// 获取可变原请求体
    pub fn req_mut(&mut self) -> &mut HyperRequest<ReqBody> {
        &mut self.req
    }

    /// 获取路径参数集合
    pub fn path_params(&self) -> &HashMap<String, PathParam> {
        &self.path_params
    }

    /// 获取路径参数
    pub fn get_path_params<'a, T>(&'a self, key: &'a str) -> Result<T, SilentError>
    where
        T: TryFrom<&'a PathParam, Error = SilentError>,
    {
        match self.path_params.get(key) {
            Some(value) => value.try_into(),
            None => Err(SilentError::ParamsNotFound),
        }
    }

    /// 获取query参数
    pub fn params(&mut self) -> &HashMap<String, String> {
        if let Some(query) = self.uri().query() {
            let params = form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect::<HashMap<String, String>>();
            self.params = params;
        };
        &self.params
    }

    /// 转换query参数
    pub fn params_parse<T>(&mut self) -> Result<T, SilentError>
    where
        for<'de> T: Deserialize<'de>,
    {
        let query = self.uri().query().unwrap_or("");
        let params = serde_urlencoded::from_str(query)?;
        Ok(params)
    }

    /// 获取请求body
    #[inline]
    pub fn replace_body(&mut self, body: ReqBody) -> ReqBody {
        std::mem::replace(&mut self.body, body)
    }

    /// 获取请求body
    #[inline]
    pub fn take_body(&mut self) -> ReqBody {
        self.replace_body(ReqBody::Empty)
    }

    /// 获取请求content_type
    #[inline]
    pub fn content_type(&self) -> Option<Mime> {
        self.headers()
            .get(CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.parse().ok())
    }

    /// 获取请求form_data
    #[inline]
    pub async fn form_data(&mut self) -> Result<&FormData, SilentError> {
        let content_type = self.content_type().unwrap();
        if content_type.subtype() != mime::FORM_DATA {
            return Err(SilentError::ContentTypeError);
        }
        let body = self.take_body();
        let headers = self.headers();
        self.form_data
            .get_or_try_init(|| async { FormData::read(headers, body).await })
            .await
    }

    /// 转换body参数
    pub async fn body_parse<T>(&mut self, key: &str) -> Option<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        self.form_data()
            .await
            .ok()
            .and_then(|ps| ps.fields.get_vec(key))
            .and_then(|vs| from_str_multi_val(vs).ok())
    }

    /// 获取上传的文件
    #[cfg(feature = "server")]
    #[inline]
    pub async fn files<'a>(&'a mut self, key: &'a str) -> Option<&'a Vec<FilePart>> {
        self.form_data()
            .await
            .ok()
            .and_then(|ps| ps.files.get_vec(key))
    }

    /// 转换body参数按Json匹配
    pub async fn json_parse<T>(&mut self) -> Result<T, SilentError>
    where
        for<'de> T: Deserialize<'de>,
    {
        let body = self.take_body();
        let content_type = self.content_type().unwrap();
        if content_type.subtype() == mime::FORM_DATA {
            return Err(SilentError::ContentTypeError);
        }
        match body {
            ReqBody::Incoming(body) => {
                let value = self
                    .json_data
                    .get_or_try_init(|| async {
                        match content_type.subtype() {
                            mime::WWW_FORM_URLENCODED => {
                                let bytes = body.collect().await.unwrap().to_bytes();
                                serde_urlencoded::from_bytes(&bytes).map_err(SilentError::from)
                            }
                            mime::JSON => {
                                let bytes = body.collect().await.unwrap().to_bytes();
                                serde_json::from_slice(&bytes).map_err(|e| e.into())
                            }
                            _ => Err(SilentError::JsonEmpty),
                        }
                    })
                    .await?;
                Ok(serde_json::from_value(value.to_owned())?)
            }
            ReqBody::Empty => Err(SilentError::BodyEmpty),
        }
    }

    /// 获取请求body
    #[inline]
    pub fn replace_extensions(&mut self, extensions: Extensions) -> Extensions {
        std::mem::replace(self.extensions_mut(), extensions)
    }

    /// 获取请求body
    #[inline]
    pub fn take_extensions(&mut self) -> Extensions {
        self.replace_extensions(Extensions::default())
    }

    /// 分割请求体与url
    pub(crate) fn split_url(self) -> (Self, String) {
        let url = self.uri().path().to_string();
        (self, url)
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
}

#[cfg(feature = "cookie")]
fn get_cookie(req: &HyperRequest<ReqBody>) -> Result<CookieJar, SilentError> {
    let mut jar = CookieJar::new();
    if let Some(cookies) = req.headers().get(header::COOKIE) {
        for cookie_str in cookies
            .to_str()
            .map_err(|e| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse cookie: {}", e),
                )
            })?
            .split(';')
            .map(|s| s.trim())
        {
            if let Ok(cookie) = Cookie::parse_encoded(cookie_str).map(|c| c.into_owned()) {
                jar.add_original(cookie);
            }
        }
    }
    Ok(jar)
}

impl From<HyperRequest<ReqBody>> for Request {
    #[cfg(feature = "cookie")]
    fn from(req: HyperRequest<ReqBody>) -> Self {
        let cookies = get_cookie(&req).unwrap_or(CookieJar::default());
        let (parts, body) = req.into_parts();
        Self {
            req: HyperRequest::from_parts(parts, ReqBody::Empty),
            body,
            cookies,
            ..Self::default()
        }
    }
    #[cfg(not(feature = "cookie"))]
    fn from(req: HyperRequest<ReqBody>) -> Self {
        let (parts, body) = req.into_parts();
        Self {
            req: HyperRequest::from_parts(parts, ReqBody::Empty),
            body,
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
