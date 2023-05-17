use crate::ws::upgrade;
use crate::ws::websocket::WebSocket;
use crate::{header, Handler, Request, Response, Result, SilentError, StatusCode};
use async_trait::async_trait;
use headers::{Connection, HeaderMapExt, SecWebsocketAccept, SecWebsocketKey, Upgrade};
use tokio_tungstenite::tungstenite::protocol;
use tracing::error;

#[derive(Clone)]
pub struct HandlerWrapperWebSocket {
    pub config: Option<protocol::WebSocketConfig>,
}

impl HandlerWrapperWebSocket {
    pub fn new(config: Option<protocol::WebSocketConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Handler for HandlerWrapperWebSocket {
    async fn call(&self, req: Request) -> Result<Response> {
        let mut res = Response::empty();
        let req_headers = req.headers();
        if !req_headers.contains_key(header::UPGRADE) {
            return Err(SilentError::BusinessError {
                code: StatusCode::BAD_REQUEST,
                msg: "bad request: not upgrade".to_string(),
            });
        }
        if !req_headers
            .get(header::UPGRADE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase() == "websocket")
            .unwrap_or(false)
        {
            return Err(SilentError::BusinessError {
                code: StatusCode::BAD_REQUEST,
                msg: "bad request: not websocket".to_string(),
            });
        }
        let sec_ws_key = if let Some(key) = req_headers.typed_get::<SecWebsocketKey>() {
            key
        } else {
            return Err(SilentError::BusinessError {
                code: StatusCode::BAD_REQUEST,
                msg: "bad request: sec_websocket_key is not exist in request headers".to_string(),
            });
        };
        let config = self.config;
        tokio::task::spawn(async move {
            match upgrade::on(req).await {
                Ok(upgrade) => {
                    let ws =
                        WebSocket::from_raw_socket(upgrade, protocol::Role::Server, config).await;
                    if let Err(e) = ws.handle().await {
                        error!("upgrade handle error: {}", e)
                    };
                }
                Err(e) => {
                    error!("upgrade error: {}", e)
                }
            }
        });
        res.set_status(StatusCode::SWITCHING_PROTOCOLS);
        res.headers_mut().typed_insert(Connection::upgrade());
        res.headers_mut().typed_insert(Upgrade::websocket());
        res.headers_mut()
            .typed_insert(SecWebsocketAccept::from(sec_ws_key));
        Ok(res)
    }
}
