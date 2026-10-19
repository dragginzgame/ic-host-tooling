//! Inspect local ICP CLI output without running a command or printing its payload.

use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::read::read_file;
use ic_host_tools::response::{ResponseFormat, ResponseLimits, decode};

use clap::Parser as _;
use std::{error::Error, path::PathBuf};

#[derive(clap::Parser)]
struct Arguments {
    path: PathBuf,
    #[arg(value_parser = ["json", "hex"])]
    format: String,
    max_input_bytes: usize,
    max_decoded_bytes: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let format = match args.format.as_str() {
        "json" => ResponseFormat::Json,
        "hex" => ResponseFormat::Hex,
        _ => unreachable!("Clap admits only the declared formats"),
    };
    let limits = ResponseLimits {
        input_bytes: args.max_input_bytes,
        decoded_bytes: args.max_decoded_bytes,
    };
    let input = read_file(&args.path, limits.input_bytes)?;
    let bytes = decode(&input, format, limits)?;
    println!(
        "response_bytes={} sha256={}",
        bytes.len(),
        Sha256Digest::compute(&bytes)
    );
    Ok(())
}
