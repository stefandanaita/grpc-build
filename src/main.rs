use grpc_build::build;

#[derive(structopt::StructOpt)]
pub enum Command {
    Build {
        #[structopt(long)]
        in_dir: String,

        #[structopt(long)]
        out_dir: String,

        #[structopt(short = "client", long = "build_client")]
        build_client: bool,

        #[structopt(short = "server", long = "build_server")]
        build_server: bool,

        #[structopt(short = "force", long = "force")]
        force: bool,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let command = <Command as paw::ParseArgs>::parse_args()?;

    match command {
        Command::Build {
            in_dir,
            out_dir,
            build_client,
            build_server,
            force,
        } => {
            build(&in_dir, &out_dir, build_client, build_server, force)?;
        }
    }

    Ok(())
}
