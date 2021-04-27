use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn compile(
    input_dir: &str,
    output_dir: &str,
    includes: &[String],
    server: bool,
    client: bool,
) -> Result<(), anyhow::Error> {
    let mut protos = vec![];
    get_protos(protos.as_mut(), Path::new(input_dir))?;

    let mut includes = includes.iter().map(|i| PathBuf::from(i)).collect::<Vec<PathBuf>>();

    let compile_includes: PathBuf = match Path::new(input_dir).parent() {
        None => PathBuf::from("."),
        Some(parent) => parent.to_path_buf(),
    };

    includes.push(compile_includes);

    tonic_build::configure()
        .out_dir(output_dir)
        .build_client(client)
        .build_server(server)
        .compile(protos.as_slice(), includes.as_slice())?;

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
