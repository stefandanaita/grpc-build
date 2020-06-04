use crate::graph_layout::{display, generate};
use crate::tonic_builder::compile;
use petgraph::graph::NodeIndex;
use std::fs::File;

mod graph_layout;
mod tonic_builder;

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    compile("protos", "compiled", true, true)?;

    let graph = generate("compiled")?;

    let mut proto_lib = File::create("src/lib.rs")?;
    display(&graph, &mut proto_lib, &NodeIndex::from(0))?;

    Ok(())
}
