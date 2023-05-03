use super::Route;
use crate::handler::HandlerWrapperHtml;
use crate::{Handler, HandlerWrapper, Method, Request, SilentError};
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

pub trait HandlerGetter {
    fn get_handler_mut(&mut self) -> &mut HashMap<Method, Arc<dyn Handler>>;
}

pub trait HandlerAppend<F, T, Fut>: HandlerGetter
where
    Fut: Future<Output = Result<T, SilentError>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    T: Serialize + Send + 'static,
{
    fn get(self, handler: F) -> Self;
    fn post(self, handler: F) -> Self;
    fn put(self, handler: F) -> Self;
    fn delete(self, handler: F) -> Self;
    fn patch(self, handler: F) -> Self;
    fn options(self, handler: F) -> Self;
    fn handler_append(&mut self, method: Method, handler: F) {
        let handler = Arc::new(HandlerWrapper::new(handler));
        self.get_handler_mut().insert(method, handler);
    }
}

impl HandlerGetter for Route {
    fn get_handler_mut(&mut self) -> &mut HashMap<Method, Arc<dyn Handler>> {
        if self.path == self.create_path {
            &mut self.handler
        } else {
            let mut iter = self.create_path.splitn(2, '/');
            let _local_url = iter.next().unwrap_or("");
            let last_url = iter.next().unwrap_or("");
            let route = self
                .children
                .iter_mut()
                .find(|c| c.create_path == last_url)
                .unwrap();
            <Route as HandlerGetter>::get_handler_mut(route)
        }
    }
}

impl<F, T, Fut> HandlerAppend<F, T, Fut> for Route
where
    Fut: Future<Output = Result<T, SilentError>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    T: Serialize + Send + 'static,
{
    fn get(mut self, handler: F) -> Self {
        self.handler_append(Method::GET, handler);
        self
    }

    fn post(mut self, handler: F) -> Self {
        self.handler_append(Method::POST, handler);
        self
    }

    fn put(mut self, handler: F) -> Self {
        self.handler_append(Method::PUT, handler);
        self
    }

    fn delete(mut self, handler: F) -> Self {
        self.handler_append(Method::DELETE, handler);
        self
    }

    fn patch(mut self, handler: F) -> Self {
        self.handler_append(Method::PATCH, handler);
        self
    }

    fn options(mut self, handler: F) -> Self {
        self.handler_append(Method::OPTIONS, handler);
        self
    }
}

pub trait HtmlHandlerAppend<F, Fut>: HandlerGetter
where
    Fut: Future<Output = Result<&'static str, SilentError>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
{
    fn get_html(self, handler: F) -> Self;
    fn html_handler_append(&mut self, method: Method, handler: F) {
        let handler = Arc::new(HandlerWrapperHtml::new(handler));
        self.get_handler_mut().insert(method, handler);
    }
}

impl<F, Fut> HtmlHandlerAppend<F, Fut> for Route
where
    Fut: Future<Output = Result<&'static str, SilentError>> + Send + Sync + 'static,
    F: Fn(Request) -> Fut + Send + Sync + 'static,
{
    fn get_html(mut self, handler: F) -> Self {
        self.html_handler_append(Method::GET, handler);
        self
    }
}
