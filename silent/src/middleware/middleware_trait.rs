use std::sync::Arc;

use async_trait::async_trait;

use crate::{Handler, Request, Response, Result};

pub struct Next {
    inner: NextInstance,
    next: Option<Arc<Next>>,
}

pub(crate) enum NextInstance {
    Middleware(Arc<dyn MiddleWareHandler>),
    EndPoint(Arc<dyn Handler>),
}

impl Next {
    pub(crate) fn build(
        endpoint: Arc<dyn Handler>,
        mut middlewares: Vec<Arc<dyn MiddleWareHandler>>,
    ) -> Self {
        let end_point = Next {
            inner: NextInstance::EndPoint(endpoint),
            next: None,
        };
        if middlewares.is_empty() {
            end_point
        } else {
            let next = Next {
                inner: NextInstance::Middleware(middlewares.pop().unwrap()),
                next: Some(Arc::new(end_point)),
            };
            middlewares.into_iter().fold(next, |next, mw| Next {
                inner: NextInstance::Middleware(mw),
                next: Some(Arc::new(next)),
            })
        }
    }
}

impl Next {
    pub async fn call(&self, req: Request) -> Result<Response> {
        match &self.inner {
            NextInstance::Middleware(mw) => {
                mw.handle(req, self.next.clone().unwrap().as_ref()).await
            }
            NextInstance::EndPoint(ep) => ep.call(req).await,
        }
    }
}
#[async_trait]
pub trait MiddleWareHandler: Send + Sync + 'static {
    async fn match_req(&self, _req: &Request) -> bool {
        true
    }
    async fn handle(&self, _req: Request, _next: &Next) -> Result<Response>;
}

#[async_trait]
impl MiddleWareHandler for Next {
    async fn handle(&self, req: Request, next: &Next) -> Result<Response> {
        next.call(req).await
    }
}

#[cfg(test)]
mod tests {
    use crate::HandlerWrapper;
    use tracing::info;

    use super::*;

    struct TestMiddleWare {
        count: u32,
    }

    #[async_trait]
    impl MiddleWareHandler for TestMiddleWare {
        async fn handle(&self, req: Request, next: &Next) -> Result<Response> {
            info!("{}", self.count);
            next.call(req).await
        }
    }

    async fn hello_world(_req: Request) -> Result<String> {
        Ok("Hello World".into())
    }

    #[tokio::test]
    async fn test_middleware() -> Result<()> {
        let handler_wrapper = HandlerWrapper::new(hello_world).arc();
        let middleware1 = TestMiddleWare { count: 1 };
        let middleware2 = TestMiddleWare { count: 2 };
        let req = Request::empty();
        let middlewares = Next::build(
            handler_wrapper,
            vec![Arc::new(middleware1), Arc::new(middleware2)],
        );
        let res = middlewares.call(req).await;
        assert!(res.is_ok());
        info!("{:?}", res.unwrap());
        Ok(())
    }
}
