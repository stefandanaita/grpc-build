use grpc_build::build;

#[derive(structopt::StructOpt)]
pub enum Command {
    Build {
        #[structopt(long)]
        in_dir: String,

        #[structopt(long)]
        out_dir: String,

        #[structopt(short = "I", long = "include")]
        includes: Vec<String>,

        #[structopt(short = "T", long = "type_attribute")]
        type_attributes: Vec<String>,

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
            includes,
            type_attributes,
            build_client,
            build_server,
            force,
        } => {
            build(
                &in_dir,
                &out_dir,
                includes.as_slice(),
                &type_attributes
                    .iter()
                    .map(|type_attribute| type_attribute.split_once(":"))
                    .collect::<Option<Vec<(&str, &str)>>>()
                    .ok_or_else(|| {
                        anyhow::anyhow!("type attribute must have ':' between path and attribute")
                    })?,
                build_server,
                build_client,
                force,
            )?;
        }
    }

    Ok(())
}
