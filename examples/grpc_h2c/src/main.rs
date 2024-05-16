// use tonic::transport::server::TowerToHyperService;
use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use silent::prelude::{logger, HandlerAppend, Level, Route, RouteService};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let greeter = MyGreeter::default();
    logger::fmt().with_max_level(Level::INFO).init();

    let grpc = Server::builder()
        .add_service(GreeterServer::new(greeter))
        .into_router();
    let route = Route::new("").get(|_req| async { Ok("hello world") });
    let root = route.route().with_grpc(grpc.into());
    silent::prelude::Server::new()
        .bind("0.0.0.0:50051".parse().unwrap())
        .serve(root)
        .await;
    Ok(())
}
