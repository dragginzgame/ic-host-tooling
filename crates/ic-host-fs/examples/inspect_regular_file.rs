//! Inspect a local regular file, rejecting its final symlink without mutation.

#[cfg(unix)]
#[derive(clap::Parser)]
struct Arguments {
    path: std::path::PathBuf,
    max_bytes: u64,
}

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser as _;
    use ic_host_fs::read::hash_file_no_follow;
    let args = Arguments::parse();
    let identity = hash_file_no_follow(&args.path, args.max_bytes)?;
    println!("raw_bytes={} sha256={}", identity.bytes, identity.sha256);
    Ok(())
}

#[cfg(not(unix))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "final-symlink rejection requires a Unix host",
    )
    .into())
}
