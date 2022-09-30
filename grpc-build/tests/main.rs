use grpc_build::Builder;

#[test]
fn build() {
    Builder::new()
        .build_client(true)
        .build_server(true)
        .force(true)
        .out_dir("tests/compile_test/protos")
        .default_module_name("some_default")
        .build("tests/protos/grpc_build")
        .unwrap();

    let t = trybuild::TestCases::new();
    t.pass("tests/compile_test/definitions_exist.rs");
}
