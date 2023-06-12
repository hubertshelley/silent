use crate::log::error;
use crate::sse::Event;
use crate::{SilentError, StatusCode};
use futures_util::{Stream, TryStream};
use pin_project::pin_project;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time;
use tokio::time::Sleep;

/// Configure the interval between keep-alive messages, the content
/// of each message, and the associated stream.
#[derive(Debug)]
pub struct KeepAlive {
    comment_text: Cow<'static, str>,
    max_interval: Duration,
}

impl KeepAlive {
    /// Customize the interval between keep-alive messages.
    ///
    /// Default is 15 seconds.
    pub fn interval(mut self, time: Duration) -> Self {
        self.max_interval = time;
        self
    }

    /// Customize the text of the keep-alive message.
    ///
    /// Default is an empty comment.
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.comment_text = text.into();
        self
    }

    /// Wrap an event stream with keep-alive functionality.
    ///
    /// See [`keep_alive`](keep_alive) for more.
    pub fn stream<S>(
        self,
        event_stream: S,
    ) -> impl TryStream<Ok = Event, Error = impl StdError + Send + Sync + 'static> + Send + 'static
    where
        S: TryStream<Ok = Event> + Send + 'static,
        S::Error: StdError + Send + Sync + 'static,
    {
        let alive_timer = time::sleep(self.max_interval);
        SseKeepAlive {
            event_stream,
            comment_text: self.comment_text,
            max_interval: self.max_interval,
            alive_timer,
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
struct SseKeepAlive<S> {
    #[pin]
    event_stream: S,
    comment_text: Cow<'static, str>,
    max_interval: Duration,
    #[pin]
    alive_timer: Sleep,
}

/// Keeps event source connection alive when no events sent over a some time.
///
/// Some proxy servers may drop HTTP connection after a some timeout of inactivity.
/// This function helps to prevent such behavior by sending comment events every
/// `keep_interval` of inactivity.
///
/// By default the comment is `:` (an empty comment) and the time interval between
/// events is 15 seconds. Both may be customized using the builder pattern
/// as shown below.
///
/// ```
/// use std::time::Duration;
/// use std::convert::Infallible;
/// use futures_util::StreamExt;
/// use tokio::time::interval;
/// use tokio_stream::wrappers::IntervalStream;
/// use silent::{Filter, Stream, sse::Event};
///
/// // create server-sent event
/// fn sse_counter(counter: u64) ->  Result<Event, Infallible> {
///     Ok(Event::default().data(counter.to_string()))
/// }
///
/// fn main() {
///     let routes = warp::path("ticks")
///         .and(warp::get())
///         .map(|| {
///             let mut counter: u64 = 0;
///             let interval = interval(Duration::from_secs(15));
///             let stream = IntervalStream::new(interval);
///             let event_stream = stream.map(move |_| {
///                 counter += 1;
///                 sse_counter(counter)
///             });
///             // reply using server-sent events
///             let stream = warp::sse::keep_alive()
///                 .interval(Duration::from_secs(5))
///                 .text("thump".to_string())
///                 .stream(event_stream);
///             warp::sse::reply(stream)
///         });
/// }
/// ```
///
/// See [notes](https://www.w3.org/TR/2009/WD-eventsource-20090421/#notes).
pub fn keep_alive() -> KeepAlive {
    KeepAlive {
        comment_text: Cow::Borrowed(""),
        max_interval: Duration::from_secs(15),
    }
}

impl<S> Stream for SseKeepAlive<S>
where
    S: TryStream<Ok = Event> + Send + 'static,
    S::Error: StdError + Send + Sync + 'static,
{
    type Item = Result<Event, SilentError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut pin = self.project();
        match pin.event_stream.try_poll_next(cx) {
            Poll::Pending => match Pin::new(&mut pin.alive_timer).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => {
                    // restart timer
                    pin.alive_timer
                        .reset(tokio::time::Instant::now() + *pin.max_interval);
                    let comment_str = pin.comment_text.clone();
                    let event = Event::default().comment(comment_str);
                    Poll::Ready(Some(Ok(event)))
                }
            },
            Poll::Ready(Some(Ok(event))) => {
                // restart timer
                pin.alive_timer
                    .reset(tokio::time::Instant::now() + *pin.max_interval);
                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(error))) => {
                error!("sse::keep error: {}", error);
                Poll::Ready(Some(Err(SilentError::BusinessError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    msg: "sse::keep error".to_string(),
                })))
            }
        }
    }
}
