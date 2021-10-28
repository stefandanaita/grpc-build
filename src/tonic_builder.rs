use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs, io};
use tonic_build::Builder;

pub fn compile(
    input_dir: &str,
    output_dir: &str,
    server: bool,
    client: bool,
    user_config: impl FnOnce(Builder) -> Builder,
) -> Result<(), anyhow::Error> {
    let mut protos = vec![];
    get_protos(protos.as_mut(), Path::new(input_dir))?;

    let compile_includes: PathBuf = match Path::new(input_dir).parent() {
        None => PathBuf::from("."),
        Some(parent) => parent.to_path_buf(),
    };

    user_config(
        tonic_build::configure()
            .out_dir(output_dir)
            .build_client(client)
            .build_server(server),
    )
    .compile(protos.as_slice(), &[compile_includes])?;

    Ok(())
}

fn get_protos(protos: &mut Vec<PathBuf>, dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                get_protos(protos, &path)?;
            } else if let Some(extension) = path.extension() {
                if extension == OsStr::new("proto") {
                    protos.push(path.as_path().to_owned());
                }
            }
        }
    }
    Ok(())
}
