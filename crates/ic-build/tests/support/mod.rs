// Each integration target uses only the fixtures it needs.
#![allow(dead_code)]

#[path = "../../test-support/fs/mod.rs"]
pub mod fs_fixture;

#[path = "../../test-support/cargo/mod.rs"]
pub mod cargo;

#[path = "../../test-support/cargo/wasm.rs"]
pub mod wasm;

use std::path::{Path, PathBuf};

pub fn discover_workspace(manifest_dir: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    use ic_build::cargo::observation::{decode_workspace_root, workspace_root_command};
    use ic_host_process::tool::{
        CompletedExecution, ExecutionFailure, OutputLimit, OutputLimits, capture_command,
    };

    let mut command = workspace_root_command(
        manifest_dir,
        std::process::Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())),
    )?;
    let output = capture_command(
        &mut command,
        OutputLimits {
            stdout: OutputLimit::Terminate(64 * 1024),
            stderr: OutputLimit::Terminate(64 * 1024),
            timeout: None,
        },
    )
    .and_then(CompletedExecution::require_success)
    .and_then(CompletedExecution::require_complete)
    .map_err(|error| {
        let kind = match error.execution_error().map(|execution| &execution.failure) {
            Some(ExecutionFailure::Io { source, .. }) => source.kind(),
            Some(ExecutionFailure::ExitStatus) => std::io::ErrorKind::InvalidInput,
            _ => std::io::ErrorKind::Other,
        };
        std::io::Error::new(kind, error)
    })?;
    decode_workspace_root(&output.stdout)
}

pub fn wasm_path(target_dir: &Path, crate_name: &str, profile_target_dir: &str) -> PathBuf {
    target_dir
        .join("wasm32-unknown-unknown")
        .join(profile_target_dir)
        .join(format!("{crate_name}.wasm"))
}
