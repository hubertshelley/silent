use crate::{Handler, MiddleWareHandler, Next, Request, Response, Result};
use async_trait::async_trait;
use chrono::Utc;

/// ExceptionHandler 中间件
/// ```rust
/// use silent::prelude::*;
/// use silent::middlewares::{RequestTimeLogger};
/// // Define a request time logger middleware
/// let _ = RequestTimeLogger::new();
#[derive(Default, Clone)]
pub struct RequestTimeLogger;

impl RequestTimeLogger {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MiddleWareHandler for RequestTimeLogger {
    async fn handle(&self, req: Request, next: &Next) -> Result<Response> {
        let method = req.method().clone();
        let url = req.uri().to_string().clone();
        let http_version = req.version();
        let peer_addr = req.remote();
        let start_time = Utc::now().time();
        let res = next.call(req).await;
        let end_time = Utc::now().time();
        let req_time = end_time - start_time;
        Ok(match res {
            Ok(res) => {
                if res.status.as_u16() >= 400 {
                    tracing::info!(
                        "{} {} {} {:?} {} {:?} {}",
                        peer_addr,
                        method,
                        url,
                        http_version,
                        res.status.as_u16(),
                        res.content_length().lower(),
                        req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0
                    );
                } else {
                    tracing::warn!(
                        "{} {} {} {:?} {} {:?} {}",
                        peer_addr,
                        method,
                        url,
                        http_version,
                        res.status.as_u16(),
                        res.content_length().lower(),
                        req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0
                    );
                }
                res
            }
            Err(e) => {
                tracing::error!(
                    "{} {} {} {:?} {} {:?} {} {}",
                    peer_addr,
                    method,
                    url,
                    http_version,
                    e.status().as_u16(),
                    0,
                    req_time.num_nanoseconds().unwrap_or(0) as f64 / 1000000.0,
                    e.to_string()
                );
                e.into()
            }
        })
    }
}
