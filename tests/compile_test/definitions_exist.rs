mod protos {
    include!("protos/mod.rs");
}

use protos::grpc_build::{
    client::helloworld::greeter_client::GreeterClient,
    response::helloworld::HelloReply,
    request::helloworld::HelloRequest,
};

async fn foo(client: &mut GreeterClient<tonic::transport::Channel>, req: HelloRequest) -> anyhow::Result<HelloReply> {
    Ok(client.say_hello(req).await?.into_inner())
}

fn main() {}
