use crate::core::path_param::PathParam;
use crate::core::req_body::ReqBody;
use crate::SilentError;
use http_body_util::BodyExt;
use hyper::Request as HyperRequest;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use url::form_urlencoded;

#[derive(Debug)]
pub struct Request {
    req: HyperRequest<ReqBody>,
    pub path_params: HashMap<String, PathParam>,
    params: HashMap<String, String>,
    body: Option<Value>,
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
            body: None,
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

    pub async fn body(mut self) -> Result<Option<Value>, SilentError> {
        let body = self.req.into_body();
        let body = match body {
            ReqBody::Incoming(body) => {
                let body = body.collect().await?.to_bytes();
                let body = form_urlencoded::parse(body.as_ref())
                    .into_owned()
                    .collect::<Value>();
                Some(body)
            }
            ReqBody::Empty(()) => None,
        };
        self.body = body;
        Ok(self.body)
    }

    pub async fn json(mut self) -> Result<Option<Value>, SilentError> {
        let body = self.req.into_body();
        let body = match body {
            ReqBody::Incoming(body) => {
                let body = body.collect().await?.to_bytes();
                let body = serde_json::from_slice(&body)?;
                Some(body)
            }
            ReqBody::Empty(()) => None,
        };
        self.body = body;
        Ok(self.body)
    }

    pub async fn body_parse<T>(self) -> Result<T, SilentError>
    where
        for<'de> T: Deserialize<'de>,
    {
        let body = self.body().await?;
        match body {
            None => Err(SilentError::BodyEmpty),
            Some(value) => {
                let value: T = serde_json::from_value(value)?;
                Ok(value)
            }
        }
    }

    pub async fn json_parse<T>(self) -> Result<T, SilentError>
    where
        for<'de> T: Deserialize<'de>,
    {
        let body = self.json().await?;
        match body {
            None => Err(SilentError::BodyEmpty),
            Some(value) => {
                let value: T = serde_json::from_value(value)?;
                Ok(value)
            }
        }
    }

    pub(crate) fn split_url(self) -> (Self, String) {
        let url = self.uri().path().to_string();
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
