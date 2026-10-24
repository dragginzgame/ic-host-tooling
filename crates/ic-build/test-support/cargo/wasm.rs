//! Shared real-Wasm workspace for integration targets.

use std::{io, path::Path};

/// Write and lock an independent, dependency-free cdylib workspace.
pub fn write_workspace(root: &Path) -> io::Result<()> {
    super::cargo::write_fixture_package(
        root,
        "[workspace]\n[lib]\ncrate-type = [\"cdylib\"]\n",
        "pub fn value() -> u32 { 7 }\n",
    )?;
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = std::process::Command::new(cargo)
        .current_dir(root)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args(["generate-lockfile", "--offline"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "Cargo fixture lock preparation failed: {status}"
        )));
    }
    Ok(())
}

/// Construct a real-Wasm test recipe with this fixture adapter's selected environment.
pub fn spec(root: &Path, target: &Path, packages: &[&str]) -> ic_build::artifacts::WasmBuildSpec {
    ic_build::artifacts::WasmBuildSpec::new(
        root,
        target,
        packages,
        super::cargo::environment(),
        CONFIGURATION_LIMITS,
    )
}

// This caller admits small configuration trees; libraries do not select this policy.
const CONFIGURATION_LIMITS: ic_build::cargo::environment::ConfigurationLimits =
    ic_build::cargo::environment::ConfigurationLimits {
        file_bytes: 1024 * 1024,
        total_bytes: 16 * 1024 * 1024,
        files: 128,
        depth: 32,
    };
