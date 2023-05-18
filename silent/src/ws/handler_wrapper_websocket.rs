use crate::ws::handler::websocket_handler;
use crate::ws::upgrade;
use crate::ws::websocket::WebSocket;
use crate::{Handler, Request, Response, Result};
use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::protocol;
use tracing::error;

#[derive(Clone)]
pub struct HandlerWrapperWebSocket<F> {
    pub config: Option<protocol::WebSocketConfig>,
    handler: Option<Arc<F>>,
}

impl<F, Fut> HandlerWrapperWebSocket<F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    F: Fn(WebSocket) -> Fut + Send + Sync + 'static,
{
    pub fn new(config: Option<protocol::WebSocketConfig>) -> Self {
        Self {
            config,
            handler: None,
        }
    }

    pub fn set_handler(mut self, handler: F) -> Self {
        self.handler = Some(Arc::new(handler));
        self
    }
}

#[async_trait]
impl<F, Fut> Handler for HandlerWrapperWebSocket<F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    F: Fn(WebSocket) -> Fut + Send + Sync + 'static,
{
    async fn call(&self, req: Request) -> Result<Response> {
        let res = websocket_handler(&req)?;
        let config = self.config;
        let handler = self.handler.clone();
        tokio::task::spawn(async move {
            match upgrade::on(req).await {
                Ok(upgrade) => {
                    let ws =
                        WebSocket::from_raw_socket(upgrade, protocol::Role::Server, config).await;
                    if let Some(handler) = handler {
                        handler(ws).await;
                    } else if let Err(e) = ws.handle().await {
                        error!("upgrade handle error: {}", e)
                    }
                }
                Err(e) => {
                    error!("upgrade error: {}", e)
                }
            }
        });
        Ok(res)
    }
}
