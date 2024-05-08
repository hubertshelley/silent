use std::error::Error as StdError;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Bytes;
use http_body_util::BodyExt;
use tonic::codegen::{Body, Service};
use tonic::{transport::Server as TonicServer, Request, Response, Status};
use tower::ServiceExt;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use silent::prelude::{
    error, full, logger, HandlerAppend, HandlerGetter, Level, ReqBody, ResBody, Route, RouterAdapt,
    Server,
};
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

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

pub struct GrpcHandler<Svc, QB>(Svc, PhantomData<QB>);

#[async_trait]
impl<Svc, QB, SB, E, Fut> Handler for GrpcHandler<Svc, QB>
where
    QB: TryFrom<ReqBody> + Body + Send + Sync + 'static,
    <QB as TryFrom<ReqBody>>::Error: StdError + Send + Sync + 'static,
    SB: Body + Send + Sync + 'static,
    SB::Data: Into<Bytes> + Send + fmt::Debug + 'static,
    SB::Error: StdError + Send + Sync + 'static,
    E: StdError + Send + Sync + 'static,
    Svc: Service<hyper::Request<QB>, Response = hyper::Response<SB>, Future = Fut>
        + Send
        + Sync
        + Clone
        + 'static,
    Svc::Error: StdError + Send + Sync + 'static,
    Fut: Future<Output = Result<hyper::Response<SB>, E>> + Send + 'static,
{
    async fn call(&self, req: silent::Request) -> silent::Result<silent::Response> {
        let mut svc = self.0.clone();
        if svc.ready().await.is_err() {
            error!("tower service not ready.");
            return Err(SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "tower service not ready.".to_string(),
            ));
        }
        let hyper_req = req.into_http();

        let hyper_res = match svc.call(hyper_req).await {
            Ok(hyper_res) => hyper_res,
            Err(e) => {
                error!(error = ?e, "call tower service failed: {}", e);
                return Err(SilentError::business_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("call tower service failed: {}", e),
                ));
            }
        }
        .map(|res| {
            ResBody::Boxed(Box::pin(
                res.map_frame(|f| f.map_data(|data| data.into()))
                    .map_err(|e| e.into()),
            ))
        });
        let mut res = silent::Response::empty();
        res.merge_hyper(hyper_res);
        Ok(res)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::fmt().with_max_level(Level::INFO).init();
    let greeter = MyGreeter::default();
    let greeter_server = GreeterServer::new(greeter);
    let grpc_handler = GrpcHandler(greeter_server);
    let route = Route::new("")
        .handler(Method::GET, Arc::new(grpc_handler))
        .get(|_req| async { Ok("hello world") });
    Server::new().serve(route).await;
    let addr = "[::1]:50051".parse().unwrap();
    TonicServer::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;
    Ok(())
}
