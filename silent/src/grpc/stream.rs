use crate::prelude::ReqBody;
use crate::SilentError;
use bytes::Bytes;
use futures::Stream;
use http::request::Parts;
use http::StatusCode;
use http_body::Body;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use pin_project_lite::pin_project;
use std::task::Poll;
use tokio::runtime::Handle;
use tower_service::Service;
use tracing::error;

pin_project! {
    pub(crate) struct GrpcStream{
        #[pin]
        incoming: Incoming,
        handler: axum::Router<()>,
        parts: Parts,
    }
}

impl GrpcStream {
    pub fn new(incoming: Incoming, handler: axum::Router<()>, parts: Parts) -> Self {
        Self {
            incoming,
            handler,
            parts,
        }
    }
}

impl Stream for GrpcStream {
    type Item = crate::Result<Bytes>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut stream = self.project();
        match stream.incoming.as_mut().poll_frame(cx) {
            Poll::Ready(Some(Ok(chunk))) => match chunk.into_data() {
                Ok(chunk) => {
                    let body = ReqBody::Once(chunk);
                    let req = hyper::Request::from_parts(stream.parts.clone(), body);
                    let mut handler = stream.handler.clone();
                    tokio::task::block_in_place(move || {
                        Handle::current().block_on(async move {
                            let axum_res = handler.call(req).await.map_err(|e| {
                                error!(error = ?e, "call axum router failed: {}", e);
                                SilentError::business_error(
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    format!("call axum router failed: {}", e),
                                )
                            })?;
                            let mut body = axum_res.into_body();
                            if let Some(Ok(chunk)) = body.frame().await {
                                if let Ok(chunk) = chunk.into_data() {
                                    eprintln!("stream chunk: {:#?}", chunk);
                                    Poll::Ready(Some(Ok(chunk)))
                                } else {
                                    Poll::Ready(None)
                                }
                            } else {
                                Poll::Ready(None)
                            }
                        })
                    })
                }
                Err(_) => Poll::Ready(None),
            },
            Poll::Pending => Poll::Pending,
            _ => Poll::Ready(Some(Ok(Bytes::new()))),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size_hint = self.incoming.size_hint();
        (
            size_hint.lower() as usize,
            size_hint.upper().map(|x| x as usize),
        )
    }
}
