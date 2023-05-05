use crate::core::form::{FilePart, FormData};
use crate::core::path_param::PathParam;
use crate::core::req_body::ReqBody;
use crate::core::serde::from_str_multi_val;
use crate::header::CONTENT_TYPE;
use crate::SilentError;
use http_body_util::BodyExt;
use hyper::Request as HyperRequest;
use mime::Mime;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use tokio::sync::OnceCell;
use url::form_urlencoded;

#[derive(Debug)]
pub struct Request {
    req: HyperRequest<ReqBody>,
    pub path_params: HashMap<String, PathParam>,
    params: HashMap<String, String>,
    body: ReqBody,
    form_data: OnceCell<FormData>,
    json_data: OnceCell<Value>,
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
            params: HashMap::new(),
            body: ReqBody::Empty,
            form_data: OnceCell::new(),
            json_data: OnceCell::new(),
        }
    }

    pub(crate) fn set_path_params(&mut self, key: String, value: PathParam) {
        self.path_params.insert(key, value);
    }

    pub fn path_params(&self) -> &HashMap<String, PathParam> {
        &self.path_params
    }

    pub fn get_path_params(&self, key: &str) -> Option<&PathParam> {
        self.path_params.get(key)
    }

    pub fn params(&mut self) -> &HashMap<String, String> {
        if let Some(query) = self.uri().query() {
            let params = form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect::<HashMap<String, String>>();
            self.params = params;
        };
        &self.params
    }

    pub fn params_parse<T>(&mut self) -> Result<T, SilentError>
    where
        for<'de> T: Deserialize<'de>,
    {
        let query = self.uri().query().unwrap_or("");
        let params = serde_urlencoded::from_str(query)?;
        Ok(params)
    }

    #[inline]
    pub fn replace_body(&mut self, body: ReqBody) -> ReqBody {
        std::mem::replace(&mut self.body, body)
    }

    #[inline]
    pub fn take_body(&mut self) -> ReqBody {
        self.replace_body(ReqBody::Empty)
    }

    #[inline]
    pub fn content_type(&self) -> Option<Mime> {
        self.headers()
            .get(CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.parse().ok())
    }

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

    #[inline]
    pub async fn files<'a>(&'a mut self, key: &'a str) -> Option<&'a Vec<FilePart>> {
        self.form_data()
            .await
            .ok()
            .and_then(|ps| ps.files.get_vec(key))
    }

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

    pub(crate) fn split_url(self) -> (Self, String) {
        let url = self.uri().path().to_string();
        (self, url)
    }
}

impl From<HyperRequest<ReqBody>> for Request {
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
