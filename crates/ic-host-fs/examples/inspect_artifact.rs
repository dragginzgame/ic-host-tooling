//! Local artifact evidence with explicit consumer limits and optional digest authority.

use ic_host_artifacts::artifact::verify_reader;
use ic_host_artifacts::wasm::{InspectionLimits, inspect};
use ic_host_fs::read::read_file;

use std::{error::Error, io, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or_else(|| invalid_argument("usage: inspect_artifact PATH MAX_BYTES MAX_SECTIONS MAX_EXPORTS MAX_CUSTOM_SECTIONS [SHA256]"))?;
    let mut number = || -> Result<usize, Box<dyn Error>> {
        Ok(args
            .next()
            .ok_or_else(|| invalid_argument("missing inspection limit"))?
            .to_str()
            .ok_or_else(|| invalid_argument("inspection limits must be UTF-8"))?
            .parse()?)
    };
    let limits = InspectionLimits {
        module_bytes: number()?,
        sections: number()?,
        exports: u32::try_from(number()?)?,
        custom_sections: number()?,
    };
    let expected = args
        .next()
        .map(|value| {
            value
                .into_string()
                .map_err(|_| invalid_argument("digest must be UTF-8"))
        })
        .transpose()?;
    if args.next().is_some() {
        return Err(invalid_argument("unexpected argument").into());
    }
    let bytes = read_file(Path::new(&path), limits.module_bytes)?;
    let identity = if let Some(expected) = expected {
        verify_reader(
            bytes.as_slice(),
            limits.module_bytes as u64,
            expected.parse()?,
        )?
    } else {
        ic_host_artifacts::artifact::hash_reader(bytes.as_slice(), limits.module_bytes as u64)?
    };
    let facts = inspect(&bytes, limits)?;
    println!(
        "raw_bytes={} sha256={} code_section_bytes={} data_section_bytes={} defined_functions={} data_segments={}",
        identity.bytes,
        identity.sha256,
        facts.code_section_bytes,
        facts.data_section_bytes,
        facts.defined_functions,
        facts.data_segments
    );
    for (name, export) in facts.exports {
        println!(
            "export={name:?} kind={:?} index={}",
            export.kind, export.index
        );
    }
    for section in facts.custom_sections {
        println!(
            "custom_section={:?} bytes={}",
            section.name,
            section.data.len()
        );
    }
    Ok(())
}

fn invalid_argument(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
