//! Build a caller-selected native example using Cargo's reported executable path.
//!
//! Run with WORKSPACE PACKAGE EXAMPLE. The caller keeps custody of sources,
//! configuration, tools and the mutable target through executable consumption.
//! Command environment, admission and child cleanup are explicit caller choices.

use clap::{Arg, Command as Cli, value_parser};
use ic_build::cargo::{
    native::{NativeExecutableSpec, NativeTargetKind, build_native_executable},
    selection::CargoSelection,
};
use ic_host_process::child::OwnedChild;
use std::{error::Error, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let arguments = Cli::new("native_executable")
        .about("Build a caller-selected native Cargo example")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("workspace")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(Arg::new("package").required(true))
        .arg(Arg::new("example").required(true))
        .get_matches();
    let workspace = arguments
        .get_one::<PathBuf>("workspace")
        .unwrap()
        .canonicalize()?;
    let package = arguments.get_one::<String>("package").unwrap();
    let target_name = arguments.get_one::<String>("example").unwrap();
    let spec = NativeExecutableSpec {
        package,
        target_name,
        target_kind: NativeTargetKind::Example,
        selection: CargoSelection::default(),
        max_message_bytes: 16 * 1024 * 1024,
    };
    let built = build_native_executable(
        &spec,
        || {
            let mut cargo = Command::new("cargo");
            cargo.current_dir(&workspace);
            cargo
        },
        std::io::stderr(),
        || Ok(()),
        OwnedChild::spawn_direct,
        || false,
    )?;
    println!("{}", built.executable.display());
    Ok(())
}
