#[test]
fn build() {
    grpc_build::build(
        "tests/protos/grpc_build",
        "tests/compile_test/protos",
        true,
        true,
        true,
    )
    .unwrap();

    let t = trybuild::TestCases::new();
    t.pass("tests/compile_test/definitions_exist.rs");
}
