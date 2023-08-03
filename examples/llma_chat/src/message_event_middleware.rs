use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

type SenderMap = RwLock<HashMap<String, UnboundedSender<String>>>;

pub(crate) static ONLINE_MAP: Lazy<SenderMap> = Lazy::new(SenderMap::default);

#[async_trait::async_trait]
pub(crate) trait EventKeyExt: Send + Sync + 'static {
    async fn set_sender(&self, sender: UnboundedSender<String>);
    async fn send(&self, msg: String);
}

#[async_trait::async_trait]
impl EventKeyExt for String {
    async fn set_sender(&self, sender: UnboundedSender<String>) {
        ONLINE_MAP.write().await.insert(self.clone(), sender);
    }
    async fn send(&self, msg: String) {
        let users = ONLINE_MAP.read().await;
        if let Some(sender) = users.get(self) {
            sender.send(msg).unwrap();
        }
    }
}
//
// pub(crate) struct MessageEventMiddleware {
//     pub(crate) sender_map: String,
// }
//
// impl MessageEventMiddleware {
//     pub(crate) fn new() -> Self {
//         Self {
//             sender_map: "".to_string(),
//         }
//     }
// }
//
// #[async_trait::async_trait]
// impl MiddleWareHandler for MessageEventMiddleware {
//     async fn pre_request(&self, req: &mut Request, _res: &mut Response) -> Result<()> {
//         let m = req.extensions_mut().insert(self.sender_map.clone());
//         info!("pre_request: {:?}", m);
//         Ok(())
//     }
// }
