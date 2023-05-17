use crate::ws::message::Message;
use crate::ws::upgrade::{Upgraded, WebSocketParts};
use crate::{Result, SilentError};
use futures_util::sink::{Sink, SinkExt};
use futures_util::stream::{Stream, StreamExt};
use futures_util::{future, ready};
use hyper::upgrade::Upgraded as HyperUpgraded;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::protocol;
use tokio_tungstenite::WebSocketStream;
use tracing::{debug, error};

pub(crate) struct WebSocket {
    parts: WebSocketParts,
    upgrade: WebSocketStream<HyperUpgraded>,
}

impl WebSocket {
    #[inline]
    pub(crate) async fn from_raw_socket(
        upgraded: Upgraded,
        role: protocol::Role,
        config: Option<protocol::WebSocketConfig>,
    ) -> Self {
        let (parts, upgraded) = upgraded.into_parts();
        Self {
            parts,
            upgrade: WebSocketStream::from_raw_socket(upgraded, role, config).await,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (WebSocketParts, Self) {
        (self.parts.clone(), self)
    }

    /// Receive another message.
    ///
    /// Returns `None` if the stream has closed.
    #[allow(dead_code)]
    pub async fn recv(&mut self) -> Option<Result<Message>> {
        self.next().await
    }

    /// Send a message.
    #[allow(dead_code)]
    pub async fn send(&mut self, msg: Message) -> Result<()> {
        self.upgrade.send(msg.inner).await.map_err(|e| e.into())
    }

    /// Gracefully close this websocket.
    #[allow(dead_code)]
    #[inline]
    pub async fn close(mut self) -> Result<()> {
        future::poll_fn(|cx| Pin::new(&mut self).poll_close(cx)).await
    }
}

impl Sink<Message> for WebSocket {
    type Error = SilentError;

    #[inline]
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.upgrade)
            .poll_ready(cx)
            .map_err(|e| e.into())
    }

    #[inline]
    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<()> {
        Pin::new(&mut self.upgrade)
            .start_send(item.inner)
            .map_err(|e| e.into())
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut self.upgrade)
            .poll_flush(cx)
            .map_err(|e| e.into())
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut self.upgrade)
            .poll_close(cx)
            .map_err(|e| e.into())
    }
}

impl WebSocket {
    pub(crate) async fn handle(self) -> Result<()> {
        let (parts, ws) = self.into_parts();
        let parts = Arc::new(RwLock::new(parts));
        let (mut user_ws_tx, mut user_ws_rx) = ws.split();
        let (_tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender_parts = parts.clone();
        let receiver_parts = parts;
        let fut = async move {
            let _sender_parts = sender_parts.clone();
            while let Some(message) = rx.recv().await {
                // if let Some(on_send) = on_send {
                //     let message = on_send(message.clone(), sender_parts).await.unwrap();
                //     user_ws_tx.send(message).await.unwrap();
                // } else {
                //     user_ws_tx.send(message).await.unwrap();
                // }
                user_ws_tx.send(message).await.unwrap();
            }
        };
        tokio::task::spawn(fut);
        let fut = async move {
            let _receiver_parts = receiver_parts.clone();
            while let Some(message) = user_ws_rx.next().await {
                match message {
                    Ok(_message) => {
                        // if let Some(on_receive) = on_receive {
                        //     on_receive(message, receiver_parts).await.unwrap();
                        // }
                    }
                    Err(e) => {
                        error!("ws recv error: {}", e);
                    }
                }
            }
        };
        tokio::task::spawn(fut);
        Ok(())
    }
}

impl Stream for WebSocket {
    type Item = Result<Message>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match ready!(Pin::new(&mut self.upgrade).poll_next(cx)) {
            Some(Ok(item)) => Poll::Ready(Some(Ok(Message { inner: item }))),
            Some(Err(e)) => {
                debug!("websocket poll error: {}", e);
                Poll::Ready(Some(Err(e.into())))
            }
            None => {
                debug!("websocket closed");
                Poll::Ready(None)
            }
        }
    }
}
