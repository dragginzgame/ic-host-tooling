//! Inspect a saved gzip artifact without writing decoded bytes or printing them.

use clap::Parser as _;
use ic_host_artifacts::artifact::{Sha256Digest, hash_gzip};
use ic_host_fs::read::read_file;
use std::{error::Error, path::PathBuf};

#[derive(clap::Parser)]
struct Arguments {
    path: PathBuf,
    max_compressed_bytes: usize,
    max_decoded_bytes: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let compressed = read_file(&args.path, args.max_compressed_bytes)?;
    let decoded = hash_gzip(
        &compressed,
        args.max_compressed_bytes,
        args.max_decoded_bytes as u64,
    )?;
    println!(
        "compressed_bytes={} sha256={}",
        compressed.len(),
        Sha256Digest::compute(&compressed)
    );
    println!("decoded_bytes={} sha256={}", decoded.bytes, decoded.sha256);
    Ok(())
}
