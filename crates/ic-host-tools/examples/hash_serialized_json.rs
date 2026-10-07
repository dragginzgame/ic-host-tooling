//! Demonstrate caller-owned JSON encoding over a bounded hashing sink.

use ic_host_artifacts::artifact::{HashingWriter, read_reader};
use std::{env, error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err("usage: hash_serialized_json INPUT_BYTES OUTPUT_BYTES < input.json".into());
    }
    let input_limit: usize = args[1].parse()?;
    let output_limit: u64 = args[2].parse()?;
    let input = read_reader(io::stdin().lock(), input_limit)?;
    let value: serde_json::Value = serde_json::from_slice(&input)?;
    let mut writer = HashingWriter::new(io::sink(), output_limit);
    serde_json::to_writer(&mut writer, &value)?;
    let (_, identity) = writer.into_parts();
    println!(
        "serialized_bytes={} sha256={}",
        identity.bytes, identity.sha256
    );
    Ok(())
}
