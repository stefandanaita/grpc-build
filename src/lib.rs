use crate::graph_layout::{display, generate};
use crate::tonic_builder::compile;
use petgraph::graph::NodeIndex;
use std::fs::File;

mod graph_layout;
mod tonic_builder;

pub fn build(
    in_dir: Option<String>,
    out_dir: Option<String>,
    build_server: bool,
    build_client: bool,
) -> Result<(), anyhow::Error> {
    let input_dir: String = match in_dir {
        None => String::from("protos"),
        Some(dir) => dir,
    };
    let output_dir: String = match out_dir {
        None => String::from("src/protogen"),
        Some(dir) => dir,
    };

    compile(&input_dir, &output_dir, build_server, build_client)?;

    let graph = generate(&output_dir)?;

    let mut proto_lib = File::create(format!("{}/mod.rs", output_dir))?;
    display(&graph, &mut proto_lib, &NodeIndex::from(0))?;

    Ok(())
}
