use anyhow::{anyhow, Context, Result};
use prost::Message;
use prost_build::{protoc, protoc_include, Config, ServiceGenerator};
use prost_types::{FileDescriptorProto, FileDescriptorSet};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};
use tonic_build::Builder;

pub mod base;
mod ident;
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
    user_config: impl Fn(Builder) -> Builder,
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

pub fn build_with_prost(
    in_dir: &str,
    out_dir: &str,
    build_server: bool,
    build_client: bool,
    force: bool,
) -> Result<()> {
    build_with_prost_config(
        in_dir,
        out_dir,
        force,
        &[] as &[&str],
        |c| c,
        |c: Builder| {
            c.build_client(build_client)
                .build_server(build_server)
                .service_generator()
        },
    )
}

pub fn build_with_prost_config(
    in_dir: &str,
    out_dir: &str,
    force: bool,
    protoc_args: &[impl AsRef<OsStr>],
    prost_config: impl FnOnce(Config) -> Config,
    service_generator: impl FnOnce(Builder) -> Box<dyn ServiceGenerator>,
) -> Result<()> {
    if !force && Path::new(out_dir).exists() {
        return Err(anyhow!("the output directory already exists: {}", out_dir));
    }

    base::prepare_out_dir(out_dir).context("failed to prepare out dir")?;

    prost_compile(
        in_dir,
        out_dir,
        protoc_args,
        prost_config,
        service_generator,
    )
    .context("failed to compile the protos")?;

    base::refactor(out_dir).context("failed to refactor the protos")?;

    Ok(())
}

fn prost_compile(
    input_dir: &str,
    output_dir: &str,
    protoc_args: &[impl AsRef<OsStr>],
    prost_config: impl FnOnce(Config) -> Config,
    service_generator: impl FnOnce(Builder) -> Box<dyn ServiceGenerator>,
) -> Result<(), anyhow::Error> {
    let protos = crate::base::get_protos(input_dir).collect::<Vec<_>>();

    let compile_includes: PathBuf = match Path::new(input_dir).parent() {
        None => PathBuf::from("."),
        Some(parent) => parent.to_path_buf(),
    };

    let tmp = tempfile::Builder::new().prefix("grpc-build").tempdir()?;
    let file_descriptor_path = tmp.path().join("grpc-descriptor-set");

    let mut cmd = Command::new(protoc());
    cmd.arg("--include_imports")
        .arg("--include_source_info")
        .arg("-o")
        .arg(&file_descriptor_path);

    cmd.arg("-I").arg(&compile_includes);

    // Set the protoc include after the user includes in case the user wants to
    // override one of the built-in .protos.
    cmd.arg("-I").arg(protoc_include());

    for arg in protoc_args {
        cmd.arg(arg);
    }

    for proto in &protos {
        cmd.arg(proto);
    }

    let _output = cmd.output().map_err(|error| {
        anyhow::anyhow!(format!(
            "failed to invoke protoc (hint: https://docs.rs/prost-build/#sourcing-protoc): {}",
            error
        ),)
    })?;

    let buf = std::fs::read(&file_descriptor_path)?;
    let file_descriptor_set =
        FileDescriptorSet::decode(&*buf).context("invalid FileDescriptorSet")?;

    // Build mapping of <full_name, annotation>.
    let annotations: HashMap<String, String> =
        file_descriptor_set
            .file
            .iter()
            .fold(HashMap::new(), |mut acc, descriptor| {
                acc.extend(build_annotations_in_file(descriptor, descriptor.package()));
                acc
            });

    let mut config = prost_config(prost_build::Config::new());
    for (name, annotation) in &annotations {
        config.type_attribute(&name, annotation);
    }

    config
        .skip_protoc_run()
        .out_dir(output_dir)
        .file_descriptor_set_path(&file_descriptor_path)
        .service_generator(service_generator(tonic_build::configure()));

    config
        .compile_protos(&protos, &[&compile_includes])
        .context("failed to generate rust files")?;

    Ok(())
}

fn compile(
    input_dir: &str,
    output_dir: &str,
    server: bool,
    client: bool,
    user_config: impl Fn(Builder) -> Builder,
) -> Result<(), anyhow::Error> {
    let protos = crate::base::get_protos(input_dir).collect::<Vec<_>>();

    let compile_includes: PathBuf = match Path::new(input_dir).parent() {
        None => PathBuf::from("."),
        Some(parent) => parent.to_path_buf(),
    };

    user_config(tonic_build::configure())
        .build_client(client)
        .build_server(server)
        .out_dir(output_dir)
        .compile(&protos, &[&compile_includes])?;

    Ok(())
}

/// Build annotations for the top-level messages in a file,
fn build_annotations_in_file(
    descriptor: &FileDescriptorProto,
    namespace: &str,
) -> HashMap<String, String> {
    descriptor
        .message_type
        .iter()
        .map(|message| {
            let full_name = fully_qualified_name(namespace, message.name());
            let item_path = ident::to_upper_camel(message.name());
            let impl_ = format!(
                "impl {item_path} {{
                    pub fn full_proto_name() -> &'static str {{ \"{full_name}\" }}
                }}"
            );
            (full_name, impl_)
        })
        .collect()
}

fn fully_qualified_name(namespace: &str, name: &str) -> String {
    let namespace = namespace.trim_start_matches('.');
    if namespace.is_empty() {
        name.into()
    } else {
        let mut full_name = String::with_capacity(namespace.len() + 1 + name.len());
        full_name.push_str(namespace);
        full_name.push('.');
        full_name.push_str(name);
        full_name
    }
}
