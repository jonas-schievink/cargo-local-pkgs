#[macro_use] extern crate log;
extern crate toml;
extern crate rustc_serialize;

mod lockfile;

use std::fs::File;
use std::io::{self, Read, Write, ErrorKind};
use std::process::{self, Command, ExitStatus};
use std::fmt;
use std::env;

#[cfg(feature = "logging")]
fn init_logging() {
    extern crate env_logger;
    env_logger::init().unwrap();
    info!("logging enabled");
}
#[cfg(not(feature = "logging"))]
fn init_logging() {}


/// The kinds of errors that can occur in `run()`.
#[derive(Debug)]
enum RunError {
    /// The lock file didn't exist
    LockFileNotFound,
    /// Lock file exists, but is invalid
    LockFileError(lockfile::ReadError),
    /// I/O error when trying to open the given file
    ///
    /// (less specific than `LockFileNotFound`, but more specific then `IoError`)
    FileError(String, io::Error),
    /// Unknown I/O error
    IoError(io::Error),
    /// Invalid command line arguments (with description)
    InvalidArgument(String),
    /// Invoked subcommand returned an error
    ExecError(ExitStatus),
}

impl From<io::Error> for RunError {
    fn from(e: io::Error) -> Self {
        RunError::IoError(e)
    }
}

impl From<lockfile::ReadError> for RunError {
    fn from(e: lockfile::ReadError) -> Self {
        RunError::LockFileError(e)
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RunError::LockFileNotFound => write!(f, "lock file 'Cargo.lock' not found (try running \
                                                     `cargo generate-lockfile` first)"),
            RunError::LockFileError(ref e) => write!(f, "error reading lockfile: {}", e),
            RunError::FileError(ref name, ref e) => write!(f, "error opening '{}': {}", name, e),
            RunError::IoError(ref e) => write!(f, "{}", e),
            RunError::InvalidArgument(ref e) => write!(f, "invalid arguments: {}", e),
            RunError::ExecError(status) => write!(f, "command exited with error: {}", status),
        }
    }
}

/// Main entry point.
fn run() -> Result<(), RunError> {
    // First, validate cmd line arguments
    let mut args = env::args();
    // We can dispose of the first 2 arguments (the executable, and the subcommand passed by cargo)
    args.next().expect("exec name not passed as argument");
    if args.next() == None {
        return Err(RunError::InvalidArgument("subcommand not passed as argument (not running with \
                                              cargo?)".into()));
    }
    // Then we expect at least one more argument to pass to cargo
    let cargo_cmd = match args.next() {
        Some(cmd) => cmd,
        None => return Err(RunError::InvalidArgument("no cargo command passed".into())),
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
            return Err(RunError::ExecError(status));
        }

        Ok(())
    };

    let mut file = try!(File::open("Cargo.lock").map_err(|e| match e.kind() {
        ErrorKind::NotFound => {
            RunError::LockFileNotFound
        }
        _ => {
            RunError::FileError(String::from("Cargo.lock"), e)
        }
    }));
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
            writeln!(stderr, "an error occurred: {}", e).unwrap();
            process::exit(1);
        }
    }
}
