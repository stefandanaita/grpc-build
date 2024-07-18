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
pub fn get_protos(input: impl AsRef<Path>, follow_links: bool) -> impl Iterator<Item = PathBuf> {
    fn inner(input: &Path, follow_links: bool) -> impl Iterator<Item = PathBuf> {
        // TODO: maybe add this?
        // println!("cargo:rerun-if-changed={}", input.display());

        WalkDir::new(input)
            .follow_links(follow_links)
            .into_iter()
            .filter_map(|r| r.map_err(|err| println!("cargo:warning={:?}", err)).ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |e| e == "proto"))
            .map(|e| e.path().to_path_buf())
    }
    inner(input.as_ref(), follow_links)
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
        fs_err::write(output.join("mod.rs"), tree.generate_module())?;

        Command::new("rustfmt")
            .arg(output.join("mod.rs"))
            .spawn()
            .context("failed to format the mod.rs output")?;

        Ok(())
    }
    inner(output.as_ref())
}

#[cfg(test)]
mod test {
    use super::refactor;

    #[test]
    fn refactor_test_moves_files_to_correct_place() {
        let files = vec![
            "root.pak.a1.rs",
            "root.pak.a2.rs",
            "root.pak.rs",
            "root.now.deeply.nested.rs",
            "root.rs",
            "other.rs",
        ];

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        // create files
        for file in &files {
            let path = temp_dir_path.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::File::create(path.clone()).unwrap();
            // write file name as its content
            std::fs::write(path.clone(), format!("// {} contents", file)).unwrap();
        }

        let expected_file_contents = vec![
            ("root/pak/a1.rs", vec!["// root.pak.a1.rs contents"]),
            ("root/pak/a2.rs", vec!["// root.pak.a2.rs content"]),
            (
                "root/pak.rs",
                vec!["pub mod a1;", "pub mod a2;", "// root.pak.rs contents"],
            ),
            ("root/now.rs", vec!["pub mod deeply;"]),
            ("root/now/deeply.rs", vec!["pub mod nested;"]),
            (
                "root/now/deeply/nested.rs",
                vec!["// root.now.deeply.nested.rs contents"],
            ),
            (
                "root.rs",
                vec!["pub mod pak;", "pub mod now;", "// root.rs contents"],
            ),
            ("mod.rs", vec!["pub mod other;", "pub mod root;"]),
            ("other.rs", vec!["// other.rs contents"]),
        ];

        // Act
        refactor(&temp_dir_path).unwrap();

        // check if files are moved and contents are correct
        for (file, contents) in &expected_file_contents {
            let path = temp_dir_path.join(file);
            assert!(path.exists());
            let content = std::fs::read_to_string(path).unwrap();
            for line in contents {
                assert!(
                    content.contains(line),
                    "{} does not contain {}",
                    content,
                    line
                );
            }
        }
    }
}
