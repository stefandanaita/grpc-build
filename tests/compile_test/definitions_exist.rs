mod protos {
    include!("protos/mod.rs");
}

use protos::grpc_build::{
    client::helloworld::greeter_client::GreeterClient, request::helloworld::HelloRequest,
    response::helloworld::HelloReply,
};

async fn foo(
    client: &mut GreeterClient<tonic::transport::Channel>,
    req: HelloRequest,
) -> anyhow::Result<HelloReply> {
    Ok(client.say_hello(req).await?.into_inner())
}

fn main() {
    assert_eq!(
        HelloReply::full_proto_name(),
        "grpc_build.response.helloworld.HelloReply"
    );
}
