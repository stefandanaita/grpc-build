# grpc-build

`grpc-build` provides an flexible way to manage protobuf files and generate the gRPC code required by [tonic](https://github.com/hyperium/tonic).

It is built on top of [tonic_build](https://github.com/hyperium/tonic/tree/master/tonic-build) and it extends its functionality by compiling all the protobuf files inside a directory and generating the `mod.rs` file according to the protobuf directory structure.

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
The most convenient way of using `grpc_build` as a library is by taking advantage of Rust's `build.rs` file.

```
// build.rs
use grpc_build::build;

fn main() {
    build(
        "protos",
        "src/protogen",
        true,
        true,
        true,
    )
    .unwrap();
}
```

## Continuous Integration Examples

### GitHub Actions
```
SoonTM
```

### CircleCI
```
SoonTM
```

## License
This project is licensed under the [MIT license](https://github.com/stefandanaita/grpc-build/blob/master/LICENSE).
