use std::fmt;
use std::fmt::Formatter;
use tokio_tungstenite::tungstenite::protocol;

#[derive(Eq, PartialEq, Clone)]
pub struct Message {
    pub(crate) inner: protocol::Message,
}

impl fmt::Debug for Message {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
