use crate::{Request, Response, Result, SilentError};
use cookie::{Cookie, CookieJar};
use http::StatusCode;

pub trait CookieExt {
    /// Get `CookieJar` reference.
    fn cookies(&self) -> Result<&CookieJar>;
    /// Get `CookieJar` mutable reference.
    fn cookies_mut(&mut self) -> Result<&mut CookieJar>;
    /// Get `Cookie` from cookies.
    fn cookie<T: AsRef<str>>(&self, name: T) -> Result<Option<&Cookie<'static>>>;
}

impl CookieExt for Request {
    fn cookies(&self) -> Result<&CookieJar> {
        self.extensions().get().ok_or_else(|| {
            SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Request does not have cookie jar",
            )
        })
    }

    fn cookies_mut(&mut self) -> Result<&mut CookieJar> {
        self.extensions_mut().get_mut().ok_or_else(|| {
            SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Request does not have cookie jar",
            )
        })
    }

    fn cookie<T: AsRef<str>>(&self, name: T) -> Result<Option<&Cookie<'static>>> {
        Ok(self
            .extensions()
            .get::<CookieJar>()
            .ok_or_else(|| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Request does not have cookie jar",
                )
            })?
            .get(name.as_ref()))
    }
}

impl CookieExt for Response {
    fn cookies(&self) -> Result<&CookieJar> {
        self.extensions().get().ok_or_else(|| {
            SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Request does not have cookie jar",
            )
        })
    }

    fn cookies_mut(&mut self) -> Result<&mut CookieJar> {
        self.extensions_mut().get_mut().ok_or_else(|| {
            SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Request does not have cookie jar",
            )
        })
    }

    fn cookie<T: AsRef<str>>(&self, name: T) -> Result<Option<&Cookie<'static>>> {
        Ok(self
            .extensions()
            .get::<CookieJar>()
            .ok_or_else(|| {
                SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Request does not have cookie jar",
                )
            })?
            .get(name.as_ref()))
    }
}
