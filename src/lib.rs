use anyhow::{anyhow, Context, Result};
use prost::Message;
use prost_types::{FileDescriptorProto, FileDescriptorSet};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tonic_build::Builder;

pub mod base;
mod ident;
pub mod tree;

// pub use grpc_build_derive::FullyQualifiedName;

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

    // Steps:
    // 1. compile protos along with the filedescriptor set in a temp folder
    // 2. then we parse the filedescriptor set to obtain all message names
    // 3. add the proc macro annotation to generate the `full_proto_name` function.
    // 4. finally compile the protos using the annotation in the user-defined dir

    let tmp = tempfile::Builder::new().prefix("grpc-build").tempdir()?;
    let file_descriptor_path = tmp.path().join("grpc-descriptor-set");
    let tmp_dir = tempfile::Builder::new().prefix("grpc-build").tempdir()?;

    // let file_descriptor_path = Path::new(output_dir).join("grpc-descriptor-set");

    user_config(
        tonic_build::configure()
            .build_client(client)
            .build_server(server)
            // HACK: we compile it once to get the file_descriptor_set.
            // But we just hide it in a temp folder.
            // This is because tonci_build does not expose the `skip_protoc_run` fucntion
            // like prost_build so we cannot skip it.
            .out_dir(tmp_dir.path())
            .file_descriptor_set_path(&file_descriptor_path),
    )
    .compile(&protos, &[&compile_includes])?;

    let buf = std::fs::read(&file_descriptor_path)?;
    let file_descriptor_set =
        FileDescriptorSet::decode(&*buf).context("invalid FileDescriptorSet")?;

    // // Build mapping of <full_name, annotation>.
    // let annotations: String = file_descriptor_set
    //     .file
    //     .iter()
    //     .map(|descriptor| build_full_name_impls(descriptor, descriptor.package()))
    //     .collect();

    // for (name, annotation) in &annotations {
    // Build mapping of <full_name, annotation>.
    let annotations: HashMap<String, String> =
        file_descriptor_set
            .file
            .iter()
            .fold(HashMap::new(), |mut acc, descriptor| {
                acc.extend(build_annotations_in_file(descriptor, descriptor.package()));
                acc
            });

    let mut config = tonic_build::configure();
    for (name, annotation) in &annotations {
        config = config.type_attribute(&name, annotation);
    }

    user_config(
        config
            .build_client(client)
            .build_server(server)
            .out_dir(output_dir),
    )
    .compile(&protos, &[&compile_includes])?;

    // Ok(annotations)
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
            let item_path = fully_qualified_path(message.name());
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

fn fully_qualified_path(name: &str) -> String {
    ident::to_upper_camel(name)
}
