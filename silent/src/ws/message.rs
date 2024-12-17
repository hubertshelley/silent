use crate::{Result, SilentError};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;
use tokio_tungstenite::tungstenite::protocol;
use tokio_tungstenite::tungstenite::protocol::frame::{Payload, Utf8Payload};

#[derive(Eq, PartialEq, Clone)]
pub struct Message {
    pub(crate) inner: protocol::Message,
}

impl Deref for Message {
    type Target = protocol::Message;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Message {
    /// Construct a new Text `Message`.
    #[inline]
    pub fn text<S: Into<Utf8Payload>>(s: S) -> Message {
        Message {
            inner: protocol::Message::text(s),
        }
    }

    /// Construct a new Binary `Message`.
    #[inline]
    pub fn binary<V: Into<Payload>>(v: V) -> Message {
        Message {
            inner: protocol::Message::binary(v.into()),
        }
    }

    /// Construct a new Ping `Message`.
    #[inline]
    pub fn ping<V: Into<Payload>>(v: V) -> Message {
        Message {
            inner: protocol::Message::Ping(v.into()),
        }
    }

    /// Construct a new pong `Message`.
    #[inline]
    pub fn pong<V: Into<Payload>>(v: V) -> Message {
        Message {
            inner: protocol::Message::Pong(v.into()),
        }
    }

    /// Construct the default Close `Message`.
    #[inline]
    pub fn close() -> Message {
        Message {
            inner: protocol::Message::Close(None),
        }
    }

    /// Construct a Close `Message` with a code and reason.
    #[inline]
    pub fn close_with(code: impl Into<u16>, reason: impl Into<Cow<'static, str>>) -> Message {
        Message {
            inner: protocol::Message::Close(Some(protocol::frame::CloseFrame {
                code: protocol::frame::coding::CloseCode::from(code.into()),
                reason: reason.into(),
            })),
        }
    }

    /// Returns true if this message is a Text message.
    #[inline]
    pub fn is_text(&self) -> bool {
        self.inner.is_text()
    }

    /// Returns true if this message is a Binary message.
    #[inline]
    pub fn is_binary(&self) -> bool {
        self.inner.is_binary()
    }

    /// Returns true if this message is a Close message.
    #[inline]
    pub fn is_close(&self) -> bool {
        self.inner.is_close()
    }

    /// Returns true if this message is a Ping message.
    #[inline]
    pub fn is_ping(&self) -> bool {
        self.inner.is_ping()
    }

    /// Returns true if this message is a Pong message.
    #[inline]
    pub fn is_pong(&self) -> bool {
        self.inner.is_pong()
    }

    /// Try to get the close frame (close code and reason).
    #[inline]
    pub fn close_frame(&self) -> Option<(u16, &str)> {
        if let protocol::Message::Close(Some(ref close_frame)) = self.inner {
            Some((close_frame.code.into(), close_frame.reason.as_ref()))
        } else {
            None
        }
    }

    /// Try to get a reference to the string text, if this is a Text message.
    #[inline]
    pub fn to_str(&self) -> Result<&str> {
        self.inner
            .to_text()
            .map_err(|_| SilentError::WsError("not a text message".into()))
    }

    /// Returns the bytes of this message, if the message can contain data.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match self.inner {
            protocol::Message::Text(ref s) => s.as_slice(),
            protocol::Message::Binary(ref v) => v.as_slice(),
            protocol::Message::Ping(ref v) => v.as_slice(),
            protocol::Message::Pong(ref v) => v.as_slice(),
            protocol::Message::Close(_) => &[],
            protocol::Message::Frame(ref v) => v.payload(),
        }
    }

    /// Destructure this message into binary data.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner.into_data().as_slice().to_vec()
    }
}

impl fmt::Debug for Message {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
