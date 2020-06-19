use crate::graph_layout::{display, generate};
use crate::tonic_builder::compile;
use petgraph::graph::NodeIndex;
use std::fs;
use std::fs::File;
use std::path::Path;
use thiserror::Error;

mod graph_layout;
mod tonic_builder;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("The output directory already exists: {0}")]
    OutputDirectoryExistsError(String),

    #[error("{0}")]
    Error(String),
}

pub fn build(
    in_dir: &str,
    out_dir: &str,
    build_server: bool,
    build_client: bool,
    force: bool,
) -> Result<(), BuildError> {
    if Path::new(out_dir).exists() {
        if !force {
            return Err(BuildError::OutputDirectoryExistsError(String::from(
                out_dir,
            )));
        }

        match fs::remove_dir_all(out_dir) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to remove the output directory: {:?}", e);
                return Err(BuildError::Error(String::from(
                    "Could not remove the output directory",
                )))
            }
        };
    }

    match fs::create_dir_all(out_dir) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to create the output directory: {:?}", e);
            return Err(BuildError::Error(String::from(
                "Could not create the output directory",
            )))
        }
    };

    match compile(in_dir, out_dir, build_server, build_client) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to compile the protos: {:?}", e);
            return Err(BuildError::Error(String::from(
                "Failed the compile the protos",
            )))
        }
    };

    let graph = match generate(out_dir) {
        Ok(graph) => graph,
        Err(e) => {
            eprintln!("Failed to generate the graph: {:?}", e);
            return Err(BuildError::Error(String::from(
                "Failed to generate the graph",
            )))
        }
    };

    let mut proto_lib = match File::create(format!("{}/mod.rs", out_dir)) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create the mod.rs file: {:?}", e);
            return Err(BuildError::Error(String::from(
                "Failed to create the mod.rs file",
            )))
        }
    };

    match display(&graph, &mut proto_lib, NodeIndex::from(0)) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to populate the mod.rs file: {:?}", e);
            return Err(BuildError::Error(String::from(
                "Failed to populate the mod.rs file",
            )))
        }
    };

    Ok(())
}
