//! Bounded Git observations with explicit tool admission and status scope.

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ic_host_process::provenance::{
        IgnoreSubmodules, StatusOptions, UntrackedFiles, capture_git,
    };
    use ic_host_process::tool::{AdmittedTool, ExecutionContext, OutputLimits, ToolSpec};

    use std::{ffi::OsString, io, path::PathBuf, time::Duration};

    let invalid = |message| io::Error::new(io::ErrorKind::InvalidInput, message);
    let mut args = std::env::args_os().skip(1);
    let mut required = || {
        args.next().ok_or_else(|| invalid(
        "usage: inspect_git TOOL SHA256 VERSION WORKDIR MAX_TOOL_BYTES MAX_STDOUT_BYTES MAX_STDERR_BYTES TIMEOUT_MS UNTRACKED SUBMODULES",
    ))
    };
    let executable = PathBuf::from(required()?);
    let digest = required()?
        .into_string()
        .map_err(|_| invalid("digest must be UTF-8"))?
        .parse()?;
    let version = required()?
        .into_string()
        .map_err(|_| invalid("version must be UTF-8"))?;
    let directory = PathBuf::from(required()?);
    let mut number = || -> Result<u64, Box<dyn std::error::Error>> {
        Ok(required()?
            .to_str()
            .ok_or_else(|| invalid("limits must be UTF-8"))?
            .parse()?)
    };
    let executable_bytes = number()?;
    let limits = OutputLimits {
        stdout_bytes: usize::try_from(number()?)?,
        stderr_bytes: usize::try_from(number()?)?,
        timeout: Duration::from_millis(number()?),
    };
    let untracked = match required()?.to_str() {
        Some("no") => UntrackedFiles::No,
        Some("normal") => UntrackedFiles::Normal,
        Some("all") => UntrackedFiles::All,
        _ => return Err(invalid("untracked must be no, normal or all").into()),
    };
    let ignore_submodules = match required()?.to_str() {
        Some("none") => IgnoreSubmodules::None,
        Some("untracked") => IgnoreSubmodules::Untracked,
        Some("dirty") => IgnoreSubmodules::Dirty,
        Some("all") => IgnoreSubmodules::All,
        _ => return Err(invalid("submodules must be none, untracked, dirty or all").into()),
    };
    if args.next().is_some() {
        return Err(invalid("unexpected argument").into());
    }
    // The example explicitly selects an empty environment; the library never
    // chooses environment exclusions or tool/version pins for a consumer.
    let context = ExecutionContext {
        current_dir: &directory,
        environment: &[],
    };
    let git = AdmittedTool::admit(
        &ToolSpec {
            executable: &executable,
            sha256: digest,
            executable_bytes,
            version_arguments: &[OsString::from("--version")],
            version_identity: &version,
        },
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
    println!("revision={}", observed.revision);
    println!("tree={}", observed.tree);
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
