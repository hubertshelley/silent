use crate::ws::message::Message;
use crate::{Result, SilentError};
use async_trait::async_trait;
use futures_util::sink::{Sink, SinkExt};
use futures_util::stream::{Stream, StreamExt};
use futures_util::{future, ready};
use hyper::upgrade;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tungstenite::tungstenite::protocol;
use tokio_tungstenite::WebSocketStream;
use tracing::debug;

pub(crate) struct WebSocket {
    upgrade: WebSocketStream<upgrade::Upgraded>,
}

impl WebSocket {
    #[inline]
    pub(crate) async fn from_raw_socket(
        upgraded: upgrade::Upgraded,
        role: protocol::Role,
        config: Option<protocol::WebSocketConfig>,
    ) -> Self {
        Self {
            upgrade: WebSocketStream::from_raw_socket(upgraded, role, config).await,
        }
    }

    /// Receive another message.
    ///
    /// Returns `None` if the stream has closed.
    pub async fn recv(&mut self) -> Option<Result<Message>> {
        self.next().await
    }

    /// Send a message.
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

#[async_trait]
pub(crate) trait WSHandlerTrait {
    async fn handle(&mut self) -> Result<()>;
}

#[async_trait]
impl WSHandlerTrait for WebSocket {
    async fn handle(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                op_message = self.recv() => {
                    match op_message{
                        Some(Ok(message)) => {
                            println!("ws recv: {:?}", message);
                            self.send(message).await?;
                        }
                        Some(Err(e)) => {
                            println!("ws recv error: {}", e);
                        }
                        None => {
                            println!("ws finished");
                            break;
                        }
                    }
                }
            }
        }
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
