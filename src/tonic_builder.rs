use std::path::Path;
use std::{fs, io};

pub fn compile(
    input_dir: &str,
    output_dir: &str,
    server: bool,
    client: bool,
) -> Result<(), anyhow::Error> {
    let mut protos: Vec<String> = vec![];
    get_protos(protos.as_mut(), Path::new(input_dir))?;

    fs::create_dir_all(output_dir)?;
    tonic_build::configure()
        .out_dir(output_dir)
        .build_client(server)
        .build_server(client)
        .compile(protos.as_slice(), &[String::from(".")])?;

    Ok(())
}

fn get_protos(protos: &mut Vec<String>, dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                get_protos(protos, &path)?;
            } else {
                let os_ext = &path.extension();

                if os_ext.is_some() {
                    let ext = os_ext.unwrap().to_str().unwrap();

                    if ext.contains("proto") {
                        let path_str = path.as_path().to_str().unwrap();
                        protos.push(String::from(path_str));
                    }
                }
            }
        }
    }
    Ok(())
}
