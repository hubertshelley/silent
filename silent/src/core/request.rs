#[cfg(feature = "multipart")]
use crate::core::form::{FilePart, FormData};
use crate::core::path_param::PathParam;
use crate::core::req_body::ReqBody;
#[cfg(feature = "multipart")]
use crate::core::serde::from_str_multi_val;
use crate::header::CONTENT_TYPE;
#[cfg(feature = "scheduler")]
use crate::Scheduler;
use crate::{Configs, SilentError};
use bytes::Bytes;
#[cfg(feature = "cookie")]
use cookie::{Cookie, CookieJar};
use http::request::Parts;
use http::{Extensions, HeaderMap, HeaderValue, Method, Uri, Version};
use http::{Request as BaseRequest, StatusCode};
use http_body_util::BodyExt;
use mime::Mime;
use serde::de::StdError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::net::IpAddr;
#[cfg(feature = "scheduler")]
use std::sync::Arc;
#[cfg(feature = "scheduler")]
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use url::form_urlencoded;

/// 请求体
/// ```
/// use silent::Request;
/// let req = Request::empty();
/// ```
#[derive(Debug)]
pub struct Request {
    // req: BaseRequest<ReqBody>,
    parts: Parts,
    path_params: HashMap<String, PathParam>,
    params: HashMap<String, String>,
    body: ReqBody,
    #[cfg(feature = "multipart")]
    form_data: OnceCell<FormData>,
    json_data: OnceCell<Value>,
    #[cfg(feature = "cookie")]
    pub(crate) cookies: CookieJar,
    pub(crate) configs: Configs,
}

impl Request {
    /// 从http请求体创建请求
    pub fn into_http(self) -> http::Request<ReqBody> {
        http::Request::from_parts(self.parts, self.body)
    }
    /// Strip the request to [`hyper::Request`].
    #[doc(hidden)]
    pub fn strip_to_hyper<QB>(&mut self) -> Result<hyper::Request<QB>, SilentError>
    where
        QB: TryFrom<ReqBody>,
        <QB as TryFrom<ReqBody>>::Error: StdError + Send + Sync + 'static,
    {
        let mut builder = http::request::Builder::new()
            .method(self.method().clone())
            .uri(self.uri().clone())
            .version(self.version());
        if let Some(headers) = builder.headers_mut() {
            *headers = std::mem::take(self.headers_mut());
        }
        if let Some(extensions) = builder.extensions_mut() {
            *extensions = std::mem::take(self.extensions_mut());
        }

        let body = self.take_body();
        builder
            .body(body.try_into().map_err(|e| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("request strip to hyper failed: {e}"),
                )
            })?)
            .map_err(|e| SilentError::business_error(StatusCode::BAD_REQUEST, e.to_string()))
    }
    /// Strip the request to [`hyper::Request`].
    #[doc(hidden)]
    pub async fn strip_to_bytes_hyper(&mut self) -> Result<hyper::Request<Bytes>, SilentError> {
        let mut builder = http::request::Builder::new()
            .method(self.method().clone())
            .uri(self.uri().clone())
            .version(self.version());
        if let Some(headers) = builder.headers_mut() {
            *headers = std::mem::take(self.headers_mut());
        }
        if let Some(extensions) = builder.extensions_mut() {
            *extensions = std::mem::take(self.extensions_mut());
        }

        let mut body = self.take_body();
        builder
            .body(body.frame().await.unwrap().unwrap().into_data().unwrap())
            .map_err(|e| SilentError::business_error(StatusCode::BAD_REQUEST, e.to_string()))
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::empty()
    }
}

impl Request {
    /// 创建空请求体
    pub fn empty() -> Self {
        let (parts, _) = BaseRequest::builder()
            .method("GET")
            .body(())
            .unwrap()
            .into_parts();
        Self {
            // req: BaseRequest::builder()
            //     .method("GET")
            //     .body(().into())
            //     .unwrap(),
            parts,
            path_params: HashMap::new(),
            params: HashMap::new(),
            body: ReqBody::Empty,
            #[cfg(feature = "multipart")]
            form_data: OnceCell::new(),
            json_data: OnceCell::new(),
            #[cfg(feature = "cookie")]
            cookies: CookieJar::default(),
            configs: Configs::default(),
        }
    }

    /// 从请求体创建请求
    #[inline]
    pub fn from_parts(parts: Parts, body: ReqBody) -> Self {
        Self {
            parts,
            body,
            ..Self::default()
        }
    }

    /// 获取访问真实地址
    #[inline]
    pub fn remote(&self) -> IpAddr {
        self.headers()
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok())
            .unwrap()
            .parse()
            .unwrap()
    }

    /// 获取请求方法
    #[inline]
    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    /// 获取请求方法
    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.parts.method
    }
    /// 获取请求uri
    #[inline]
    pub fn uri(&self) -> &Uri {
        &self.parts.uri
    }
    /// 获取请求uri
    #[inline]
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.parts.uri
    }
    /// 获取请求版本
    #[inline]
    pub fn version(&self) -> Version {
        self.parts.version
    }
    /// 获取请求版本
    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.parts.version
    }
    /// 获取请求头
    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.parts.headers
    }
    /// 获取请求头
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.parts.headers
    }
    /// 获取请求拓展
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }
    /// 获取请求拓展
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }
    pub(crate) fn set_path_params(&mut self, key: String, value: PathParam) {
        self.path_params.insert(key, value);
    }

    /// 获取配置
    #[inline]
    pub fn get_config<T: Send + Sync + 'static>(&self) -> Result<&T, SilentError> {
        self.configs.get::<T>().ok_or(SilentError::ConfigNotFound)
    }

    /// 获取配置(Uncheck)
    #[inline]
    pub fn get_config_uncheck<T: Send + Sync + 'static>(&self) -> &T {
        self.configs.get::<T>().unwrap()
    }

    /// 获取全局配置
    #[inline]
    pub fn configs(&self) -> Configs {
        self.configs.clone()
    }

    /// 获取可变全局配置
    #[inline]
    pub fn configs_mut(&mut self) -> &mut Configs {
        &mut self.configs
    }

    /// 获取可变原请求体
    // pub fn req_mut(&mut self) -> &mut BaseRequest<ReqBody> {
    //     &mut self.req
    // }

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
    #[cfg(feature = "multipart")]
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
    #[cfg(feature = "multipart")]
    pub async fn form_field<T>(&mut self, key: &str) -> Option<T>
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
    #[cfg(feature = "multipart")]
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
    #[cfg(feature = "scheduler")]
    #[inline]
    /// Get `Scheduler` from extensions.
    pub fn scheduler(&self) -> &Arc<Mutex<Scheduler>> {
        self.extensions().get().unwrap()
    }
}
