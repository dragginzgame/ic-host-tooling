//! Inspect a local regular file, rejecting its final symlink without mutation.

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ic_host_artifacts::artifact::Sha256Digest;
    use ic_host_fs::read::read_file_no_follow;
    use std::{io, path::Path};

    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: inspect_regular_file PATH MAX_BYTES",
        )
    })?;
    let max_bytes = args
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing byte limit"))?
        .into_string()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "byte limit must be UTF-8"))?
        .parse()?;
    if args.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unexpected argument").into());
    }
    let bytes = read_file_no_follow(Path::new(&path), max_bytes)?;
    println!(
        "raw_bytes={} sha256={}",
        bytes.len(),
        Sha256Digest::compute(&bytes)
    );
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
