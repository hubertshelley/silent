use crate::header::{CACHE_CONTROL, CONTENT_TYPE};
use crate::prelude::stream_body;
use crate::sse::{keep_alive, SSEEvent};
use crate::{log, HeaderValue, Response, Result, SilentError, StatusCode};
use futures_util::{future, Stream, TryStreamExt};

pub fn sse_reply<S>(stream: S) -> Response
where
    S: Stream<Item = Result<SSEEvent>> + Send + 'static,
{
    let event_stream = keep_alive().stream(stream);
    let body_stream = event_stream
        .map_err(|error| {
            // FIXME: error logging
            log::error!("sse stream error: {}", error);
            SilentError::BusinessError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: "sse::keep error".to_string(),
            }
        })
        .into_stream()
        .and_then(|event| future::ready(Ok(event.to_string())));

    let mut res = Response::empty();
    res.set_body(stream_body(body_stream));
    // Set appropriate content type
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
    // Disable response body caching
    res.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    res
}
