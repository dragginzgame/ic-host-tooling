//! Inspect local ICP CLI output without running a command or printing its payload.

use ic_host_artifacts::artifact::hash_reader;
use ic_host_fs::read::read_file;
use ic_host_tools::response::{ResponseFormat, ResponseLimits, decode};

use std::{error::Error, io, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or_else(|| {
        invalid_argument(
            "usage: inspect_response PATH json|hex|labeled MAX_INPUT_BYTES MAX_DECODED_BYTES",
        )
    })?;
    let format = match args.next().as_deref().and_then(std::ffi::OsStr::to_str) {
        Some("json") => ResponseFormat::Json,
        Some("hex") => ResponseFormat::Hex,
        Some("labeled") => ResponseFormat::LabeledHex,
        _ => return Err(invalid_argument("format must be json, hex or labeled").into()),
    };
    let mut limit = || -> Result<usize, Box<dyn Error>> {
        Ok(args
            .next()
            .ok_or_else(|| invalid_argument("missing response limit"))?
            .to_str()
            .ok_or_else(|| invalid_argument("limits must be UTF-8"))?
            .parse()?)
    };
    let limits = ResponseLimits {
        input_bytes: limit()?,
        decoded_bytes: limit()?,
    };
    if args.next().is_some() {
        return Err(invalid_argument("unexpected argument").into());
    }
    let input = read_file(Path::new(&path), limits.input_bytes)?;
    let bytes = decode(&input, format, limits)?;
    let identity = hash_reader(bytes.as_slice(), u64::try_from(limits.decoded_bytes)?)?;
    println!(
        "response_bytes={} sha256={}",
        identity.bytes, identity.sha256
    );
    Ok(())
}

fn invalid_argument(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
