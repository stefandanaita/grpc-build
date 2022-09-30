use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// A mirror of [`tonic_build::Builder`] for our own control
pub struct Builder {
    pub(crate) tonic: tonic_build::Builder,
    pub(crate) prost: prost_build::Config,
    pub(crate) protoc_args: Vec<OsString>,
    pub(crate) out_dir: Option<PathBuf>,
    pub(crate) force: bool,
    pub(crate) default_module_name: Option<String>,
    pub(crate) follow_links: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            tonic: tonic_build::configure(),
            prost: Default::default(),
            protoc_args: Default::default(),
            out_dir: None,
            force: false,
            default_module_name: None,
            follow_links: false,
        }
    }
}

impl Builder {
    pub(crate) fn get_out_dir(&self) -> Result<PathBuf, anyhow::Error> {
        self.out_dir.clone().map(Ok).unwrap_or_else(|| {
            std::env::var_os("OUT_DIR")
                .ok_or_else(|| anyhow::anyhow!("could not determine $OUT_DIR"))
                .map(Into::into)
        })
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Follow symbolic links when finding .proto files.
    pub fn follow_links(mut self, follow_links: bool) -> Self {
        self.follow_links = follow_links;
        self
    }

    pub fn out_dir(mut self, out_dir: impl AsRef<Path>) -> Self {
        self.out_dir = Some(out_dir.as_ref().to_owned());
        self
    }

    /// Configures what filename protobufs with no package definition are written to.
    pub fn default_module_name(mut self, name: impl AsRef<str>) -> Self {
        self.default_module_name = Some(name.as_ref().to_string());
        self
    }

    /// Enable or disable gRPC client code generation.
    pub fn build_client(mut self, enable: bool) -> Self {
        self.tonic = self.tonic.build_client(enable);
        self
    }

    /// Enable or disable gRPC server code generation.
    pub fn build_server(mut self, enable: bool) -> Self {
        self.tonic = self.tonic.build_server(enable);
        self
    }

    /// Declare externally provided Protobuf package or type.
    ///
    /// Passed directly to `prost_build::Config.extern_path`.
    /// Note that both the Protobuf path and the rust package paths should both be fully qualified.
    /// i.e. Protobuf paths should start with "." and rust paths should start with "::"
    pub fn extern_path(mut self, proto_path: impl AsRef<str>, rust_path: impl AsRef<str>) -> Self {
        self.prost.extern_path(
            proto_path.as_ref().to_string(),
            rust_path.as_ref().to_string(),
        );
        self
    }

    /// Add additional attribute to matched messages, enums, and one-offs.
    ///
    /// Passed directly to `prost_build::Config.field_attribute`.
    pub fn field_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.prost
            .field_attribute(path.as_ref(), attribute.as_ref());
        self
    }

    /// Add additional attribute to matched messages, enums, and one-offs.
    ///
    /// Passed directly to `prost_build::Config.type_attribute`.
    pub fn type_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.prost.type_attribute(path.as_ref(), attribute.as_ref());
        self
    }

    /// Add additional attribute to matched server `mod`s. Matches on the package name.
    pub fn server_mod_attribute<P: AsRef<str>, A: AsRef<str>>(
        mut self,
        path: P,
        attribute: A,
    ) -> Self {
        self.tonic = self.tonic.server_mod_attribute(path, attribute);
        self
    }

    /// Add additional attribute to matched service servers. Matches on the service name.
    pub fn server_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.tonic = self.tonic.server_attribute(path, attribute);
        self
    }

    /// Add additional attribute to matched client `mod`s. Matches on the package name.
    pub fn client_mod_attribute<P: AsRef<str>, A: AsRef<str>>(
        mut self,
        path: P,
        attribute: A,
    ) -> Self {
        self.tonic = self.tonic.client_mod_attribute(path, attribute);
        self
    }

    /// Add additional attribute to matched service clients. Matches on the service name.
    pub fn client_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.tonic = self.tonic.client_attribute(path, attribute);
        self
    }

    /// Configure Prost `protoc_args` build arguments.
    ///
    /// Note: Enabling `--experimental_allow_proto3_optional` requires protobuf >= 3.12.
    pub fn protoc_arg<A: AsRef<str>>(mut self, arg: A) -> Self {
        self.protoc_args.push(arg.as_ref().into());
        self
    }

    /// Enable or disable directing Prost to compile well-known protobuf types instead
    /// of using the already-compiled versions available in the `prost-types` crate.
    ///
    /// This defaults to `false`.
    pub fn compile_well_known_types(mut self, compile_well_known_types: bool) -> Self {
        if compile_well_known_types {
            self.prost.compile_well_known_types();
        };
        self
    }

    /// Configures the optional module filename for easy inclusion of all generated Rust files
    ///
    /// If set, generates a file (inside the `OUT_DIR` or `out_dir()` as appropriate) which contains
    /// a set of `pub mod XXX` statements combining to load all Rust files generated.  This can allow
    /// for a shortcut where multiple related proto files have been compiled together resulting in
    /// a semi-complex set of includes.
    pub fn include_file(mut self, path: impl AsRef<Path>) -> Self {
        self.prost.include_file(path.as_ref());
        self
    }
}
