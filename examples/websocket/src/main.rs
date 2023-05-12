use async_trait::async_trait;
use futures_util::sink::{Sink, SinkExt};
use futures_util::stream::{Stream, StreamExt};
use futures_util::{future, ready};
use silent::prelude::*;
use std::fmt;
use std::fmt::Formatter;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio_tungstenite::tungstenite::protocol;
use tokio_tungstenite::WebSocketStream;

fn main() {
    logger::fmt().init();
    let route = Route::new("")
        .get(show_form)
        .append(Route::new("ws").insert_handler(Method::GET, Arc::new(WSHandler { config: None })));
    Server::new().bind_route(route).run();
}

struct WSHandler {
    config: Option<protocol::WebSocketConfig>,
}

impl WSHandler2 {
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
    #[inline]
    pub async fn close(mut self) -> Result<()> {
        future::poll_fn(|cx| Pin::new(&mut self).poll_close(cx)).await
    }
}

impl Sink<Message> for WSHandler2 {
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

#[derive(Eq, PartialEq, Clone)]
pub struct Message {
    inner: protocol::Message,
}

impl fmt::Debug for Message {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

struct WSHandler2 {
    upgrade: WebSocketStream<upgrade::Upgraded>,
}

#[async_trait]
trait WSHandlerTrait {
    async fn handle(&mut self) -> Result<()>;
}

#[async_trait]
impl WSHandlerTrait for WSHandler2 {
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
                            e.trace();
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

impl Stream for WSHandler2 {
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

#[async_trait]
impl Handler for WSHandler {
    async fn call(&self, mut req: Request) -> Result<Response> {
        let mut res = Response::empty();
        if !req.headers().contains_key(header::UPGRADE) {
            return Err(SilentError::BusinessError {
                code: StatusCode::BAD_REQUEST,
                msg: "bad request: not upgrade".to_string(),
            });
        }
        let config = self.config;
        tokio::task::spawn(async move {
            match upgrade::on(req.req_mut()).await {
                Ok(upgraded) => {
                    if let Err(e) =
                        WSHandler2::from_raw_socket(upgraded, protocol::Role::Server, config)
                            .await
                            .handle()
                            .await
                    {
                        eprintln!("server foobar io error: {}", e)
                    };
                }
                Err(e) => eprintln!("upgrade error: {}", e),
            }
        });
        res.set_status(StatusCode::SWITCHING_PROTOCOLS);
        Ok(res)
    }
}

async fn show_form(_req: Request) -> Result<&'static str> {
    Ok(r#"<!DOCTYPE html>
<html>
    <head>
        <title>WS</title>
    </head>
    <body>
        <h1>WS</h1>
        <div id="status">
            <p><em>Connecting...</em></p>
        </div>
        <script>
            const status = document.getElementById('status');
            const msg = document.getElementById('msg');
            const submit = document.getElementById('submit');
            const ws = new WebSocket(`ws://${location.host}/ws?id=123&name=dddf`);

            ws.onopen = function() {
                status.innerHTML = '<p><em>Connected!</em></p>';
            };
        </script>
    </body>
</html>
"#)
}
