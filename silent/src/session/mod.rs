use crate::{MiddleWareHandler, Request, Response, Result, SilentError, StatusCode};
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
    async fn pre_request(&self, req: &mut Request, _res: &mut Response) -> Result<()> {
        let cookies = req.cookies().clone();
        let cookie = cookies.get("silent-web-session");
        if cookie.is_none() {
            req.extensions_mut().insert(Session::new());
            return Ok(());
        }
        let cookie = cookie.unwrap();
        let session = self
            .session_store
            .read()
            .await
            .load_session(cookie.value().to_string())
            .await
            .map_err(|e| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to load session: {}", e),
                )
            })?;
        if let Some(session) = session {
            req.extensions_mut().insert(session);
        } else {
            req.extensions_mut().insert(Session::new());
        }
        Ok(())
    }
    async fn after_response(&self, res: &mut Response) -> Result<()> {
        let session_store = self.session_store.read().await;
        let session = res.extensions.remove::<Session>();
        let cookie = res.cookies().get("silent-web-session");
        if let Some(mut session) = session {
            if let Some(cookie) = cookie {
                if let Ok(session_id) = Session::id_from_cookie_value(cookie.value()) {
                    if session.id() != session_id {
                        session.regenerate()
                    }
                }
            } else {
                session.regenerate()
            }
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
        }
        Ok(())
    }
}
