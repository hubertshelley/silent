use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use silent::prelude::info;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}
#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://0.0.0.0:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    info!("RESPONSE={:?}", response);

    info!("MESSAGE={:?}", response.into_inner());

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    info!("RESPONSE={:?}", response);

    info!("MESSAGE={:?}", response.into_inner());

    Ok(())
}
