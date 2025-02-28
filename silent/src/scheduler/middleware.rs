use crate::{Handler, MiddleWareHandler, Next, Request, Response, Result};
use async_trait::async_trait;

#[derive(Debug, Default)]
pub struct SchedulerMiddleware {}

impl SchedulerMiddleware {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MiddleWareHandler for SchedulerMiddleware {
    async fn handle(&self, mut req: Request, next: &Next) -> Result<Response> {
        let scheduler = super::SCHEDULER.clone();
        req.extensions_mut().insert(scheduler);
        next.call(req).await
    }
}
