use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn compile(
    input_dir: &str,
    output_dir: &str,
    includes: &[String],
    type_attributes: &[(&str, &str)],
    server: bool,
    client: bool,
) -> Result<(), anyhow::Error> {
    let mut protos = vec![];
    get_protos(protos.as_mut(), Path::new(input_dir))?;

    let mut includes = includes.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();

    let compile_includes: PathBuf = match Path::new(input_dir).parent() {
        None => PathBuf::from("."),
        Some(parent) => parent.to_path_buf(),
    };

    includes.push(compile_includes);

    let descriptor_path = PathBuf::from(output_dir).join("proto_descriptor.bin");

    let mut builder = tonic_build::configure()
        .out_dir(output_dir)
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::pbjson_types")
        .build_client(client)
        .build_server(server);

    for (path, attribute) in type_attributes {
        builder = builder.type_attribute(path, attribute);
    }

    builder
        .compile(protos.as_slice(), includes.as_slice())
        .map_err(anyhow::Error::from)?;

    pbjson_build::Builder::new()
        .out_dir(output_dir)
        .register_descriptors(&std::fs::read(descriptor_path)?)?
        .ignore_unknown_fields()
        .exclude([".google.protobuf"])
        .build(&["."])
        .map_err(anyhow::Error::from)
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
