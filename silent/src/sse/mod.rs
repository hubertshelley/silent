//! Server-Sent Events (SSE)
//!
//! # Example
//!
//! ```
//!
//! use std::time::Duration;
//! use std::convert::Infallible;
//! use silent::{HandlerWrapperResponse, Method, prelude::Route, Response, sse::Event};
//! use futures_util::{stream::iter, Stream};
//! use silent::prelude::HandlerGetter;
//!
//! fn sse_events() -> impl Stream<Item = Result<Event, Infallible>> {
//!     iter(vec![
//!         Ok(Event::default().data("unnamed event")),
//!         Ok(
//!             Event::default().event("chat")
//!             .data("chat message")
//!         ),
//!         Ok(
//!             Event::default().id(13.to_string())
//!             .event("chat")
//!             .data("other chat message\nwith next line")
//!             .retry(Duration::from_millis(5000))
//!         ),
//!     ])
//! }
//!
//! let route = Route::new("push-notifications")
//!     .handler(Method::GET, HandlerWrapperResponse::new(|req| async {
//!         let res = Response::empty();
//!         res.set_body(warp::sse::keep_alive().stream(sse_events()));
//!         res
//!     }).arc());
//! ```
//!
//! Each field already is event which can be sent to client.
//! The events with multiple fields can be created by combining fields using tuples.
//!
//! See also the [EventSource](https://developer.mozilla.org/en-US/docs/Web/API/EventSource) API,
//! which specifies the expected behavior of Server Sent Events.
//!

mod event;
mod keep_alive;

pub use event::Event;
pub use keep_alive::{keep_alive, KeepAlive};
