use std::future::Future;
use std::sync::Arc;
use crate::ws::message::Message;
use crate::{Request, Result};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::ws::websocket::WebSocket;

pub struct WebSocketHandler<
    FnOnConnect,
    FnOnSend,
    FnOnSendFut,
    FnOnReceive,
    FnOnNoneResultFut,
    FnOnClose
>
    where
        FnOnConnect: Fn(Request, UnboundedSender<Message>) -> Result<()> + Send + Sync + 'static,
        FnOnSendFut: Future<Output=Result<Message>> + Send + Sync + 'static,
        FnOnNoneResultFut: Future<Output=Result<()>> + Send + Sync + 'static,
        FnOnSend: Fn(Message, &WebSocket) -> FnOnSendFut + Send + Sync + 'static,
        FnOnReceive: Fn(Message, &WebSocket) -> FnOnNoneResultFut + Send + Sync + 'static,
        FnOnClose: Fn(Message, &WebSocket) -> FnOnNoneResultFut + Send + Sync + 'static,
{
    pub(crate) sender: UnboundedSender<Message>,
    pub(crate) receiver: UnboundedReceiver<Message>,
    pub(crate) on_connect: Arc<FnOnConnect>,
    pub(crate) on_send: Arc<FnOnSend>,
    pub(crate) on_receive: Arc<FnOnReceive>,
    pub(crate) on_close: Arc<FnOnClose>,
}
