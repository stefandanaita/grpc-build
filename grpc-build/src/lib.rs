use anyhow::{anyhow, Context, Result};
use prost::Message;
use prost_types::{FileDescriptorProto, FileDescriptorSet};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
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

    // Steps:
    // 1. compile protos along with the filedescriptor set in a temp folder
    // 2. then we parse the filedescriptor set to obtain all message names
    // 3. add the proc macro annotation to generate the `full_proto_name` function.
    // 4. finally compile the protos using the annotation in the user-defined dir

    let tmp = tempfile::Builder::new().prefix("grpc-build").tempdir()?;
    let file_descriptor_path = tmp.path().join("grpc-descriptor-set");
    let tmp_dir = tempfile::Builder::new().prefix("grpc-build").tempdir()?;

    let config = user_config(tonic_build::configure());
    config
        .clone()
        .build_client(client)
        .build_server(server)
        // HACK: we compile it once to get the file_descriptor_set.
        // But we just hide it in a temp folder.
        // This is because tonci_build does not expose the `skip_protoc_run` fucntion
        // like prost_build so we cannot skip it.
        .out_dir(tmp_dir.path())
        .file_descriptor_set_path(file_descriptor_path.clone())
        .compile(&protos, &[compile_includes.clone()])?;

    let buf = std::fs::read(file_descriptor_path.clone())?;
    let file_descriptor_set = FileDescriptorSet::decode(&*buf)
        .map_err(|error| anyhow::anyhow!("invalid FileDescriptorSet: {}", error))?;

    let mut config = tonic_build::configure();

    // Build mapping of <full_name, annotation>.
    let annotations: HashMap<String, String> =
        file_descriptor_set
            .file
            .iter()
            .fold(HashMap::new(), |mut acc, descriptor| {
                acc.extend(build_annotations_in_file(descriptor, descriptor.package()));
                acc
            });

    for (name, annotation) in &annotations {
        config = config.type_attribute(&name, annotation);
    }

    config
        .build_client(client)
        .out_dir(output_dir)
        .build_server(server)
        .file_descriptor_set_path(file_descriptor_path.clone())
        .compile(&protos, &[compile_includes])?;

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
            (
                full_name.clone(),
                format!(
                    "#[derive(::grpc_build_derive::FullyQualifiedName)] #[name = \"{}\"]",
                    &full_name
                ),
            )
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
