use crate::graph_layout::{display, generate};
use crate::tonic_builder::compile;
use petgraph::graph::NodeIndex;
use std::fs::File;
use std::fs;
use std::path::Path;
use thiserror::Error;

mod graph_layout;
mod tonic_builder;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("The output directory already exists: {0}")]
    OutputDirectoryExistsError(String),

    #[error("Generic error")]
    GenericError(),
}

pub fn build(
    in_dir: Option<String>,
    out_dir: Option<String>,
    build_server: bool,
    build_client: bool,
    force: bool,
) -> Result<(), BuildError> {
    let input_dir: String = match in_dir {
        None => String::from("protos"),
        Some(dir) => dir,
    };
    let output_dir: String = match out_dir {
        None => String::from("src/protogen"),
        Some(dir) => dir,
    };

    if Path::new(&output_dir).exists() {
        if !force {
            return Err(BuildError::OutputDirectoryExistsError(String::from(&output_dir)));
        }

        match fs::remove_dir_all(&output_dir) {
            Ok(_) => {},
            Err(_) => return Err(BuildError::GenericError()),
        };
    }

    match fs::create_dir_all(&output_dir) {
        Ok(_) => {},
        Err(_) => return Err(BuildError::GenericError()),
    };

    match compile(&input_dir, &output_dir, build_server, build_client) {
        Ok(_) => {},
        Err(_) => return Err(BuildError::GenericError()),
    };

    let graph = match generate(&output_dir) {
        Ok(graph) => graph,
        Err(_) => return Err(BuildError::GenericError()),
    };

    let mut proto_lib = match File::create(format!("{}/mod.rs", output_dir)) {
        Ok(file) => file,
        Err(_) => return Err(BuildError::GenericError()),
    };

    match display(&graph, &mut proto_lib, &NodeIndex::from(0)) {
        Ok(_) => {},
        Err(_) => return Err(BuildError::GenericError()),
    };

    Ok(())
}
