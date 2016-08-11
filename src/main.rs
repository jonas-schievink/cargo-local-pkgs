#[macro_use] extern crate log;
extern crate clap;
extern crate serde_json;

use clap::{ArgMatches, App, Arg, AppSettings};
use serde_json::Value;

use std::io::{self, Write};
use std::process::{self, Command};
use std::error::Error;
use std::str::FromStr;

#[cfg(feature = "logging")]
fn init_logging() {
    extern crate env_logger;
    env_logger::init().unwrap();
    info!("logging enabled");
}
#[cfg(not(feature = "logging"))]
fn init_logging() {}

/// Collects local packages and returns them as a vector of package names to be passed to
/// `cargo -p <pkg>`.
fn collect_local_pkgs(cargo_args: &[&str]) -> Result<Vec<String>, Box<Error>> {
    let mut cargo = Command::new("cargo");
    cargo.arg("metadata");
    cargo.args(cargo_args);
    debug!("invoking {:?}", cargo);

    let stdout = String::from_utf8(try!(cargo.output()).stdout).unwrap();
    let stdout = stdout.lines().last().unwrap();
    debug!("cargo metadata: {}", stdout);

    let mut local_pkgs = Vec::new();

    let main_obj = try!(Value::from_str(&stdout));
    let main_obj = main_obj.as_object().unwrap();
    let pkg_array = main_obj["packages"].as_array().unwrap();
    for pkg_obj in pkg_array {
        let pkg_obj = pkg_obj.as_object().unwrap();

        if pkg_obj["source"].is_null() {
            let name = pkg_obj["name"].as_string().unwrap();
            local_pkgs.push(name.into());
        }
    }

    debug!("got local package list: {:?}", local_pkgs);

    Ok(local_pkgs)
}

/// Main entry point.
fn run(args: &ArgMatches) -> Result<(), Box<Error>> {
    // When used through cargo, the subcommand is always `local-pkgs`.
    let (local_pkgs, subcmd_args) = args.subcommand();
    info!("subcmd: {} {:?}", local_pkgs, subcmd_args);
    let mut arg_iter = subcmd_args.unwrap().values_of("").unwrap();
    let cargo_cmd = arg_iter.next().unwrap();
    let subcmd_args = arg_iter.collect::<Vec<_>>();
    let cargo_args = args.value_of("CARGO_ARGS").map(|cargs| cargs.split_whitespace().collect::<Vec<_>>()).unwrap_or(subcmd_args);

    // Helper for running Cargo on a package
    let run_cargo = |pkg: &str| -> Result<(), Box<Error>> {
        let mut cmd = Command::new("cargo");
        cmd.arg(&cargo_cmd).arg("-p").arg(pkg).args(&cargo_args);
        info!("running {:?}", cmd);
        let status = try!(cmd.status());

        if !status.success() {
            return Err(format!("subcommand exited with error code: {:?}", status).into());
        }

        Ok(())
    };

    let local_pkgs = try!(collect_local_pkgs(&cargo_args));

    // Now run the subcommand for all packages we collected
    for pkg in &local_pkgs {
        try!(run_cargo(pkg));
    }

    Ok(())
}

fn main() {
    init_logging();

    let args = App::new("local-pkgs")
        //.setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::AllowExternalSubcommands)
        .arg(Arg::with_name("CARGO_ARGS")
            .long("cargo-args")
            .help("Specifies the arguments to pass to all internal cargo invocations. This is \
                   normally inferred from all passed args."))
        .get_matches();

    match run(&args) {
        Ok(()) => {},
        Err(e) => {
            let mut stderr = io::stderr();
            writeln!(stderr, "an error occurred: {}", e).unwrap();
            process::exit(1);
        }
    }
}
