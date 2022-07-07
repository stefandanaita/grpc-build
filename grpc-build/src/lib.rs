use anyhow::{anyhow, Context, Ok, Result};
use prost::Message;
use prost_build::{protoc, protoc_include, Module};
use prost_types::{FileDescriptorProto, FileDescriptorSet};
use std::{collections::HashMap, path::Path, process::Command};

pub mod base;
mod builder;
pub mod tree;
pub use builder::Builder;

impl Builder {
    pub fn build(
        self,
        in_dir: impl AsRef<Path>,
    ) -> Result<(), anyhow::Error> {
        let out_dir = self.get_out_dir()?;
        if !self.force && out_dir.exists() {
            return Err(anyhow!(
                "the output directory already exists: {}",
                out_dir.display()
            ));
        }

        base::prepare_out_dir(&out_dir).context("failed to prepare out dir")?;

        self.compile(in_dir.as_ref(), &out_dir)
            .context("failed to compile the protos")?;

        base::refactor(out_dir).context("failed to refactor the protos")?;

        Ok(())
    }

    fn compile(self, input_dir: &Path, out_dir: &Path) -> Result<(), anyhow::Error> {
        let tmp = tempfile::Builder::new().prefix("grpc-build").tempdir()?;
        let file_descriptor_path = tmp.path().join("grpc-descriptor-set");

        self.run_protoc(input_dir.as_ref(), &file_descriptor_path)?;

        let buf = std::fs::read(&file_descriptor_path)?;
        let file_descriptor_set =
            FileDescriptorSet::decode(&*buf).context("invalid FileDescriptorSet")?;

        self.generate_services(out_dir, file_descriptor_set)
    }

    fn run_protoc(
        &self,
        input_dir: &Path,
        file_descriptor_path: &Path,
    ) -> Result<(), anyhow::Error> {
        let protos = crate::base::get_protos(input_dir).collect::<Vec<_>>();

        let compile_includes: &Path = match input_dir.parent() {
            None => Path::new("."),
            Some(parent) => parent,
        };

        let mut cmd = Command::new(protoc());
        cmd.arg("--include_imports")
            .arg("--include_source_info")
            .arg("-o")
            .arg(file_descriptor_path);
        cmd.arg("-I").arg(compile_includes);

        cmd.arg("-I").arg(protoc_include());

        for arg in &self.protoc_args {
            cmd.arg(arg);
        }

        for proto in &protos {
            cmd.arg(proto);
        }

        cmd.output().context(
            "failed to invoke protoc (hint: https://docs.rs/prost-build/#sourcing-protoc)",
        )?;
        Ok(())
    }

    fn generate_services(
        mut self,
        out_dir: &Path,
        file_descriptor_set: FileDescriptorSet,
    ) -> Result<(), anyhow::Error> {
        let service_generator = self.tonic.service_generator();
        self.prost.service_generator(service_generator);

        let requests = file_descriptor_set
            .file
            .into_iter()
            .map(|descriptor| {
                // Add our NamedMessage derive
                for (name, annotation) in derive_named_messages(&descriptor) {
                    self.prost.type_attribute(&name, annotation);
                }

                (
                    Module::from_protobuf_package_name(descriptor.package()),
                    descriptor,
                )
            })
            .collect::<Vec<_>>();

        let file_names = requests
            .iter()
            .map(|(module, _)| (module.clone(), module.to_file_name_or("_")))
            .collect::<HashMap<Module, String>>();

        let modules = self.prost.generate(requests)?;
        for (module, content) in &modules {
            let file_name = file_names
                .get(module)
                .expect("every module should have a filename");
            let output_path = out_dir.join(file_name);

            let previous_content = std::fs::read(&output_path);

            // only write the file if the contents have changed
            if previous_content
                .map(|previous_content| previous_content != content.as_bytes())
                .unwrap_or(true)
            {
                std::fs::write(output_path, content)?;
            }
        }

        Ok(())
    }
}

/// Build annotations for the top-level messages in a file,
fn derive_named_messages(
    descriptor: &FileDescriptorProto,
) -> impl Iterator<Item = (String, String)> + '_ {
    let namespace = descriptor.package();
    descriptor.message_type.iter().map(|message| {
        let full_name = fully_qualified_name(namespace, message.name());
        let derive =
            format!("#[derive(::grpc_build_core::NamedMessage)] #[name = \"{full_name}\"]");
        (full_name, derive)
    })
}

fn fully_qualified_name(namespace: &str, name: &str) -> String {
    let namespace = namespace.trim_start_matches('.');
    if namespace.is_empty() {
        name.into()
    } else {
        format!("{namespace}.{name}")
    }
}
