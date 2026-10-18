//! Build a caller-selected native example using Cargo's reported executable path.
//!
//! Run with WORKSPACE PACKAGE EXAMPLE. The caller keeps custody of sources,
//! configuration, tools and the mutable target through executable consumption.
//! Command environment, admission and child cleanup are explicit caller choices.

use ic_build::cargo::{
    native::{NativeExecutableSpec, NativeTargetKind, build_native_executable},
    selection::CargoSelection,
};
use ic_host_process::child::OwnedChild;
use std::{error::Error, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut arguments = std::env::args_os().skip(1);
    let workspace = PathBuf::from(
        arguments
            .next()
            .ok_or("expected WORKSPACE PACKAGE EXAMPLE")?,
    )
    .canonicalize()?;
    let package = arguments
        .next()
        .ok_or("expected PACKAGE")?
        .into_string()
        .map_err(|_| "PACKAGE must be UTF-8")?;
    let target_name = arguments
        .next()
        .ok_or("expected EXAMPLE")?
        .into_string()
        .map_err(|_| "EXAMPLE must be UTF-8")?;
    let spec = NativeExecutableSpec {
        package: &package,
        target_name: &target_name,
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
