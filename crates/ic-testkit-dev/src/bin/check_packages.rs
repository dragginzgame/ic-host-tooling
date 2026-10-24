//! Qualify the exact coordinated archives through IC Build's owning engine.

use std::{
    error::Error,
    fmt::Write as _,
    path::Path,
    process::{Command, ExitCode},
};

use ic_build::cargo::packages::{
    PackageArchiveSpec, PackageCommandError, PackageQualificationLimits, qualify_package_archives,
};
use ic_build::cargo::selection::CargoFeatureSelection;
use ic_host_process::tool::{OutputLimit, OutputLimits};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", failure_report(error.as_ref()));
            ExitCode::FAILURE
        }
    }
}

fn failure_report(error: &(dyn Error + 'static)) -> String {
    let mut report = format!("Package qualification failed: {error}");
    let mut cause = Some(error);
    while let Some(error) = cause {
        if let Some(command) = error.downcast_ref::<PackageCommandError>() {
            if let Some(evidence) = command
                .execution
                .as_ref()
                .and_then(|error| error.evidence())
            {
                for (label, bytes) in [("stdout", &evidence.stdout), ("stderr", &evidence.stderr)] {
                    let tail = &bytes[bytes.len().saturating_sub(8192)..];
                    let _ = writeln!(report, "\n{label} (last {} captured bytes):", tail.len());
                    for line in String::from_utf8_lossy(tail).lines() {
                        let _ = writeln!(report, "{}", ic_host_text::escape::terminal_text(line));
                    }
                }
            }
            break;
        }
        cause = error.source();
    }
    report
}

fn run() -> Result<(), Box<dyn Error>> {
    clap::Command::new("check_packages")
        .about("Qualify the selected workspace package archives through IC Build")
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("development package requires the repository workspace")?;
    let target = workspace.join("target");
    std::fs::create_dir_all(target.join("package"))?;
    let evidence = ic_host_fs::directory::create_unique_directory(
        &target,
        "package-qualification",
        0o700,
        None,
    )?;
    eprintln!("Package qualification evidence: {}", evidence.display());
    let version = env!("CARGO_PKG_VERSION");
    let packages = include_str!("../../../../ci/public-packages.txt")
        .lines()
        .map(|name| PackageArchiveSpec {
            name,
            version,
            patch_registry: true,
        })
        .collect::<Vec<_>>();
    eprintln!(
        "Current archive stage: {}",
        evidence.join("current-stage").display()
    );
    qualify_package_archives(
        workspace,
        &target.join("package"),
        &evidence,
        &packages,
        &CargoFeatureSelection {
            all_features: true,
            ..CargoFeatureSelection::default()
        },
        PackageQualificationLimits {
            process: OutputLimits {
                stdout: OutputLimit::Terminate(64 * 1024 * 1024),
                stderr: OutputLimit::Terminate(64 * 1024 * 1024),
                timeout: None,
            },
            file_bytes: 16 * 1024 * 1024,
            source_entries: 8192,
            source_depth: 64,
        },
        || {
            let mut command =
                Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
            command.env("CARGO_TARGET_DIR", &target);
            command
        },
        |archive, destination| {
            let mut command = Command::new("tar");
            command.arg("-xzf").arg(archive).arg("-C").arg(destination);
            command
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_host_process::tool::{ExecutionError, ExecutionEvidence, ExecutionFailure, ToolError};

    #[test]
    fn failure_report_preserves_stage_paths_and_bounds_escaped_capture() {
        let mut stderr = vec![b'x'; 20_000];
        stderr.extend_from_slice(b"\nerror: broken fixture\x1b[31m\n");
        let command = PackageCommandError {
            stage: "test-targets".to_owned(),
            stdout_path: "/evidence/test-targets.json".into(),
            stderr_path: "/evidence/test-targets.stderr".into(),
            execution: Some(ToolError::Execution(Box::new(ExecutionError {
                failure: ExecutionFailure::ExitStatus,
                evidence: ExecutionEvidence {
                    stderr,
                    ..ExecutionEvidence::default()
                },
                cleanup: None,
            }))),
            retention: Vec::new(),
        };
        let error = ic_build::cargo::packages::PackageQualificationError {
            evidence: "/evidence".into(),
            cause: Box::new(command),
        };
        let report = failure_report(&error);
        assert!(report.contains("package stage test-targets"));
        assert!(report.contains("/evidence/test-targets.stderr"));
        assert!(report.contains("error: broken fixture"));
        assert!(!report.contains('\x1b'));
        assert!(report.len() < 9000);
    }
}
