use crate::core::path_param::PathParam;
use crate::core::req_body::ReqBody;
use crate::header::{HeaderValue, CONTENT_TYPE};
use crate::SilentError;
use http_body_util::BodyExt;
use hyper::Request as HyperRequest;
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
    form_data: OnceCell<Value>,
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

    pub async fn body_parse<T>(&mut self) -> Result<T, SilentError>
    where
        for<'de> T: Deserialize<'de>,
    {
        let application_json = HeaderValue::from_static("application/json");
        let body = self.take_body();
        let content_type = self
            .headers()
            .get(CONTENT_TYPE)
            .unwrap_or(&application_json)
            .to_str()
            .unwrap_or("application/json");
        match body {
            ReqBody::Incoming(body) => {
                let value = self
                    .form_data
                    .get_or_try_init(|| async {
                        let bytes = body.collect().await.unwrap().to_bytes();
                        match content_type {
                            "application/x-www-form-urlencoded" => {
                                serde_urlencoded::from_bytes(&bytes).map_err(SilentError::from)
                            }
                            "application/json" => {
                                serde_json::from_slice(&bytes).map_err(|e| e.into())
                            }
                            _ => serde_json::from_slice(&bytes).map_err(|e| e.into()),
                        }
                        // serde_json::from_slice(&bytes)
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
