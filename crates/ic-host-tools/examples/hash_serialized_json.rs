//! Demonstrate caller-owned JSON encoding over a bounded hashing sink.

use ic_host_artifacts::artifact::{BoundedWriter, Sha256Digest, read_reader};
use sha2::{Digest, Sha256};
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
    let mut writer = BoundedWriter::new(Sha256::new(), output_limit);
    serde_json::to_writer(&mut writer, &value)?;
    let count = writer.bytes_written();
    let digest = Sha256Digest::from_bytes(writer.into_inner().finalize().into());
    println!("serialized_bytes={count} sha256={digest}");
    Ok(())
}
