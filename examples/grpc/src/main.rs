use async_trait::async_trait;
use std::sync::Arc;
use tonic::codegen::Service;
use tonic::{transport::Server as TonicServer, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use silent::prelude::{error, logger, HandlerAppend, HandlerGetter, Level, Route, Server};
use silent::{Handler, Method, SilentError, StatusCode};

pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

pub struct AxumRouterHandler(axum::Router<()>);

#[async_trait]
impl Handler for AxumRouterHandler {
    async fn call(&self, req: silent::Request) -> silent::Result<silent::Response> {
        let req = req.into_http();
        let mut handler = self.0.clone();
        let axum_res = handler.call(req).await.map_err(|e| {
            error!(error = ?e, "call axum router failed: {}", e);
            SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("call axum router failed: {}", e),
            )
        })?;
        let mut res = silent::Response::empty();
        res.merge_axum(axum_res).await;
        Ok(res)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let greeter = MyGreeter::default();
    logger::fmt().with_max_level(Level::INFO).init();
    let greeter_server = GreeterServer::new(greeter);
    let grpc = AxumRouterHandler(
        TonicServer::builder()
            // Wrap all services in the middleware stack
            .add_service(greeter_server)
            .into_router(),
    );
    let route = Route::new("")
        .handler(Method::GET, Arc::new(grpc))
        .get(|_req| async { Ok("hello world") });
    Server::new()
        .bind("0.0.0.0:50051".parse().unwrap())
        .serve(route)
        .await;
    Ok(())
}
