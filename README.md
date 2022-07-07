# grpc-build

`grpc-build` provides an flexible way to manage protobuf files and generate the gRPC code required by [tonic](https://github.com/hyperium/tonic).

It is built on top of [tonic_build](https://github.com/hyperium/tonic/tree/master/tonic-build) and it extends its functionality by compiling all the protobuf files inside a directory.
In addition to that, this library adds another feature: full proto name annotation.
This could be useful in cases where you want use the full name (package + message name) to identify a protobuf message.
Therefore, for each top-level protobuf message this library adds a method to its generated struct returning its full proto name.

Given the following protobuf definition:
```protobuf
// my_message.proto
package grpc_build;
message Message {}
```

The library will generate the standard Rust code plus the extra impl for each message.

```rust
// Message.rs (generated)
struct Message {}

impl NamedMessage for Message {
    /// This returns package (grpc-build) + message name (Message).
    const NAME: &'static str = "grpc_build.MyMessage"
}
```

If the protobuf content is valid (worth [linting it](https://buf.build/docs/tour-4)), `grpc-build` will take care of the protobuf imports and it will also generate the `mod.rs` file to allow the compiler to find the generated code. This file will be placed inside the *output directory*.

It comes both as a library that can be used directly inside a project and as a binary that can be used in CI pipelines.

[Documentation](https://docs.rs/grpc-build) - [Crates.io](https://crates.io/crates/grpc-build)

## Getting started

### Using it as a binary
Get the [latest binary release](https://github.com/stefandanaita/grpc-build/releases) and use it inside your CI pipeline.

```
grpc-build build --in-dir="<protobuf directory>" --out-dir="<codegen>"
```

Depending on the requirements, you can generate the gRPC Client and/or Server by using the `--build-client` (`-c`) and `--build-server` (`-s`) flags.

To overwrite the contents of the output directory, use the `--force` (`-f`) flag.

```
// both client and server, overwriting the existing protogen
grpc-build build -c -s --in-dir="<protobuf directory>" --out-dir="<codegen>" -f
```

### Using it as a library

The most convenient way of using `grpc_build` as a library is by taking advantage of Rust's `build.rs` file. Don't forget to add `grpc_build` to the [build-dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#build-dependencies) list.

```rust
// build.rs
use grpc_build::Builder;

fn main() {
    Builder::new()
        .build_client(true)
        .build_server(true)
        .force(true)
        .out_dir("src/protogen")
        .build("protos")
        .unwrap();
}
```

If you want to set advanced compilation options (like an additional `#[derive]` for the generated types), use the `build_with_config` function, which exposes the underlying [`tonic_build::Builder`](https://docs.rs/tonic-build/0.5.0/tonic_build/struct.Builder.html).

A more advanced usage is to use the `get_protos` and `refactor` functions yourself. The following example does almost the same as the example above, except you don't get the `NamedMessage` traits auto derived

```rust
fn main() {
    let proto_src_dir = "protos";
    let proto_out_dir = "src/protogen";

    let protos: Vec<_> = crate::base::get_protos(proto_src_dir).collect();

    grpc_build::prepare_out_dir(proto_out_dir).unwrap();

    tonic_build::configure()
        .out_dir(proto_out_dir)
        .build_server(true)
        .build_client(true)
        .compile(&protos, &["."])
        .unwrap();

    grpc_build::refactor(proto_out_dir).unwrap();
}
```

## License
This project is licensed under the [MIT license](https://github.com/stefandanaita/grpc-build/blob/master/LICENSE).
