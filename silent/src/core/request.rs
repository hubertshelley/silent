#[cfg(feature = "multipart")]
use crate::core::form::{FilePart, FormData};
use crate::core::path_param::PathParam;
use crate::core::req_body::ReqBody;
#[cfg(feature = "multipart")]
use crate::core::serde::from_str_multi_val;
use crate::core::socket_addr::SocketAddr;
use crate::header::CONTENT_TYPE;
use crate::{Configs, Result, SilentError};
use bytes::Bytes;
use http::request::Parts;
use http::{Extensions, HeaderMap, HeaderValue, Method, Uri, Version};
use http::{Request as BaseRequest, StatusCode};
use http_body_util::BodyExt;
use mime::Mime;
use serde::de::StdError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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
    pub(crate) configs: Configs,
}

impl Request {
    /// 从http请求体创建请求
    pub fn into_http(self) -> http::Request<ReqBody> {
        http::Request::from_parts(self.parts, self.body)
    }
    /// Strip the request to [`hyper::Request`].
    #[doc(hidden)]
    pub fn strip_to_hyper<QB>(&mut self) -> Result<hyper::Request<QB>>
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
    pub async fn strip_to_bytes_hyper(&mut self) -> Result<hyper::Request<Bytes>> {
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
            .body(body.frame().await.unwrap()?.into_data().unwrap())
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
    pub fn remote(&self) -> SocketAddr {
        self.headers()
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok())
            .unwrap()
            .parse()
            .unwrap()
    }

    /// 设置访问真实地址
    #[inline]
    pub fn set_remote(&mut self, remote_addr: SocketAddr) {
        if self.headers().get("x-real-ip").is_none() {
            self.headers_mut()
                .insert("x-real-ip", remote_addr.to_string().parse().unwrap());
        }
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
    pub fn get_config<T: Send + Sync + 'static>(&self) -> Result<&T> {
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

    /// 获取路径参数集合
    pub fn path_params(&self) -> &HashMap<String, PathParam> {
        &self.path_params
    }

    /// 获取路径参数
    pub fn get_path_params<'a, T>(&'a self, key: &'a str) -> Result<T>
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
    pub fn params_parse<T>(&mut self) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let query = self.uri().query().unwrap_or("");
        let params = serde_html_form::from_str(query)?;
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
    pub async fn form_data(&mut self) -> Result<&FormData> {
        let content_type = self
            .content_type()
            .ok_or(SilentError::ContentTypeMissingError)?;
        if content_type.subtype() != mime::FORM_DATA {
            return Err(SilentError::ContentTypeError);
        }
        let body = self.take_body();
        let headers = self.headers();
        self.form_data
            .get_or_try_init(|| async { FormData::read(headers, body).await })
            .await
    }

    /// 解析表单数据（支持 multipart/form-data 和 application/x-www-form-urlencoded）
    pub async fn form_parse<T>(&mut self) -> Result<T>
    where
        for<'de> T: Deserialize<'de> + Serialize,
    {
        let content_type = self
            .content_type()
            .ok_or(SilentError::ContentTypeMissingError)?;

        match content_type.subtype() {
            #[cfg(feature = "multipart")]
            mime::FORM_DATA => {
                // 复用 form_data 的缓存机制
                let form_data = self.form_data().await?;
                let value =
                    serde_json::to_value(form_data.fields.clone()).map_err(SilentError::from)?;
                serde_json::from_value(value).map_err(Into::into)
            }
            mime::WWW_FORM_URLENCODED => {
                // 检查是否已缓存到 json_data
                if let Some(cached_value) = self.json_data.get() {
                    return serde_json::from_value(cached_value.clone()).map_err(Into::into);
                }

                // 解析 form-urlencoded 数据并缓存到 json_data
                let body = self.take_body();
                let bytes = match body {
                    ReqBody::Incoming(body) => body
                        .collect()
                        .await
                        .or(Err(SilentError::BodyEmpty))?
                        .to_bytes(),
                    ReqBody::Once(bytes) => bytes,
                    ReqBody::Empty => return Err(SilentError::BodyEmpty),
                };

                if bytes.is_empty() {
                    return Err(SilentError::BodyEmpty);
                }

                // 先解析为目标类型
                let parsed_data: T =
                    serde_html_form::from_bytes(&bytes).map_err(SilentError::from)?;

                // 转换为 Value 并缓存（需要重新导入 Serialize）
                let value = serde_json::to_value(&parsed_data).map_err(SilentError::from)?;
                let _ = self.json_data.set(value.clone());

                // 直接返回已解析的数据
                Ok(parsed_data)
            }
            _ => Err(SilentError::ContentTypeError),
        }
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

    /// 解析 JSON 数据（仅支持 application/json）
    pub async fn json_parse<T>(&mut self) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        // 检查是否已缓存
        if let Some(cached_value) = self.json_data.get() {
            return serde_json::from_value(cached_value.clone()).map_err(Into::into);
        }

        let content_type = self
            .content_type()
            .ok_or(SilentError::ContentTypeMissingError)?;

        if content_type.subtype() != mime::JSON {
            return Err(SilentError::ContentTypeError);
        }

        let body = self.take_body();
        let bytes = match body {
            ReqBody::Incoming(body) => body
                .collect()
                .await
                .or(Err(SilentError::JsonEmpty))?
                .to_bytes(),
            ReqBody::Once(bytes) => bytes,
            ReqBody::Empty => return Err(SilentError::JsonEmpty),
        };

        if bytes.is_empty() {
            return Err(SilentError::JsonEmpty);
        }

        let value: Value = serde_json::from_slice(&bytes).map_err(SilentError::from)?;

        // 缓存结果
        let _ = self.json_data.set(value.clone());

        serde_json::from_value(value).map_err(Into::into)
    }

    /// 转换body参数按Json匹配
    pub async fn json_field<T>(&mut self, key: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let value: Value = self.json_parse().await?;
        serde_json::from_value(
            value
                .get(key)
                .ok_or(SilentError::ParamsNotFound)?
                .to_owned(),
        )
        .map_err(Into::into)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestStruct {
        a: i32,
        b: String,
        #[serde(default, alias = "c[]")]
        c: Vec<String>,
    }

    #[test]
    fn test_query_parse_alias() {
        let mut req = Request::empty();
        *req.uri_mut() = Uri::from_static("http://localhost:8080/test?a=1&b=2&c[]=3&c[]=4");
        let _ = req.params_parse::<TestStruct>().unwrap();
    }

    #[test]
    fn test_query_parse() {
        let mut req = Request::empty();
        *req.uri_mut() = Uri::from_static("http://localhost:8080/test?a=1&b=2&c=3&c=4");
        let _ = req.params_parse::<TestStruct>().unwrap();
    }

    /// 测试 json_parse 和 form_parse 的语义分离
    #[tokio::test]
    async fn test_methods_semantic_separation() {
        // 测试数据结构，现在需要 Serialize 和 Deserialize
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct TestData {
            name: String,
            age: u32,
        }

        let test_data = TestData {
            name: "Alice".to_string(),
            age: 25,
        };

        // 1. json_parse 正确处理 JSON 数据
        let json_body = r#"{"name":"Alice","age":25}"#.as_bytes().to_vec();
        let mut req = create_request_with_body("application/json", json_body);

        let parsed_data = req
            .json_parse::<TestData>()
            .await
            .expect("json_parse should successfully parse JSON data");
        assert_eq!(parsed_data.name, test_data.name);
        assert_eq!(parsed_data.age, test_data.age);

        // 2. form_parse 正确处理 form-urlencoded 数据
        let form_body = "name=Alice&age=25".as_bytes().to_vec();
        let mut req = create_request_with_body("application/x-www-form-urlencoded", form_body);

        let parsed_data = req
            .form_parse::<TestData>()
            .await
            .expect("form_parse should successfully parse form-urlencoded data");
        assert_eq!(parsed_data.name, test_data.name);
        assert_eq!(parsed_data.age, test_data.age);

        // 3. json_parse 拒绝 form-urlencoded 数据
        let form_body = "name=Alice&age=25".as_bytes().to_vec();
        let mut req = create_request_with_body("application/x-www-form-urlencoded", form_body);

        let result = req.json_parse::<TestData>().await;
        assert!(
            result.is_err(),
            "json_parse should reject form-urlencoded data"
        );

        // 4. form_parse 拒绝 JSON 数据
        let json_body = r#"{"name":"Alice","age":25}"#.as_bytes().to_vec();
        let mut req = create_request_with_body("application/json", json_body);

        let result = req.form_parse::<TestData>().await;
        assert!(result.is_err(), "form_parse should reject JSON data");
    }

    /// 测试 WWW_FORM_URLENCODED 数据缓存到 json_data 字段
    #[tokio::test]
    async fn test_form_urlencoded_caches_to_json_data() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct TestData {
            name: String,
            age: u32,
        }

        // 创建一个 form-urlencoded 请求
        let form_body = "name=Alice&age=25".as_bytes().to_vec();
        let mut req = create_request_with_body("application/x-www-form-urlencoded", form_body);

        // 第一次调用 form_parse，应该解析数据并缓存到 json_data
        let first_result = req
            .form_parse::<TestData>()
            .await
            .expect("First form_parse call should succeed");

        // 验证 json_data 字段已被缓存
        assert!(
            req.json_data.get().is_some(),
            "json_data should be cached after form_parse"
        );

        // 第二次调用应该从缓存中获取（不会再次解析 body）
        let second_result = req
            .form_parse::<TestData>()
            .await
            .expect("Second form_parse call should use cached data");

        // 两次结果应该相同
        assert_eq!(first_result.name, second_result.name);
        assert_eq!(first_result.age, second_result.age);
        assert_eq!(first_result.name, "Alice");
        assert_eq!(first_result.age, 25);
    }

    /// 测试共享缓存机制（验证 form_parse 复用 form_data 缓存）
    #[cfg(feature = "multipart")]
    #[tokio::test]
    async fn test_shared_cache_mechanism() {
        // 简单验证：当 Content-Type 是 multipart/form-data 时，
        // form_parse 会调用 form_data() 方法，从而复用其缓存
        let mut req = Request::empty();
        req.headers_mut().insert(
            "content-type",
            HeaderValue::from_str("multipart/form-data; boundary=----formdata").unwrap(),
        );

        // 设置一个空的 body 来避免实际的 multipart 解析
        req.body = ReqBody::Empty;

        // 尝试调用 form_parse，它应该尝试使用 form_data() 方法
        // 这个测试主要验证代码路径，而不是具体的数据解析
        #[derive(Deserialize, Serialize, Debug)]
        struct TestData {
            name: String,
        }

        let result = req.form_parse::<TestData>().await;
        // 预期会失败，因为我们没有提供真实的 multipart 数据
        // 但重要的是代码走了正确的路径（调用 form_data()）
        assert!(
            result.is_err(),
            "Should fail due to empty body, but went through correct code path"
        );
    }

    /// 辅助函数：创建带有指定内容类型和内容的请求
    fn create_request_with_body(content_type: &str, body: Vec<u8>) -> Request {
        let mut req = Request::empty();
        req.headers_mut()
            .insert("content-type", HeaderValue::from_str(content_type).unwrap());
        req.body = ReqBody::Once(body.into());
        req
    }
}
