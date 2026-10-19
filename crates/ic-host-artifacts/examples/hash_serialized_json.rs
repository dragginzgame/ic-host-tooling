//! Demonstrate caller-owned JSON encoding over a bounded hashing sink.

use clap::Parser as _;
use ic_host_artifacts::artifact::{HashingWriter, read_reader};
use std::{error::Error, io};

#[derive(clap::Parser)]
struct Arguments {
    input_bytes: usize,
    output_bytes: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let input = read_reader(io::stdin().lock(), args.input_bytes)?;
    let value: serde_json::Value = serde_json::from_slice(&input)?;
    let mut writer = HashingWriter::new(io::sink(), args.output_bytes);
    serde_json::to_writer(&mut writer, &value)?;
    let (_, identity) = writer.into_parts();
    println!(
        "serialized_bytes={} sha256={}",
        identity.bytes, identity.sha256
    );
    Ok(())
}
