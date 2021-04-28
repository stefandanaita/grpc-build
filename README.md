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

To use additional include paths, use the `--include` (`-I`) flag.

```
grpc-build build -c -s --in-dir="<protobuf directory>" --out-dir="<codegen>" -f -I/usr/local/include -I${GOPATH}/src/github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis
```

### Using it as a library

The most convenient way of using `grpc_build` as a library is by taking advantage of Rust's `build.rs` file. Don't forget to add `grpc_build` to the [build-dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#build-dependencies) list.

```
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

## License
This project is licensed under the [MIT license](https://github.com/stefandanaita/grpc-build/blob/master/LICENSE).
