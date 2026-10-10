//! Resolve a caller-selected local executable without executing or admitting it.

#[cfg(unix)]
#[expect(
    clippy::unnecessary_debug_formatting,
    reason = "literal path bytes must stay escaped rather than become terminal control characters"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ic_host_process::tool::resolve_executable;
    use std::{io, path::PathBuf};

    let mut args = std::env::args_os().skip(1);
    let mut required = || {
        args.next().map(PathBuf::from).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: resolve_tool REQUEST ABSOLUTE_WORKDIR [SEARCH_DIRECTORY ...]",
            )
        })
    };
    let requested = required()?;
    let current_dir = required()?;
    let directories = args.map(PathBuf::from).collect::<Vec<_>>();
    let selected = resolve_executable(&requested, &current_dir, &directories)?;
    // Debug quoting preserves non-UTF-8 paths and prevents control characters
    // from becoming additional terminal lines. This example intentionally prints
    // the selected path, not a trust or executable identity claim.
    println!("candidate={:?}", selected.as_os_str());
    Ok(())
}

#[cfg(not(unix))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "tool resolution requires a Unix host",
    )
    .into())
}
