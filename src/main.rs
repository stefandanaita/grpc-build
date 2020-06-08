use crate::graph_layout::{display, generate};
use crate::tonic_builder::compile;
use petgraph::graph::NodeIndex;
use std::fs::File;

mod graph_layout;
mod tonic_builder;

#[derive(structopt::StructOpt)]
pub enum Command {
    Build {
        #[structopt(long)]
        in_dir: Option<String>,

        #[structopt(long)]
        out_dir: Option<String>,

        #[structopt(short = "client", long = "build_client")]
        build_client: bool,

        #[structopt(short = "server", long = "build_server")]
        build_server: bool,
    },
}

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    let command = <Command as paw::ParseArgs>::parse_args()?;

    match command {
        Command::Build {
            in_dir,
            out_dir,
            build_client,
            build_server,
        } => {
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
        }
    }

    Ok(())
}
