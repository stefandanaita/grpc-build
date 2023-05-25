use anyhow::Result;
use clap::Parser;
use grpc_build::Builder;

#[derive(Parser)]
pub enum Command {
    Build {
        #[arg(long)]
        in_dir: String,

        #[arg(long)]
        out_dir: String,

        #[arg(short = 'c', long = "build_client")]
        build_client: bool,

        #[arg(short = 's', long = "build_server")]
        build_server: bool,

        #[arg(short = 'f', long = "force")]
        force: bool,
    },
}

fn main() -> Result<()> {
    let command = Command::try_parse()?;

    match command {
        Command::Build {
            in_dir,
            out_dir,
            build_client,
            build_server,
            force,
        } => Builder::new()
            .build_client(build_client)
            .build_server(build_server)
            .force(force)
            .out_dir(out_dir)
            .build(in_dir),
    }
}
