use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use walkdir::WalkDir;

// the use of these `inner` functions is a compile time optimisation. In this case it's probably
// minimal but it improves how the code compiles. The inner functions are not generic, so can be built exactly once
// but the outer functions are generic and must be built for every input type (String, &String, &str, &Path, etc).
// Since the outer function just calls the inner function, this is very cheap, but still provides the ergonomic generic API

pub fn prepare_out_dir(out_dir: impl AsRef<Path>) -> Result<()> {
    fn inner(out_dir: &Path) -> Result<()> {
        if out_dir.exists() {
            fs_err::remove_dir_all(out_dir).with_context(|| {
                format!(
                    "could not remove the output directory: {}",
                    out_dir.display()
                )
            })?;
        }

        fs_err::create_dir_all(out_dir).with_context(|| {
            format!(
                "could not create the output directory: {}",
                out_dir.display()
            )
        })?;

        Ok(())
    }
    inner(out_dir.as_ref())
}

/// Get all the `.proto` files within the provided directory
pub fn get_protos(input: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    fn inner(input: &Path) -> impl Iterator<Item = PathBuf> {
        // TODO: maybe add this?
        // println!("cargo:rerun-if-changed={}", input.display());

        WalkDir::new(input)
            .into_iter()
            .filter_map(|r| r.map_err(|err| println!("cargo:warning={:?}", err)).ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |e| e == "proto"))
            .map(|e| e.path().to_path_buf())
    }
    inner(input.as_ref())
}

/// [`tonic_build::Builder::compile`] outputs all the rust files into the output dir all at the top level.
/// This might not be the most desirable. Running this function converts the file into a more expected directory
/// structure and generates the expected mod file output
pub fn refactor(output: impl AsRef<Path>) -> Result<()> {
    fn inner(output: &Path) -> Result<()> {
        let tree: crate::tree::Tree = fs_err::read_dir(output)?
            .filter_map(|r| r.map_err(|err| println!("cargo:warning={:?}", err)).ok())
            .filter(|e| e.path().extension().map_or(false, |e| e == "rs"))
            .filter(|e| !e.path().ends_with("mod.rs"))
            .map(|e| e.path())
            .collect();

        tree.move_paths(output, OsString::new(), PathBuf::new())?;
        fs_err::write(output.join("mod.rs"), tree.to_string())?;

        Command::new("rustfmt")
            .arg(output.join("mod.rs"))
            .spawn()
            .context("failed to format the mod.rs output")?;

        Ok(())
    }
    inner(output.as_ref())
}
