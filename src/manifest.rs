//! Utility for getting the package name from a `Cargo.toml`

use toml::{self, ParserError};

use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum ReadError {
    /// Malformed TOML
    TomlErrors(Vec<ParserError>),
    /// Key `package.name` not specified
    NoPackageName,
    /// The package name is not a string, but a different TOML value
    NotAString(&'static str),
}

impl From<Vec<ParserError>> for ReadError {
    fn from(errs: Vec<ParserError>) -> Self {
        ReadError::TomlErrors(errs)
    }
}

impl Error for ReadError {
    fn description(&self) -> &str {
        match *self {
            ReadError::TomlErrors(_) => "malformed TOML",
            ReadError::NoPackageName => "manifest doesn't specify a package name",
            ReadError::NotAString(_) => "package name is not a string",
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
            ReadError::NoPackageName => {
                try!(writeln!(f, "{}", self.description()));
            }
            ReadError::NotAString(but) => {
                try!(writeln!(f, "package name is a {} (string required)", but));
            }
        }

        Ok(())
    }
}

pub fn package_name_from_manifest(manifest: &str) -> Result<String, ReadError> {
    let val: toml::Value = try!(manifest.parse());
    match val.lookup("package.name") {
        Some(name_val) => match name_val.as_str() {
            Some(s) => Ok(String::from(s)),
            None => Err(ReadError::NotAString(name_val.type_str())),
        },
        None => Err(ReadError::NoPackageName),
    }
}
