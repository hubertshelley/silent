use crate::{Request, Response};
use async_session::Session;
use http_body::Body;
use serde::de::DeserializeOwned;

pub trait SessionExt {
    /// Get `Session` reference.
    fn sessions(&self) -> Session;
    /// Get `Session` mutable reference.
    fn sessions_mut(&mut self) -> &mut Session;
    /// Get `Session` from session.
    fn session<K: AsRef<str>, V: DeserializeOwned>(&self, name: K) -> Option<V>;
}

impl SessionExt for Request {
    fn sessions(&self) -> Session {
        self.extensions().get().cloned().unwrap_or_default()
    }

    fn sessions_mut(&mut self) -> &mut Session {
        self.extensions_mut().get_mut().unwrap()
    }

    fn session<K: AsRef<str>, V: DeserializeOwned>(&self, name: K) -> Option<V> {
        self.sessions().get(name.as_ref())
    }
}

impl<B: Body> SessionExt for Response<B> {
    fn sessions(&self) -> Session {
        self.extensions().get().cloned().unwrap_or_default()
    }

    fn sessions_mut(&mut self) -> &mut Session {
        self.extensions_mut().get_mut().unwrap()
    }

    fn session<K: AsRef<str>, V: DeserializeOwned>(&self, name: K) -> Option<V> {
        self.sessions().get(name.as_ref())
    }
}
