use crate::conn::SilentConnection;
use crate::core::req_body::ReqBody;
use crate::core::res_body::ResBody;
use crate::core::response::Response;
use crate::header::CONTENT_TYPE;
use crate::route::Routes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpStream;

pub(crate) struct Serve {
    pub(crate) routes: Routes,
    pub(crate) conn: Arc<SilentConnection>,
}

impl Serve {
    pub(crate) fn new(routes: Routes, conn: Arc<SilentConnection>) -> Self {
        Self { routes, conn }
    }
    pub(crate) async fn call(&self, stream: TcpStream) -> Result<(), hyper::Error> {
        let service = service_fn(move |req| self.handle(req));
        self.conn
            .http1
            .serve_connection(stream, service)
            .with_upgrades()
            .await
    }

    async fn handle(
        &self,
        req: HyperRequest<Incoming>,
    ) -> Result<HyperResponse<ResBody>, hyper::Error> {
        let (parts, body) = req.into_parts();
        let content_type = parts.headers.get(CONTENT_TYPE);
        let body = match content_type {
            Some(content_type) => {
                let content_type = content_type.to_str().unwrap();
                let content_type = content_type.split(';').next().unwrap();
                println!("{}", content_type);
                match content_type {
                    "application/json" => {
                        let body_bytes = body.collect().await?.to_bytes();
                        serde_json::from_slice(&body_bytes).unwrap_or(Value::Null)
                    }
                    "application/x-www-form-urlencoded" => {
                        let body_bytes = body.collect().await?.to_bytes();
                        println!("{:?}", body_bytes);
                        serde_urlencoded::from_bytes(&body_bytes).unwrap_or(Value::Null)
                    }
                    _ => Value::Null,
                }
            }
            _ => Value::Null,
        };
        let req = (HyperRequest::from_parts(parts, ReqBody::Empty(())), body).into();
        match self.routes.handle(req).await {
            Ok(res) => Ok(res.res),
            Err((mes, code)) => {
                tracing::error!("Failed to handle request: {:?}", mes);
                let mut res = Response::from(mes);
                res.set_status(code);
                Ok(res.res)
            }
        }
    }
}
