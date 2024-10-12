pub mod session_ext;

use crate::{
    CookieExt, Handler, MiddleWareHandler, Next, Request, Response, Result, SilentError, StatusCode,
};
use async_session::{MemoryStore, Session, SessionStore};
use async_trait::async_trait;
use cookie::Cookie;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SessionMiddleware<T>
where
    T: SessionStore,
{
    pub session_store: Arc<RwLock<T>>,
}

impl Default for SessionMiddleware<MemoryStore> {
    fn default() -> SessionMiddleware<MemoryStore> {
        let session = MemoryStore::new();
        Self::new(session)
    }
}

impl<T> SessionMiddleware<T>
where
    T: SessionStore,
{
    pub fn new(session: T) -> Self {
        let session_store = Arc::new(RwLock::new(session));
        SessionMiddleware { session_store }
    }
}

#[async_trait]
impl<T> MiddleWareHandler for SessionMiddleware<T>
where
    T: SessionStore,
{
    async fn handle(&self, mut req: Request, next: &Next) -> Result<Response> {
        let cookies = req.cookies().clone();
        let cookie = cookies.get("silent-web-session");
        let session_store = self.session_store.read().await;
        let session = if cookie.is_none() {
            req.extensions_mut().insert(cookies.clone());
            Session::new()
        } else {
            let cookie = cookie.unwrap();
            session_store
                .load_session(cookie.value().to_string())
                .await
                .map_err(|e| {
                    SilentError::business_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to load session: {}", e),
                    )
                })?
                .unwrap_or_default()
        };
        req.extensions_mut().insert(session.clone());
        let mut res = next.call(req).await?;
        res.extensions_mut().insert(session.clone());
        res.extensions_mut().insert(cookies);
        let cookie_value = session_store.store_session(session).await.map_err(|e| {
            SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to store session: {}", e),
            )
        })?;
        if let Some(cookie_value) = cookie_value {
            res.cookies_mut().add(
                Cookie::build(("silent-web-session", cookie_value))
                    .max_age(cookie::time::Duration::hours(2)),
            );
        }
        Ok(res)
    }
}
