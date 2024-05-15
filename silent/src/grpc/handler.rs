use crate::{Handler, Response, SilentError};
use async_trait::async_trait;
use http::{header, Method, StatusCode};
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
    async fn call(&self, req: crate::Request) -> crate::Result<Response> {
        if Method::CONNECT == req.method() {
            // println!("req body: {:?}", req.body());
            // let on_upgrade = req.extensions_mut().get::<OnUpgrade>().cloned();
            // // let conn = match req.extensions_mut().remove::<OnUpgrade>() {
            // let conn = if let Some(conn) = on_upgrade {
            //     conn
            // } else {
            //     return Err(SilentError::business_error(
            //         StatusCode::INTERNAL_SERVER_ERROR,
            //         format!("error during upgrade: {}", "no on_upgrade"),
            //     ));
            // };
            // // let conn = match hyper::upgrade::on(req).await {
            // let conn = match conn.await {
            //     Ok(conn) => conn,
            //     Err(e) => {
            //         eprintln!("error during upgrade: {}", e);
            //         return Err(SilentError::business_error(
            //             StatusCode::INTERNAL_SERVER_ERROR,
            //             format!("error during upgrade: {}", e),
            //         ));
            //     }
            // };
            // tokio::spawn(async move {
            //     let http = hyper::server::conn::http2::Builder::new(TokioExecutor::new());
            //     match http
            //         .serve_connection(conn, Http2ServiceHandler::new(handler))
            //         .await
            //     {
            //         Ok(_) => eprintln!("finished gracefully"),
            //         Err(err) => println!("ERROR: {err}"),
            //     }
            // });
            // let mut res = silent::Response::empty()
            //     .set_header(header::CONTENT_TYPE, "application/grpc".parse().unwrap());
            Ok(Response::empty())
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
