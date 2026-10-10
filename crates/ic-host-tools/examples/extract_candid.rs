//! Explicitly admitted extractor with bounded input/output and no file publication.

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ic_host_process::tool::{AdmittedTool, ExecutionContext, OutputLimits, ToolSpec};
    use ic_host_tools::candid::extract;

    use std::{
        ffi::OsString,
        io::{self, Write as _},
        path::PathBuf,
        time::Duration,
    };

    let mut args = std::env::args_os().skip(1);
    let mut required = || {
        args.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "usage: extract_candid TOOL SHA256 VERSION WASM WORKDIR MAX_TOOL_BYTES MAX_WASM_BYTES MAX_STDOUT_BYTES MAX_STDERR_BYTES TIMEOUT_MS"))
    };
    let executable = PathBuf::from(required()?);
    let digest = required()?
        .into_string()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "digest must be UTF-8"))?
        .parse()?;
    let version = required()?
        .into_string()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "version must be UTF-8"))?;
    let source = PathBuf::from(required()?);
    let directory = PathBuf::from(required()?);
    let mut number = || -> Result<u64, Box<dyn std::error::Error>> {
        Ok(required()?
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "limits must be UTF-8"))?
            .parse()?)
    };
    let executable_bytes = number()?;
    let source_bytes = number()?;
    let limits = OutputLimits {
        stdout: ic_host_process::tool::OutputLimit::Terminate(usize::try_from(number()?)?),
        stderr: ic_host_process::tool::OutputLimit::Terminate(usize::try_from(number()?)?),
        timeout: Some(Duration::from_millis(number()?)),
    };
    if args.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unexpected argument").into());
    }
    // This example selects a credential-free empty environment explicitly.
    let context = ExecutionContext {
        current_dir: &directory,
        environment: &[],
    };
    let tool = AdmittedTool::admit(
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
    let candid = extract(&tool, &source, &context, source_bytes, limits)?;
    io::stdout().lock().write_all(candid.text.as_bytes())?;
    Ok(())
}

#[cfg(not(unix))]
fn main() -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "verified extractor execution requires a supported Unix host",
    ))
}
