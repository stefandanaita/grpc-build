# grpc-build

`grpc-build` provides an flexible way to manage protobuf files and generate the gRPC code required by [tonic](https://github.com/hyperium/tonic).

It is built on top of [tonic_build](https://github.com/hyperium/tonic/tree/master/tonic-build) and it extends its functionality by compiling all the protobuf files inside a directory.

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
use grpc_build::build;

fn main() {
    build(
        "protos",       // protobuf files input dir
        "src/protogen", // output directory
        true,           // --build_server=true
        true,           // --build_client=true
        true,           // --force
    )
    .unwrap();
}
```

If you want to set advanced compilation options (like an additional `#[derive]` for the generated types), use the `build_with_config` function, which exposes the underlying [`tonic_build::Builder`](https://docs.rs/tonic-build/0.5.0/tonic_build/struct.Builder.html).

A more advanced usage is to use the `get_protos` and `refactor` functions yourself. The following example does the same as the example above

```rust
use grpc_build::base::{prepare_out_dir, get_protos, refactor};
use tonic_build::configure;

fn main() {
    let proto_src_dir = "protos";
    let proto_out_dir = "src/protogen";

    prepare_out_dir(proto_out_dir).unwrap();

    configure()
        .out_dir(proto_out_dir)
        .build_server(true)
        .build_client(true)
        .compile(
            &get_protos(proto_src_dir).collect::<Vec<_>>(),
            &["."]
        )
        .unwrap();

    refactor(proto_out_dir).unwrap();
}
```

## License
This project is licensed under the [MIT license](https://github.com/stefandanaita/grpc-build/blob/master/LICENSE).
