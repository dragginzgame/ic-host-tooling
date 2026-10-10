//! Inspect a saved gzip artifact without writing decoded bytes or printing them.

use ic_host_artifacts::artifact::{Sha256Digest, hash_gzip};
use ic_host_fs::read::read_file;
use std::{error::Error, io, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: inspect_gzip PATH MAX_COMPRESSED_BYTES MAX_DECODED_BYTES",
        )
    })?;
    let mut limit = || -> Result<usize, Box<dyn Error>> {
        Ok(args
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing byte limit"))?
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "limits must be UTF-8"))?
            .parse()?)
    };
    let max_compressed = limit()?;
    let max_decoded = limit()?;
    if args.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unexpected argument").into());
    }
    let compressed = read_file(Path::new(&path), max_compressed)?;
    let decoded = hash_gzip(&compressed, max_compressed, max_decoded as u64)?;
    println!(
        "compressed_bytes={} sha256={}",
        compressed.len(),
        Sha256Digest::compute(&compressed)
    );
    println!("decoded_bytes={} sha256={}", decoded.bytes, decoded.sha256);
    Ok(())
}
