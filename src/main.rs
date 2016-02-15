#[macro_use] extern crate log;
extern crate toml;
extern crate rustc_serialize;

mod lockfile;

use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{self, Command};
use std::fmt;
use std::env;
use std::error::Error;

#[cfg(feature = "logging")]
fn init_logging() {
    extern crate env_logger;
    env_logger::init().unwrap();
    info!("logging enabled");
}
#[cfg(not(feature = "logging"))]
fn init_logging() {}

/// "Stringly typed" error message
#[derive(Debug)]
struct StringError(String);

impl<T: Into<String>> From<T> for StringError {
    fn from(t: T) -> Self {
        StringError(t.into())
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
struct RunError(Box<Error>);

impl<T: Error + 'static> From<T> for RunError {
    fn from(t: T) -> Self {
        RunError(Box::new(t))
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Main entry point.
///
/// Returns a `RunError` on error, which can carry any type that implements `Display`.
fn run() -> Result<(), RunError> {
    // First, validate cmd line arguments
    let mut args = env::args();
    // We can dispose of the first 2 arguments (the executable, and the subcommand passed by cargo)
    args.next().expect("exec name not passed as argument");
    if args.next() == None {
        return Err(StringError::from(
            "subcommand not passed as argument (not running with cargo?)"
        ).into());
    }
    // Then we expect at least one more argument to pass to cargo
    let cargo_cmd = match args.next() {
        Some(cmd) => cmd,
        None => return Err(StringError::from("no cargo command passed").into()),
    };
    // Any other arg is passed along to cargo
    let cargo_args: Vec<_> = args.collect();

    // Helper for running Cargo on a package
    let run_cargo = |pkg: &str| -> Result<(), RunError> {
        let mut cmd = Command::new("cargo");
        cmd.arg(&cargo_cmd).arg("-p").arg(pkg).args(&cargo_args);
        info!("running {:?}", cmd);
        let status = try!(cmd.status());

        if !status.success() {
            return Err(StringError::from(
                format!("cargo exited with error: {}", status)
            ).into());
        }

        Ok(())
    };

    let mut file = try!(File::open("Cargo.lock"));
    let mut string = String::new();
    try!(file.read_to_string(&mut string));
    let lockfile = try!(lockfile::parse(&string));
    debug!("parsed lockfile: {:#?}", lockfile);
    info!("root pkg: {}", &lockfile.root.name);

    // Run the subcommand on the main pkg. This isn't normally done via `-p`, but should work
    // regardless.
    info!("handling root pkg");
    try!(run_cargo(&lockfile.root.name));

    for dep in &lockfile.package {
        if dep.is_local() {
            info!("got local package '{}'", dep.name);
            try!(run_cargo(&dep.name));
        }
    }

    Ok(())
}

fn main() {
    init_logging();

    match run() {
        Ok(()) => {},
        Err(e) => {
            let mut stderr = io::stderr();
            writeln!(stderr, "An error occurred: {}", e).unwrap();
            process::exit(1);
        }
    }
}
