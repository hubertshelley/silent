use crate::ws::message::Message;
use crate::{Request, Result};
use async_trait::async_trait;

#[async_trait]
pub trait WebSocketHandler: Send + Sync + 'static {
    async fn on_connect(&self, _req: &Request) -> Result<()> {
        Ok(())
    }
    async fn on_receive(&self, _message: Message) -> Result<()> {
        Ok(())
    }
    async fn on_send(&self, _message: &Message) -> Result<()> {
        Ok(())
    }
    async fn on_close(&self) {}
    async fn get_sender(&self) -> Result<tokio::sync::mpsc::UnboundedSender<Message>>;
    async fn set_sender(&mut self, _sender: tokio::sync::mpsc::UnboundedSender<Message>);
}
