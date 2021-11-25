use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use tonic_build::Builder;

pub mod base;
pub mod tree;

pub fn build(
    in_dir: &str,
    out_dir: &str,
    build_server: bool,
    build_client: bool,
    force: bool,
) -> Result<()> {
    build_with_config(in_dir, out_dir, build_server, build_client, force, |c| c)
}

pub fn build_with_config(
    in_dir: &str,
    out_dir: &str,
    build_server: bool,
    build_client: bool,
    force: bool,
    user_config: impl FnOnce(Builder) -> Builder,
) -> Result<()> {
    if !force && Path::new(out_dir).exists() {
        return Err(anyhow!("the output directory already exists: {}", out_dir));
    }

    base::prepare_out_dir(out_dir).context("failed to prepare out dir")?;

    compile(in_dir, out_dir, build_server, build_client, user_config)
        .context("failed to compile the protos")?;

    base::refactor(out_dir).context("failed to refactor the protos")?;

    Ok(())
}

fn compile(
    input_dir: &str,
    output_dir: &str,
    server: bool,
    client: bool,
    user_config: impl FnOnce(Builder) -> Builder,
) -> Result<(), anyhow::Error> {
    let protos = crate::base::get_protos(input_dir).collect::<Vec<_>>();

    let compile_includes: PathBuf = match Path::new(input_dir).parent() {
        None => PathBuf::from("."),
        Some(parent) => parent.to_path_buf(),
    };

    user_config(
        tonic_build::configure()
            .out_dir(output_dir)
            .build_client(client)
            .build_server(server),
    )
    .compile(&protos, &[compile_includes])?;

    Ok(())
}
