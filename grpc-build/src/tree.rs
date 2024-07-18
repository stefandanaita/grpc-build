//! Contains a [`Tree`] type that is used to process the dotted package names into
//! directory structured files.

use std::{
    collections::{BTreeSet, HashMap},
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    iter::FromIterator,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use fs_err::OpenOptions;

#[derive(Default, Debug, PartialEq)]
pub struct Tree(pub(crate) HashMap<PathBuf, Tree>);

impl Extend<PathBuf> for Tree {
    fn extend<T: IntoIterator<Item = PathBuf>>(&mut self, iter: T) {
        for path in iter {
            self.insert_path(path)
        }
    }
}

impl FromIterator<PathBuf> for Tree {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(iter: T) -> Self {
        let mut tree = Tree::default();
        tree.extend(iter);
        tree
    }
}

impl Tree {
    /// Given a file path that is `.` separated, it loads it into the tree.
    pub fn insert_path(mut self: &mut Self, path: PathBuf) {
        for comp in path.file_stem().unwrap().to_str().unwrap().split('.') {
            self = self.0.entry(PathBuf::from(comp)).or_default()
        }
    }

    /// Generates the module at the root level of the tree
    pub fn generate_module(&self) -> String {
        let mut module = String::from("// Module generated with `grpc_build`\n");
        let sorted: BTreeSet<_> = self.0.keys().collect();
        for k in sorted {
            module.push_str(&format!("pub mod {};\n", k.display()));
        }

        module.push_str("\n");
        module
    }

    /// Loop through the tree, determining where all the files should be
    /// and moving them there
    pub fn move_paths(&self, root: &Path, filename: OsString, output: PathBuf) -> Result<()> {
        if self.0.is_empty() {
            fs_err::create_dir_all(root.join(&output).parent().unwrap())
                .with_context(|| format!("could not create dir for file {}", output.display()))?;

            let from = root.join(filename.add("rs"));
            let to = root.join(output.with_extension("rs"));
            fs_err::rename(&from, &to).with_context(|| {
                format!("could not move {} to {}", from.display(), to.display())
            })?;
        } else {
            for (k, tree) in &self.0 {
                tree.move_paths(root, filename.add(k), output.join(k))?;
            }
            
            if !filename.is_empty() {
                self.create_module_file(root, filename, output)?;
            }
        }
        Ok(())
    }

    fn create_module_file(
        &self,
        root: &Path,
        filename: OsString,
        output: PathBuf,
    ) -> Result<(), anyhow::Error> {
        let maybe_proto_file_name = root.join(filename.add("rs"));
        let dest_tmp_file_name = root.join(output.with_extension("tmp.rs"));
        let final_dest_name = root.join(output.with_extension("rs"));

        // Write a temporary file with the module contents
        let modules = self.generate_module();
        fs_err::write(&dest_tmp_file_name, modules)
            .with_context(|| format!("could not write to file {}", final_dest_name.display()))?;

        // If there is a proto file in this directory, we append its contents to the already written temporary module file
        if fs_err::metadata(&maybe_proto_file_name)
            .map(|m| m.is_file())
            .unwrap_or(false)
        {
            merge_file_into(&maybe_proto_file_name, &dest_tmp_file_name)?;
        }

        // Finally, move the temporary file to the final destination
        fs_err::rename(&dest_tmp_file_name, &final_dest_name).with_context(|| {
            format!(
                "could not move {} to {}",
                dest_tmp_file_name.display(),
                final_dest_name.display()
            )
        })?;

        Ok(())
    }
}

fn merge_file_into(from: &PathBuf, to: &PathBuf) -> Result<(), anyhow::Error> {
    if from == to {
        bail!("Merging files, source and destination files are the same");
    }

    let mut source = OpenOptions::new()
        .read(true)
        .open(from)
        .with_context(|| format!("Failed to open not source file {}", to.display()))?;

    let mut dest = OpenOptions::new()
        .create_new(false)
        .write(true)
        .append(true)
        .open(to)
        .with_context(|| format!("Failed to open the destination file {}", from.display()))?;

    std::io::copy(&mut source, &mut dest).with_context(|| {
        format!(
            "could not copy contents from {} to {}",
            from.display(),
            to.display()
        )
    })?;

    fs_err::remove_file(&from)
        .with_context(|| format!("could not remove file {}", from.display()))?;
    Ok(())
}

// private helper trait
trait OsStrExt {
    fn add(&self, add: impl AsRef<OsStr>) -> OsString;
}

impl OsStrExt for OsStr {
    /// Adds `add` to the [`OsStr`], returning a new [`OsString`]. If there already exists data in the string,
    /// this puts a `.` separator inbetween
    fn add(&self, add: impl AsRef<OsStr>) -> OsString {
        let mut _self = self.to_owned();
        if !_self.is_empty() {
            _self.push(".");
        }
        _self.push(add);
        _self
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, tree) in &self.0 {
            write!(f, "pub mod {}", k.display())?;
            if tree.0.is_empty() {
                write!(f, ";")?;
            } else {
                write!(f, "{{{}}}", tree)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::Tree;

    macro_rules! tree {
        ($($key:literal : $val:expr,)*) => {
            Tree(HashMap::from_iter([
                $(
                    (PathBuf::from($key), $val)
                ),*
            ]))
        };
    }

    #[test]
    fn build_tree() {
        let tree: Tree = [
            "grpc_build.client.helloworld.rs",
            "grpc_build.request.helloworld.rs",
            "grpc_build.response.helloworld.rs",
            "google.protobuf.foo.rs",
            "google.protobuf.bar.rs",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect();

        let expected = tree! {
            "grpc_build": tree! {
                "client": tree! {
                    "helloworld": tree!{},
                },
                "request": tree! {
                    "helloworld": tree!{},
                },
                "response": tree! {
                    "helloworld": tree!{},
                },
            },
            "google": tree! {
                "protobuf": tree! {
                    "foo": tree!{},
                    "bar": tree!{},
                },
            },
        };

        assert_eq!(tree, expected);
    }

    #[test]
    fn generate_module_returns_at_current_level() {
        let tree: Tree = [
            "grpc_build.client.helloworld.rs",
            "grpc_build.request.helloworld.rs",
            "grpc_build.response.helloworld.rs",
            "google.protobuf.foo.rs",
            "google.protobuf.bar.rs",
            "alphabet.foo.rs",
            "hello.rs",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect();

        let expected = "// Module generated with `grpc_build`
pub mod alphabet;
pub mod google;
pub mod grpc_build;
pub mod hello;

";

        assert_eq!(tree.generate_module(), expected);
    }

    #[test]
    fn generate_module_returns_at_current_level_nested() {
        let tree: Tree = [
            "grpc_build.client.helloworld.rs",
            "grpc_build.request.helloworld.rs",
            "grpc_build.response.helloworld.rs",
            "google.protobuf.foo.rs",
            "google.protobuf.bar.rs",
            "alphabet.foo.rs",
            "hello.rs",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect();

        let inner_tree = tree.0.get(&PathBuf::from("grpc_build")).unwrap();
        let expected = "// Module generated with `grpc_build`
pub mod client;
pub mod request;
pub mod response;

";

        assert_eq!(inner_tree.generate_module(), expected);
    }
}
