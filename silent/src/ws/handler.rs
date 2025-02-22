use crate::headers::{Connection, HeaderMapExt, SecWebsocketAccept, SecWebsocketKey, Upgrade};
use crate::{Request, Response, Result, SilentError, StatusCode, header};

pub fn websocket_handler(req: &Request) -> Result<Response> {
    let mut res = Response::empty();
    let req_headers = req.headers();
    if !req_headers.contains_key(header::UPGRADE) {
        return Err(SilentError::BusinessError {
            code: StatusCode::BAD_REQUEST,
            msg: "bad request: not upgrade".to_string(),
        });
    }
    if !req_headers
        .get(header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_lowercase() == "websocket")
        .unwrap_or(false)
    {
        return Err(SilentError::BusinessError {
            code: StatusCode::BAD_REQUEST,
            msg: "bad request: not websocket".to_string(),
        });
    }
    let sec_ws_key = if let Some(key) = req_headers.typed_get::<SecWebsocketKey>() {
        key
    } else {
        return Err(SilentError::BusinessError {
            code: StatusCode::BAD_REQUEST,
            msg: "bad request: sec_websocket_key is not exist in request headers".to_string(),
        });
    };
    res.set_status(StatusCode::SWITCHING_PROTOCOLS);
    res.headers.typed_insert(Connection::upgrade());
    res.headers.typed_insert(Upgrade::websocket());
    res.headers
        .typed_insert(SecWebsocketAccept::from(sec_ws_key));
    Ok(res)
}
