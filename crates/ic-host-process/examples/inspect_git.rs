//! Bounded Git observations with explicit tool admission and status scope.

#[cfg(unix)]
#[derive(clap::Parser)]
struct Arguments {
    tool: std::path::PathBuf,
    sha256: ic_host_artifacts::artifact::Sha256Digest,
    version: String,
    workdir: std::path::PathBuf,
    max_tool_bytes: u64,
    max_stdout_bytes: usize,
    max_stderr_bytes: usize,
    timeout_ms: u64,
    #[arg(value_parser = ["no", "normal", "all"])]
    untracked: String,
    #[arg(value_parser = ["none", "untracked", "dirty", "all"])]
    submodules: String,
}

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ic_host_process::provenance::{
        IgnoreSubmodules, StatusOptions, UntrackedFiles, capture_git,
    };
    use ic_host_process::tool::{AdmittedTool, ExecutionContext, OutputLimits, VersionSpec};

    use clap::Parser as _;
    use std::{ffi::OsString, time::Duration};

    let args = Arguments::parse();
    let limits = OutputLimits {
        stdout: ic_host_process::tool::OutputLimit::Terminate(args.max_stdout_bytes),
        stderr: ic_host_process::tool::OutputLimit::Terminate(args.max_stderr_bytes),
        timeout: Some(Duration::from_millis(args.timeout_ms)),
    };
    let untracked = match args.untracked.as_str() {
        "no" => UntrackedFiles::No,
        "normal" => UntrackedFiles::Normal,
        "all" => UntrackedFiles::All,
        _ => unreachable!("Clap admits only declared untracked policies"),
    };
    let ignore_submodules = match args.submodules.as_str() {
        "none" => IgnoreSubmodules::None,
        "untracked" => IgnoreSubmodules::Untracked,
        "dirty" => IgnoreSubmodules::Dirty,
        "all" => IgnoreSubmodules::All,
        _ => unreachable!("Clap admits only declared submodule policies"),
    };
    // The example explicitly selects an empty environment; the library never
    // chooses environment exclusions or tool/version pins for a consumer.
    let context = ExecutionContext {
        current_dir: &args.workdir,
        environment: &[],
    };
    let git = AdmittedTool::admit(
        &VersionSpec {
            executable: &args.tool,
            executable_bytes: args.max_tool_bytes,
            version_arguments: &[OsString::from("--version")],
        },
        args.sha256,
        &args.version,
        &context,
        limits,
    )?;
    let observed = capture_git(
        &git,
        &context,
        StatusOptions {
            untracked,
            ignore_submodules,
        },
        limits,
    )?;
    println!("revision={}", observed.revision());
    println!("tree={}", observed.tree());
    println!("dirty={}", observed.is_dirty());
    println!("status_bytes={}", observed.status_identity.bytes);
    println!("status_sha256={}", observed.status_identity.sha256);
    Ok(())
}

#[cfg(not(unix))]
fn main() -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Git observations require a supported Unix host",
    ))
}
