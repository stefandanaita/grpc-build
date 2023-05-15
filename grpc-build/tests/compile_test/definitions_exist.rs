mod protos {
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("protos/descriptor.bin");
    include!("protos/mod.rs");
}

use grpc_build_core::NamedMessage;
use prost::Message;

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
        <HelloReply as NamedMessage>::NAME,
        "grpc_build.response.helloworld.HelloReply"
    );
    prost_types::FileDescriptorSet::decode(protos::FILE_DESCRIPTOR_SET).unwrap();
}
