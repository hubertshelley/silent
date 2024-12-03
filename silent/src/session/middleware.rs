use crate::{
    CookieExt, Handler, MiddleWareHandler, Next, Request, Response, SilentError, StatusCode,
};
use async_session::{MemoryStore, Session, SessionStore};
use async_trait::async_trait;
use cookie::{Cookie, CookieJar};
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
    async fn handle(&self, mut req: Request, next: &Next) -> crate::Result<Response> {
        let mut cookies = req.cookies().clone();
        let cookie = cookies.get("silent-web-session");
        let session_store = self.session_store.read().await;
        let mut session_key_exists = false;
        let cookie_value = if cookie.is_some() {
            session_key_exists = true;
            cookie.unwrap().value().to_string()
        } else {
            session_store.store_session(Session::new()).await?.unwrap()
        };
        let session = session_store
            .load_session(cookie_value.clone())
            .await
            .map_err(|e| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to load session: {}", e),
                )
            })?
            .ok_or(SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load session",
            ))?;
        req.extensions_mut().insert(session.clone());
        let session_copied = session.clone();
        if !session_key_exists {
            cookies.add(
                Cookie::build(("silent-web-session", cookie_value))
                    .max_age(cookie::time::Duration::hours(2)),
            );
        }
        let mut res = next.call(req).await?;
        if res.extensions().get::<Session>().is_none() {
            res.extensions_mut().insert(session_copied);
        }
        if res.extensions().get::<CookieJar>().is_none() {
            res.extensions_mut().insert(cookies);
        } else {
            let cookie_jar = res.extensions_mut().get_mut::<CookieJar>().unwrap();
            for cookie in cookie_jar.iter() {
                cookies.add(cookie.clone());
            }
            res.extensions_mut().insert(cookies.clone());
        }
        Ok(res)
    }
}
