use crate::grpc::service::GrpcService;
use crate::{Handler, Response, SilentError};
use async_trait::async_trait;
use http::{header, HeaderValue, StatusCode};
use hyper::upgrade::OnUpgrade;
use hyper_util::rt::TokioExecutor;
use tower_service::Service;
use tracing::error;

#[derive(Clone)]
pub struct GrpcHandler(axum::Router<()>);

impl From<axum::Router<()>> for GrpcHandler {
    fn from(router: axum::Router<()>) -> Self {
        Self(router)
    }
}

#[async_trait]
impl Handler for GrpcHandler {
    async fn call(&self, mut req: crate::Request) -> crate::Result<Response> {
        if let Some(on_upgrade) = req.extensions_mut().remove::<OnUpgrade>() {
            let handler = self.0.clone();
            tokio::spawn(async move {
                let conn = on_upgrade.await;
                if conn.is_err() {
                    eprintln!("upgrade error: {:?}", conn.err());
                    return;
                }
                let upgraded_io = conn.unwrap();

                let http = hyper::server::conn::http2::Builder::new(TokioExecutor::new());
                match http
                    .serve_connection(upgraded_io, GrpcService::new(handler))
                    .await
                {
                    Ok(_) => eprintln!("finished gracefully"),
                    Err(err) => println!("ERROR: {err}"),
                }
            });
            let mut res = Response::empty();
            res.set_status(StatusCode::SWITCHING_PROTOCOLS);
            res.headers_mut()
                .insert(header::UPGRADE, HeaderValue::from_static("h2c"));
            Ok(res)
        } else {
            let mut handler = self.0.clone();
            let req = req.into_http();

            let axum_res = handler.call(req).await.map_err(|e| {
                error!(error = ?e, "call axum router failed: {}", e);
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("call axum router failed: {}", e),
                )
            })?;
            let mut res = Response::empty();
            res.merge_axum(axum_res).await;
            res.headers_mut()
                .insert(header::CONTENT_TYPE, "application/grpc".parse().unwrap());
            res.headers_mut().insert(
                header::HeaderName::from_static("grpc-status"),
                "0".parse().unwrap(),
            );

            Ok(res)
        }
    }
}
