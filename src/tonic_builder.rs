use std::path::{Path, PathBuf};
use std::{fs, io};
use std::ffi::OsStr;

pub fn compile(
    input_dir: &str,
    output_dir: &str,
    server: bool,
    client: bool,
) -> Result<(), anyhow::Error> {
    let mut protos = vec![];
    get_protos(protos.as_mut(), Path::new(input_dir))?;

    fs::create_dir_all(output_dir)?;
    tonic_build::configure()
        .out_dir(output_dir)
        .build_client(server)
        .build_server(client)
        .compile(protos.as_slice(), &[PathBuf::from(".")])?;

    Ok(())
}

fn get_protos(protos: &mut Vec<PathBuf>, dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                get_protos(protos, &path)?;
            } else {
                if let Some(extension) = path.extension() {
                    if extension == OsStr::new("proto") {
                        protos.push(path.as_path().to_owned());
                    }
                }
            }
        }
    }
    Ok(())
}
