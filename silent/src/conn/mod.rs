pub mod support;

use crate::rt::RtExecutor;
use hyper::server::conn::{http1, http2};

#[doc(hidden)]
#[allow(dead_code)]
pub struct SilentConnection {
    pub(crate) http1: http1::Builder,
    pub(crate) http2: http2::Builder<RtExecutor>,
}

impl Default for SilentConnection {
    fn default() -> Self {
        Self {
            http1: http1::Builder::new(),
            http2: http2::Builder::new(RtExecutor),
        }
    }
}
