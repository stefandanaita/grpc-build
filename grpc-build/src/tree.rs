//! Contains a [`Tree`] type that is used to process the dotted package names into
//! directory structured files.

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    iter::FromIterator,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

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
        }
        Ok(())
    }
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
}
