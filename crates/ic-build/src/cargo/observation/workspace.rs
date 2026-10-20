use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
};

/// Render Cargo's workspace-location command without executing it.
///
/// Cargo owns membership, explicit `package.workspace`, exclusions and nested
/// independent workspaces. This selects an offline `locate-project`, without
/// resolving dependencies or installing a toolchain. It never guesses from the
/// directory layout or falls back to the supplied directory.
/// The caller supplies the Cargo command and environment. Existing launcher
/// arguments and working directory are preserved. Execute through the caller's
/// runner and admit successful, complete output before [`decode_workspace_root`].
/// No elapsed-time deadline is added;
/// toolchain auto-installation is disabled for this observation.
///
/// # Errors
/// Returns the filesystem error if the manifest directory cannot be canonicalized.
pub fn workspace_root_command(
    crate_manifest_dir: impl AsRef<Path>,
    mut command: Command,
) -> io::Result<Command> {
    let manifest = crate_manifest_dir
        .as_ref()
        .canonicalize()?
        .join("Cargo.toml");
    command
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args([
            "--offline",
            "locate-project",
            "--workspace",
            "--message-format=json",
        ])
        .arg("--manifest-path")
        .arg(manifest);
    Ok(command)
}

/// Decode successful, complete `cargo locate-project` JSON into its workspace root.
///
/// The response must name an absolute Cargo manifest. This validates Cargo's
/// response shape without guessing membership, checking the returned directory
/// or falling back to the supplied crate directory. Capture policy and typed
/// invocation failures belong to the caller's runner.
///
/// # Errors
/// Returns `InvalidData` for malformed JSON or a nonabsolute/nonmanifest response.
pub fn decode_workspace_root(bytes: &[u8]) -> io::Result<PathBuf> {
    let response: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let manifest = response
        .get("root")
        .and_then(serde_json::Value::as_str)
        .map(Path::new)
        .filter(|path| {
            path.is_absolute() && path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml"))
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Cargo returned no absolute workspace manifest",
            )
        })?;
    manifest.parent().map(Path::to_owned).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Cargo workspace manifest has no parent",
        )
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use ic_host_process::tool::{
        CompletedExecution, ExecutionFailure, OutputLimit, OutputLimits, ToolError, capture_command,
    };
    use std::fs;

    fn execute_workspace(
        directory: &Path,
        limits: OutputLimits,
        command: Command,
    ) -> io::Result<PathBuf> {
        let output = capture_command(&mut workspace_root_command(directory, command)?, limits)
            .and_then(CompletedExecution::require_success)
            .and_then(CompletedExecution::require_complete)
            .map_err(|error| {
                let kind = match error.execution_error().map(|execution| &execution.failure) {
                    Some(ExecutionFailure::Io { source, .. }) => source.kind(),
                    Some(ExecutionFailure::ExitStatus) => io::ErrorKind::InvalidInput,
                    _ => io::ErrorKind::Other,
                };
                io::Error::new(kind, error)
            })?;
        decode_workspace_root(&output.stdout)
    }

    #[test]
    fn discovery_refuses_oversized_complete_json_and_preserves_invocation_errors() {
        let root = crate::fs_fixture::unique_temp_directory("discovery-capture");
        let limits = OutputLimits {
            stdout: OutputLimit::Terminate(64 * 1024),
            stderr: OutputLimit::Terminate(64 * 1024),
            timeout: None,
        };
        let mut response = br#"{"root":"/tmp/Cargo.toml"}"#.to_vec();
        response.resize(64 * 1024, b' ');
        fs::write(root.join("response"), &response).unwrap();
        let mut command = Command::new("/bin/sh");
        command.current_dir(&root).env("SELECTED_RECIPE", "workspace discovery").args([
            "-c",
            "test \"$SELECTED_RECIPE\" = 'workspace discovery' && test \"$RUSTUP_AUTO_INSTALL\" = 0 && cat response",
        ]);
        assert_eq!(
            execute_workspace(&root, limits, command).unwrap(),
            Path::new("/tmp")
        );
        response.push(b' ');
        fs::write(root.join("response"), &response).unwrap();
        let mut command = Command::new("/bin/sh");
        command.current_dir(&root).args(["-c", "cat response"]);
        let error = execute_workspace(&root, limits, command).unwrap_err();
        assert!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<ToolError>()
                .is_some()
        );
        let mut failed = Command::new("/bin/sh");
        failed.args(["-c", "printf refused >&2; exit 23"]);
        let error = execute_workspace(&root, limits, failed).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        let host = error
            .get_ref()
            .unwrap()
            .downcast_ref::<ToolError>()
            .unwrap();
        let evidence = host.evidence().unwrap();
        assert_eq!(evidence.stderr, b"refused");
        assert_eq!(evidence.status.unwrap().code(), Some(23));
        let missing = Command::new(root.join("absent"));
        let error = execute_workspace(&root, limits, missing).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<ToolError>()
                .is_some()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_response_validation_rejects_unusable_roots_without_filesystem_inference() {
        for bytes in [
            b"not json".as_slice(),
            b"{}",
            br#"{"root":null}"#,
            br#"{"root":"relative/Cargo.toml"}"#,
            br#"{"root":"/tmp/NotCargo.toml"}"#,
        ] {
            assert_eq!(
                decode_workspace_root(bytes).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
        assert_eq!(
            decode_workspace_root(br#"{"root":"/not-required-to-exist/Cargo.toml"}"#).unwrap(),
            Path::new("/not-required-to-exist")
        );
    }
}
