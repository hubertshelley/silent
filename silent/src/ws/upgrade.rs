use crate::header::{HeaderMap, HeaderValue};
use crate::prelude::PathParam;
use crate::{Request, Result};
use hyper::http::Extensions;
use hyper::upgrade;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WebSocketParts {
    path_params: HashMap<String, PathParam>,
    params: HashMap<String, String>,
    headers: HeaderMap<HeaderValue>,
    extensions: Extensions,
}

impl WebSocketParts {
    #[inline]
    pub fn path_params(&self) -> &HashMap<String, PathParam> {
        &self.path_params
    }

    #[inline]
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.headers
    }

    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

pub(crate) struct Upgraded {
    head: WebSocketParts,
    upgrade: upgrade::Upgraded,
}

#[allow(dead_code)]
impl Upgraded {
    pub(crate) fn into_parts(self) -> (WebSocketParts, upgrade::Upgraded) {
        (self.head, self.upgrade)
    }

    #[inline]
    pub fn path_params(&self) -> &HashMap<String, PathParam> {
        &self.head.path_params
    }

    #[inline]
    pub fn params(&self) -> &HashMap<String, String> {
        &self.head.params
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.head.headers
    }

    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.head.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.head.extensions
    }
}

pub(crate) async fn on(mut req: Request) -> Result<Upgraded> {
    let headers = req.headers().clone();
    let path_params = req.path_params().clone();
    let params = req.params().clone();
    let upgrade = upgrade::on(req.req_mut()).await?;
    Ok(Upgraded {
        head: WebSocketParts {
            path_params,
            params,
            headers,
            extensions: Default::default(),
        },
        upgrade,
    })
}
