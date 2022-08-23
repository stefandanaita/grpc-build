/// A trait to provide a static reference to the message's name
pub trait NamedMessage {
    const NAME: &'static str;
}

// This derive is hidden since it doesn't do anything interesting.
// It's general use is `#[derive(NamedMessage)] #[name = "{message_name}"]`, which is not
// any more convenient than just implementing `NamedMessage` manually.
// This is purely a derive for our own implementation detail with prost-build
#[doc(hidden)]
pub use grpc_build_derive::NamedMessage;

impl NamedMessage for bool {
    const NAME: &'static str = "google.protobuf.BoolValue";
}
impl NamedMessage for bytes::Bytes {
    const NAME: &'static str = "google.protobuf.BytesValue";
}
impl NamedMessage for Vec<u8> {
    const NAME: &'static str = "google.protobuf.BytesValue";
}
impl NamedMessage for f64 {
    const NAME: &'static str = "google.protobuf.DoubleValue";
}
impl NamedMessage for () {
    const NAME: &'static str = "google.protobuf.Empty";
}
impl NamedMessage for f32 {
    const NAME: &'static str = "google.protobuf.FloatValue";
}
impl NamedMessage for i32 {
    const NAME: &'static str = "google.protobuf.Int32Value";
}
impl NamedMessage for i64 {
    const NAME: &'static str = "google.protobuf.Int64Value";
}
impl NamedMessage for String {
    const NAME: &'static str = "google.protobuf.StringValue";
}
impl NamedMessage for u32 {
    const NAME: &'static str = "google.protobuf.UInt32Value";
}
impl NamedMessage for u64 {
    const NAME: &'static str = "google.protobuf.UInt64Value";
}

impl NamedMessage for prost_types::Any {
    const NAME: &'static str = "google.protobuf.Any";
}
impl NamedMessage for prost_types::Api {
    const NAME: &'static str = "google.protobuf.Api";
}
impl NamedMessage for prost_types::DescriptorProto {
    const NAME: &'static str = "google.protobuf.DescriptorProto";
}
impl NamedMessage for prost_types::Duration {
    const NAME: &'static str = "google.protobuf.Duration";
}
impl NamedMessage for prost_types::Enum {
    const NAME: &'static str = "google.protobuf.Enum";
}
impl NamedMessage for prost_types::EnumDescriptorProto {
    const NAME: &'static str = "google.protobuf.EnumDescriptorProto";
}
impl NamedMessage for prost_types::EnumOptions {
    const NAME: &'static str = "google.protobuf.EnumOptions";
}
impl NamedMessage for prost_types::EnumValue {
    const NAME: &'static str = "google.protobuf.EnumValue";
}
impl NamedMessage for prost_types::EnumValueDescriptorProto {
    const NAME: &'static str = "google.protobuf.EnumValueDescriptorProto";
}
impl NamedMessage for prost_types::EnumValueOptions {
    const NAME: &'static str = "google.protobuf.EnumValueOptions";
}
impl NamedMessage for prost_types::ExtensionRangeOptions {
    const NAME: &'static str = "google.protobuf.ExtensionRangeOptions";
}
impl NamedMessage for prost_types::Field {
    const NAME: &'static str = "google.protobuf.Field";
}
impl NamedMessage for prost_types::FieldDescriptorProto {
    const NAME: &'static str = "google.protobuf.FieldDescriptorProto";
}
impl NamedMessage for prost_types::FieldMask {
    const NAME: &'static str = "google.protobuf.FieldMask";
}
impl NamedMessage for prost_types::FieldOptions {
    const NAME: &'static str = "google.protobuf.FieldOptions";
}
impl NamedMessage for prost_types::GeneratedCodeInfo {
    const NAME: &'static str = "google.protobuf.GeneratedCodeInfo";
}
impl NamedMessage for prost_types::ListValue {
    const NAME: &'static str = "google.protobuf.ListValue";
}
impl NamedMessage for prost_types::MessageOptions {
    const NAME: &'static str = "google.protobuf.MessageOptions";
}
impl NamedMessage for prost_types::Method {
    const NAME: &'static str = "google.protobuf.Method";
}
impl NamedMessage for prost_types::MethodDescriptorProto {
    const NAME: &'static str = "google.protobuf.MethodDescriptorProto";
}
impl NamedMessage for prost_types::MethodOptions {
    const NAME: &'static str = "google.protobuf.MethodOptions";
}
impl NamedMessage for prost_types::Mixin {
    const NAME: &'static str = "google.protobuf.Mixin";
}
impl NamedMessage for prost_types::OneofDescriptorProto {
    const NAME: &'static str = "google.protobuf.OneofDescriptorProto";
}
impl NamedMessage for prost_types::OneofOptions {
    const NAME: &'static str = "google.protobuf.OneofOptions";
}
impl NamedMessage for prost_types::Option {
    const NAME: &'static str = "google.protobuf.Option";
}
impl NamedMessage for prost_types::ServiceDescriptorProto {
    const NAME: &'static str = "google.protobuf.ServiceDescriptorProto";
}
impl NamedMessage for prost_types::ServiceOptions {
    const NAME: &'static str = "google.protobuf.ServiceOptions";
}
impl NamedMessage for prost_types::SourceCodeInfo {
    const NAME: &'static str = "google.protobuf.SourceCodeInfo";
}
impl NamedMessage for prost_types::SourceContext {
    const NAME: &'static str = "google.protobuf.SourceContext";
}
impl NamedMessage for prost_types::Struct {
    const NAME: &'static str = "google.protobuf.Struct";
}
impl NamedMessage for prost_types::Timestamp {
    const NAME: &'static str = "google.protobuf.Timestamp";
}
impl NamedMessage for prost_types::Type {
    const NAME: &'static str = "google.protobuf.Type";
}
impl NamedMessage for prost_types::UninterpretedOption {
    const NAME: &'static str = "google.protobuf.UninterpretedOption";
}
impl NamedMessage for prost_types::Value {
    const NAME: &'static str = "google.protobuf.Value";
}
