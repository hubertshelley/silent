use std::sync::Arc;

use super::utils::merge_grpc_response;
use crate::grpc::service::GrpcService;
use crate::prelude::HandlerGetter;
use crate::route::Route;
use crate::{Handler, Response, SilentError};
use async_trait::async_trait;
use http::{header, HeaderValue, Method, StatusCode};
use http_body_util::BodyExt;
use hyper::upgrade::OnUpgrade;
use hyper_util::rt::TokioExecutor;
use tokio::sync::Mutex;
use tonic::body::BoxBody;
use tonic::codegen::Service;
use tonic::server::NamedService;
use tonic::Status;
use tracing::{error, info};

trait GrpcRequestAdapter {
    fn into_grpc_request(self) -> http::Request<BoxBody>;
}

impl GrpcRequestAdapter for crate::Request {
    fn into_grpc_request(self) -> http::Request<BoxBody> {
        let (parts, body) = self.into_http().into_parts();
        http::Request::from_parts(
            parts,
            body.map_err(|e| {
                Status::internal(format!("convert request to http request failed: {}", e))
            })
            .boxed_unsync(),
        )
    }
}

#[derive(Clone)]
pub struct GrpcHandler<S> {
    inner: Arc<Mutex<S>>,
}

impl<S> GrpcHandler<S>
where
    S: Service<http::Request<BoxBody>, Response = http::Response<BoxBody>> + NamedService,
    S: Clone + Send + 'static,
    S: Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    pub fn new(service: S) -> Self {
        Self {
            inner: Arc::new(Mutex::new(service)),
        }
    }
    pub fn path(&self) -> &str {
        S::NAME
    }

    pub fn service(self) -> Route {
        let path = self.path().to_string();
        let handler = Arc::new(self);
        Route::new(path.as_str()).append(
            Route::new("<path:**>")
                .insert_handler(Method::POST, handler.clone())
                .insert_handler(Method::GET, handler),
        )
    }

    pub fn register(self, route: &mut Route) {
        route.extend(self.service());
    }
}

impl<S> From<S> for GrpcHandler<S>
where
    S: Service<http::Request<BoxBody>, Response = http::Response<BoxBody>> + NamedService,
    S: Clone + Send + 'static,
    S: Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    fn from(service: S) -> Self {
        Self {
            inner: Arc::new(Mutex::new(service)),
        }
    }
}

#[async_trait]
impl<S> Handler for GrpcHandler<S>
where
    S: Service<http::Request<BoxBody>, Response = http::Response<BoxBody>>,
    S: Clone + Send + 'static,
    S: Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    async fn call(&self, mut req: crate::Request) -> crate::Result<Response> {
        if let Some(on_upgrade) = req.extensions_mut().remove::<OnUpgrade>() {
            let handler = self.inner.clone();
            tokio::spawn(async move {
                let conn = on_upgrade.await;
                if conn.is_err() {
                    error!("upgrade error: {:?}", conn.err());
                    return;
                }
                let upgraded_io = conn.unwrap();

                let http = hyper::server::conn::http2::Builder::new(TokioExecutor::new());
                match http
                    .serve_connection(upgraded_io, GrpcService::new(handler))
                    .await
                {
                    Ok(_) => info!("finished gracefully"),
                    Err(err) => error!("ERROR: {err}"),
                }
            });
            let mut res = Response::empty();
            res.set_status(StatusCode::SWITCHING_PROTOCOLS);
            res.headers_mut()
                .insert(header::UPGRADE, HeaderValue::from_static("h2c"));
            Ok(res)
        } else {
            let handler = self.inner.clone();
            let mut handler = handler.lock().await;
            let req = req.into_grpc_request();

            let grpc_res = handler.call(req).await.map_err(|e| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("grpc call failed: {}", e.into()),
                )
            })?;
            let mut res = Response::empty();
            merge_grpc_response(&mut res, grpc_res).await;

            Ok(res)
        }
    }
}
