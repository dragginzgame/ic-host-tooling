//! Qualify the exact coordinated archives through IC Build's owning engine.

use std::{error::Error, path::Path, process::Command};

use ic_build::cargo::packages::{
    PackageArchiveSpec, PackageQualificationLimits, qualify_package_archives,
};
use ic_host_process::tool::{OutputLimit, OutputLimits};

fn main() -> Result<(), Box<dyn Error>> {
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
    qualify_package_archives(
        workspace,
        &target.join("package"),
        &evidence,
        &[PackageArchiveSpec {
            name: "ic-testkit",
            version,
            patch_registry: true,
        }],
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
