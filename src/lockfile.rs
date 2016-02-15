//! Utility for reading a `Cargo.lock`

// TODO: Maybe make this a reusable crate? If there's demand, tell me!

// FIXME: This is a "stringly typed" representation. A real lockfile has more restrictions we could
// possibly encode as well (in different types, perhaps?), but that would require more work and more
// dependencies (such as semver)

use toml::{self, ParserError};
use rustc_serialize::Decodable;

use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum ReadError {
    TomlErrors(Vec<ParserError>),
    DecodeError(toml::DecodeError),
}

impl From<Vec<ParserError>> for ReadError {
    fn from(errs: Vec<ParserError>) -> Self {
        ReadError::TomlErrors(errs)
    }
}

impl From<toml::DecodeError> for ReadError {
    fn from(e: toml::DecodeError) -> Self {
        ReadError::DecodeError(e)
    }
}

impl Error for ReadError {
    fn description(&self) -> &str {
        match *self {
            ReadError::TomlErrors(_) => "malformed TOML",
            ReadError::DecodeError(_) => "invalid lockfile",
        }
    }
}

impl Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::TomlErrors(ref errs) => {
                for err in errs {
                    try!(writeln!(f, "{}", err));
                }
            }
            ReadError::DecodeError(ref e) => {
                try!(writeln!(f, "{}", e));
            }
        }

        Ok(())
    }
}

/// A package as found on crates.io and in the lockfile. Can contain many crates.
///
/// The `Cargo.lock` contains a main package, and each dependency is also a package. Each package
/// can depend on any number of other packages (cycles are ignored).
#[derive(RustcDecodable, Debug)]
pub struct Package {
    /// The name of this package, for example "aho-corasick".
    pub name: String,
    /// The package version as an unchecked string. Normally, this is a valid semver version, but
    /// we don't check for that.
    pub version: String,
    /// The package's source string. If this is `None`, the package is considered local.
    ///
    /// For example: "registry+https://github.com/rust-lang/crates.io-index"
    pub source: Option<String>,
    /// The list of dependencies, in string form.
    pub dependencies: Vec<String>,
}

impl Package {
    pub fn is_local(&self) -> bool {
        self.source.is_none()
    }
}

/// Represents a parsed, but unvalidated lockfile
#[derive(RustcDecodable, Debug)]
pub struct Lockfile {
    /// The `[root]` entry in the lockfile represents the "locked" main package
    pub root: Package,
    /// The `[[package]]` array contains all transitive dependencies of the main package
    pub package: Vec<Package>,
}

/// Parse a `Cargo.lock` lockfile given as a `&str`.
pub fn parse(s: &str) -> Result<Lockfile, ReadError> {
    let val = try!(s.parse());
    let mut decoder = toml::Decoder::new(val);
    let lockfile = try!(Lockfile::decode(&mut decoder));

    Ok(lockfile)
}
