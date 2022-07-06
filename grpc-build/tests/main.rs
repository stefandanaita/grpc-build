use grpc_build::Builder;

#[test]
fn build() {
    Builder::new()
        .build_client(true)
        .build_server(true)
        .build(
            "tests/protos/grpc_build",
            "tests/compile_test/protos",
            true, // force build
        )
        .unwrap();

    let t = trybuild::TestCases::new();
    t.pass("tests/compile_test/definitions_exist.rs");
}
