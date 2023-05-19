// Copyright (c) 2018-2020 Sean McArthur
// Licensed under the MIT license http://opensource.org/licenses/MIT
//
// port from https://github.com/seanmonstar/warp/blob/master/examples/websocket_chat.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use tokio::sync::{mpsc, RwLock};

use silent::prelude::*;

type Users = RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);
static ONLINE_USERS: Lazy<Users> = Lazy::new(Users::default);

fn main() {
    logger::fmt().init();
    let route = Route::new("")
        .get(index)
        .append(Route::new("chat").ws(None, handle_socket));
    Server::new().bind_route(route).run();
}

async fn on_connect(
    parts: Arc<RwLock<WebSocketParts>>,
    sender: mpsc::UnboundedSender<Message>,
) -> Result<()> {
    let mut parts = parts.write().await;
    println!("{:?}", parts);
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    info!("new chat user: {}", my_id);
    parts.extensions_mut().insert(my_id);
    sender
        .send(Message::text(format!("Hello User#{my_id}")))
        .unwrap();
    ONLINE_USERS.write().await.insert(my_id, sender);
    Ok(())
}

async fn on_send(message: Message, _parts: Arc<RwLock<WebSocketParts>>) -> Result<Message> {
    println!("on_send: {:?}", message);
    Ok(message)
}

async fn on_receive(message: Message, parts: Arc<RwLock<WebSocketParts>>) -> Result<()> {
    let parts = parts.read().await;
    let my_id = parts.extensions().get::<usize>().unwrap();
    println!("on_receive: {:?}", message);
    let msg = if let Ok(s) = message.to_str() {
        s
    } else {
        return Err(SilentError::BusinessError {
            code: StatusCode::BAD_REQUEST,
            msg: "invalid message".to_string(),
        });
    };
    let message = Message::text(format!("<User#{my_id}>: {msg}"));
    for (uid, tx) in ONLINE_USERS.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(message.clone()) {}
        }
    }
    Ok(())
}

async fn on_close(parts: Arc<RwLock<WebSocketParts>>) {
    let parts = parts.read().await;
    let my_id = parts.extensions().get::<usize>().unwrap();
    eprintln!("good bye user: {my_id}");
    // Stream closed up, so remove from the user list
    ONLINE_USERS.write().await.remove(my_id);
}

async fn handle_socket(ws: WebSocket) {
    let (parts, ws) = ws.into_parts();
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    // Use a counter to assign a new unique ID for this user.

    let (tx, mut rx) = mpsc::unbounded_channel();
    on_connect(parts.clone(), tx.clone()).await.unwrap();
    let sender_parts = parts.clone();
    let receiver_parts = parts;

    let fut = async move {
        while let Some(message) = rx.recv().await {
            let message = on_send(message.clone(), sender_parts.clone())
                .await
                .unwrap();
            println!("before: {:?}", message);
            user_ws_tx.send(message).await.unwrap();
        }
    };
    tokio::task::spawn(fut);
    let fut = async move {
        while let Some(message) = user_ws_rx.next().await {
            if let Ok(message) = message {
                if message.is_close() {
                    break;
                }
                if on_receive(message, receiver_parts.clone()).await.is_err() {
                    break;
                }
            }
        }

        on_close(receiver_parts).await;
    };
    tokio::task::spawn(fut);
}

async fn index<'a>(_res: Request) -> Result<&'a str> {
    Ok(INDEX_HTML)
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
    <head>
        <title>WS Chat</title>
    </head>
    <body>
        <h1>WS Chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="submit">Submit</button>
        <script>
            const chat = document.getElementById('chat');
            const msg = document.getElementById('msg');
            const submit = document.getElementById('submit');
            const ws = new WebSocket(`ws://${location.host}/chat`);

            ws.onopen = function() {
                chat.innerHTML = '<p><em>Connected!</em></p>';
            };

            ws.onmessage = function(msg) {
                showMessage(msg.data);
            };

            ws.onclose = function() {
                chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
            };

            submit.onclick = function() {
                const msg = text.value;
                ws.send(msg);
                text.value = '';

                showMessage('<You>: ' + msg);
            };
            function showMessage(data) {
                const line = document.createElement('p');
                line.innerText = data;
                chat.appendChild(line);
            }
        </script>
    </body>
</html>
"#;
