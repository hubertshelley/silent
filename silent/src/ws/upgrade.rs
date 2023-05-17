use crate::header::HeaderMap;
use crate::prelude::PathParam;
use crate::{Request, Result};
use hyper::upgrade;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone)]
pub struct WebSocketParts {
    path_params: HashMap<String, PathParam>,
    params: HashMap<String, String>,
    headers: HeaderMap,
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
        },
        upgrade,
    })
}
