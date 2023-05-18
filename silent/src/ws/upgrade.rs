use crate::header::HeaderMap;
use crate::prelude::PathParam;
use crate::{Request, Result};
use hyper::upgrade;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WebSocketParts {
    path_params: HashMap<String, PathParam>,
    params: HashMap<String, String>,
    headers: HeaderMap,
    extra: HashMap<String, String>,
}

impl WebSocketParts {
    pub fn path_params(&self) -> &HashMap<String, PathParam> {
        &self.path_params
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn extra(&self) -> &HashMap<String, String> {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.extra
    }
}

pub(crate) struct Upgraded {
    parts: WebSocketParts,
    upgrade: upgrade::Upgraded,
}

impl Upgraded {
    pub(crate) fn into_parts(self) -> (WebSocketParts, upgrade::Upgraded) {
        (self.parts.clone(), self.upgrade)
    }
}

pub(crate) async fn on(mut req: Request) -> Result<Upgraded> {
    let headers = req.headers().clone();
    let path_params = req.path_params().clone();
    let params = req.params().clone();
    let upgrade = upgrade::on(req.req_mut()).await?;
    Ok(Upgraded {
        parts: WebSocketParts {
            path_params,
            params,
            headers,
            extra: Default::default(),
        },
        upgrade,
    })
}
