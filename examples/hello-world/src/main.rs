use async_trait::async_trait;
use silent::{logger, Handler, Request, Response, Route, Server, SilentError};
use std::sync::Arc;

struct HandleRoot;

#[async_trait]
impl Handler for HandleRoot {
    async fn call(&self, _req: silent::Request) -> Result<Response, SilentError> {
        Ok(Response::from("hello world".to_string()))
    }
}

struct HandleRoot1;

#[async_trait]
impl Handler for HandleRoot1 {
    async fn call(&self, _req: silent::Request) -> Result<Response, SilentError> {
        Ok(Response::from("hello world1".to_string()))
    }
}

struct HandleRoot11;

#[async_trait]
impl Handler for HandleRoot11 {
    async fn call(&self, _req: silent::Request) -> Result<Response, SilentError> {
        Ok(Response::from("hello world11".to_string()))
    }
}

struct HandleRoot12;

#[async_trait]
impl Handler for HandleRoot12 {
    async fn call(&self, _req: silent::Request) -> Result<Response, SilentError> {
        Ok(Response::from("hello world12".to_string()))
    }
    async fn middleware_call(
        &self,
        _req: &mut Request,
        res: &mut Response,
    ) -> Result<(), SilentError> {
        res.set_status(404.try_into().unwrap());
        Ok(())
    }
}

fn main() {
    logger::fmt::init();
    let route = Route {
        path: "".to_string(),
        handler: Some(Arc::new(HandleRoot)),
        children: vec![Route {
            path: "1".to_string(),
            handler: Some(Arc::new(HandleRoot1)),
            children: vec![
                Route {
                    path: "1".to_string(),
                    handler: Some(Arc::new(HandleRoot11)),
                    children: vec![],
                    middlewares: vec![],
                },
                Route {
                    path: "2".to_string(),
                    handler: Some(Arc::new(HandleRoot12)),
                    children: vec![],
                    middlewares: vec![Arc::new(HandleRoot12)],
                },
            ],
            middlewares: vec![],
        }],
        middlewares: vec![],
    };
    Server::new()
        .bind("127.0.0.1:8001".parse().unwrap())
        .bind_route(route)
        .run();
}
