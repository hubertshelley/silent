// Copyright (c) 2018-2020 Sean McArthur
// Licensed under the MIT license http://opensource.org/licenses/MIT
//
// port from https://github.com/seanmonstar/warp/blob/master/examples/sse_chat.rs

use std::collections::HashMap;
use std::result::Result as IoResult;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures_util::{future, FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use silent::prelude::*;
use silent::sse::{self, keep_alive};

type Users = Mutex<HashMap<usize, mpsc::UnboundedSender<Message>>>;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);
static ONLINE_USERS: Lazy<Users> = Lazy::new(Users::default);

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();

    let route = Route::new("").get(index).append(
        Route::new("chat")
            .handler(
                Method::GET,
                HandlerWrapperResponse::new(user_connected).arc(),
            )
            .append(
                Route::new("<id:int>")
                    .handler(Method::POST, HandlerWrapperResponse::new(chat_send).arc()),
            ),
    );

    Server::new()
        .bind("0.0.0.0:8001".parse().unwrap())
        .bind_route(route)
        .run();
}

#[derive(Debug)]
enum Message {
    UserId(usize),
    Reply(String),
}

async fn chat_send(req: Request) -> Result<Response> {
    let my_id = req.get_path_params::<i32>("id")?;
    let msg = "hello";
    user_message(my_id as usize, msg);
    Ok(Response::empty())
}

fn get_connected(_req: Request) -> impl Stream<Item = Result<sse::Event>> + Send + 'static {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    info!("new chat user: {}", my_id);

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the event source...
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    tx.send(Message::UserId(my_id))
        // rx is right above, so this cannot fail
        .unwrap();

    // Save the sender in our list of connected users.
    ONLINE_USERS.lock().insert(my_id, tx);

    // Convert messages into Server-Sent Events and returns resulting stream.
    let stream = rx.map(|msg| match msg {
        Message::UserId(my_id) => Ok(sse::Event::default().event("user").data(my_id.to_string())),
        Message::Reply(reply) => Ok(sse::Event::default().data(reply)),
    });
    stream
}

async fn user_connected(_req: Request) -> Result<Response> {
    let stream = get_connected(_req);
    // SseKeepAlive::new(stream).streaming(res).ok();
    let mut res = Response::empty()
        .set_header(
            HeaderName::from_static("Content-Type"),
            HeaderValue::from_static("text/event-stream"),
        )
        .set_header(
            HeaderName::from_static("Cache-Control"),
            HeaderValue::from_static("no-cache"),
        )
        .set_header(
            HeaderName::from_static("Connection"),
            HeaderValue::from_static("keep-alive"),
        )
        .set_header(
            HeaderName::from_static("Access-Control-Allow-Origin"),
            HeaderValue::from_static("*"),
        );
    res.set_status(StatusCode::PARTIAL_CONTENT);
    // res.set_body(stream_body(stream));
    let event_stream = sse::keep_alive().stream(stream);
    let body_stream = event_stream
        .map_err(|error| {
            // FIXME: error logging
            error!("sse stream error: {}", error);
            SilentError::BusinessError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: "sse::keep error".to_string(),
            }
        })
        .into_stream()
        .and_then(|event| future::ready(Ok(event.to_string())));
    res.set_body(stream_body(body_stream));
    Ok(res)
}

fn user_message(my_id: usize, msg: &str) {
    let new_msg = format!("<User#{my_id}>: {msg}");

    // New message from this user, send it to everyone else (except same uid)...
    //
    // We use `retain` instead of a for loop so that we can reap any user that
    // appears to have disconnected.
    ONLINE_USERS.lock().retain(|uid, tx| {
        if my_id == *uid {
            // don't send to same user, but do retain
            true
        } else {
            // If not `is_ok`, the SSE stream is gone, and so don't retain
            tx.send(Message::Reply(new_msg.clone())).is_ok()
        }
    });
}

async fn index<'a>(_req: Request) -> Result<&'a str> {
    Ok(INDEX_HTML)
}

static INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
    <head>
        <title>SSE Chat</title>
    </head>
    <body>
        <h1>SSE Chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="msg" />
        <button type="button" id="submit">Send</button>
        <script>
        const chat = document.getElementById('chat');
        const msg = document.getElementById('msg');
        const submit = document.getElementById('submit');
        let sse = new EventSource(`http://${location.host}/chat`);
        sse.onopen = function() {
            chat.innerHTML = "<p><em>Connected!</em></p>";
        }
        var userId;
        sse.addEventListener("user", function(msg) {
            userId = msg.data;
        });
        sse.onmessage = function(msg) {
            showMessage(msg.data);
        };
        document.getElementById('submit').onclick = function() {
            var msg = text.value;
            var xhr = new XMLHttpRequest();
            xhr.open("POST", `${uri}/${user_id}`, true);
            xhr.send(msg);
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
