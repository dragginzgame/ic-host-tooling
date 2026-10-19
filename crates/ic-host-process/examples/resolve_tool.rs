//! Resolve a caller-selected local executable without executing or admitting it.

#[cfg(unix)]
#[derive(clap::Parser)]
struct Arguments {
    request: std::path::PathBuf,
    absolute_workdir: std::path::PathBuf,
    search_directory: Vec<std::path::PathBuf>,
}

#[cfg(unix)]
#[expect(
    clippy::unnecessary_debug_formatting,
    reason = "literal path bytes must stay escaped rather than become terminal control characters"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser as _;
    use ic_host_process::tool::resolve_executable;
    let args = Arguments::parse();
    let selected = resolve_executable(
        &args.request,
        &args.absolute_workdir,
        &args.search_directory,
    )?;
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
