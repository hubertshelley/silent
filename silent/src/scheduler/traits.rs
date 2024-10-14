use crate::{Request, Result, Scheduler, SilentError};
use http::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;

pub trait SchedulerExt {
    fn scheduler(&self) -> Result<&Arc<Mutex<Scheduler>>>;
}

impl SchedulerExt for Request {
    fn scheduler(&self) -> Result<&Arc<Mutex<Scheduler>>> {
        self.extensions().get().ok_or_else(|| {
            SilentError::business_error(StatusCode::INTERNAL_SERVER_ERROR, "No scheduler found")
        })
    }
}
