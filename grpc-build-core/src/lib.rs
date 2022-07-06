/// A trait to provide a static reference to the message's name
pub trait NamedMessage {
    const NAME: &'static str;
}
pub use grpc_build_derive::NamedMessage;
