mod llm_middleware;
mod message_event_middleware;

use crate::llm_middleware::{LLMMiddleware, Llm};
use crate::message_event_middleware::ONLINE_MAP;
use silent::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

// #[derive(Deserialize, Debug)]
// struct ChatRequest {
//     code: String,
//     ask_str: String,
// }
//
// #[derive(Deserialize, Debug)]
// struct CodeRequest {
//     code: String,
// }

// async fn chat_handler(mut req: Request) -> Result<String> {
//     let ask_str = req.json_parse::<ChatRequest>().await?;
//     match req.extensions().get::<Llm>() {
//         None => {
//             Err(SilentError::BusinessError {
//                 code: StatusCode::INTERNAL_SERVER_ERROR,
//                 msg: "llm not found".to_string(),
//             })
//         }
//         Some(llm) => {
//             llm.chat(&ask_str.ask_str)
//         }
//     }
// }

async fn on_connect(
    parts: Arc<RwLock<WebSocketParts>>,
    sender: UnboundedSender<Message>,
) -> Result<()> {
    println!("on_connect");
    let mut parts = parts.write().await;
    let code = parts.params_mut().get("code").cloned();
    info!("on_connect: {:?}", code);
    info!("on_connect: {:?}", parts);
    match code {
        None => {
            error!("code not found");
            Err(SilentError::BusinessError {
                code: StatusCode::BAD_REQUEST,
                msg: "code not found".to_string(),
            })
        }
        Some(code) => {
            info!("on_connect: {}", code);
            info!("{:?}", parts);
            let sender_map = parts.extensions_mut().insert(code.clone());
            let (tx_sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
            ONLINE_MAP.write().await.insert(code.clone(), tx_sender);
            tokio::spawn(async move {
                while let Some(msg) = receiver.recv().await {
                    sender.send(Message::text(msg)).unwrap();
                }
            });
            info!("on_conected {:?}", sender_map);
            // info!("sender_map: {:?}", sender_map);
            Ok(())
        }
    }
}

async fn on_receive(message: Message, parts: Arc<RwLock<WebSocketParts>>) -> Result<()> {
    let ask_str = message.to_str()?;
    println!("ask_str: {}", ask_str);
    let parts = parts.write().await;
    let code = parts.extensions().get::<String>().unwrap();
    info!("code: {}", code);
    match ONLINE_MAP.read().await.get(code) {
        None => {
            error!("sender not found");
        }
        Some(sender) => {
            warn!("sender: {:?}", sender);
            match parts.extensions().get::<Llm>() {
                None => {
                    error!("llm not found");
                }
                Some(llm) => {
                    let res = llm.chat(ask_str, sender).unwrap();
                    info!("res: {}", res);
                }
            }
        }
    }
    Ok(())
}

async fn on_close(parts: Arc<RwLock<WebSocketParts>>) {
    let parts = parts.write().await;
    let code = parts.extensions().get::<String>().unwrap();
    let sender_map = parts
        .extensions()
        .get::<Arc<RwLock<HashMap<String, UnboundedSender<String>>>>>()
        .unwrap();
    sender_map.write().await.remove(code);
}

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let llm = LLMMiddleware::new();
    // let message_event_middleware = MessageEventMiddleware::new();
    let route = Route::new("")
        .append(
            Route::new("api")
                .hook(llm)
                // .hook(message_event_middleware)
                .append(
                    Route::new("chat").ws(
                        None,
                        WebSocketHandler::new()
                            .on_connect(on_connect)
                            .on_send(|msg, _| async {
                                info!("send msg: {}", msg.to_str()?);
                                Ok(msg)
                            })
                            .on_receive(on_receive)
                            .on_close(on_close),
                    ),
                ),
        )
        .append(Route::new("<path:**>").handler(
            Method::GET,
            Arc::new(static_handler("examples/llma_chat/static")),
        ));
    Server::new().run(route);
}
