//! Verify and inspect one archive member without writing or executing it.

use ic_host_artifacts::archive::{ArchiveLimits, extract_tar_gz};
use ic_host_fs::read::read_file;

use std::{error::Error, io, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or_else(|| invalid_argument("usage: inspect_archive PATH ARCHIVE_SHA256 MEMBER MEMBER_SHA256 MAX_ARCHIVE_BYTES MAX_TAR_BYTES MAX_MEMBERS MAX_MEMBER_BYTES"))?;
    let mut text = || -> Result<String, io::Error> {
        args.next()
            .ok_or_else(|| invalid_argument("missing archive argument"))?
            .into_string()
            .map_err(|_| invalid_argument("archive authority and limits must be UTF-8"))
    };
    let archive_sha256 = text()?.parse()?;
    let member = text()?;
    let member_sha256 = text()?.parse()?;
    let limits = ArchiveLimits {
        archive_bytes: text()?.parse()?,
        decompressed_bytes: text()?.parse()?,
        members: text()?.parse()?,
        member_bytes: text()?.parse()?,
    };
    if args.next().is_some() {
        return Err(invalid_argument("unexpected argument").into());
    }
    let bytes = read_file(Path::new(&path), limits.archive_bytes)?;
    let selected = extract_tar_gz(&bytes, archive_sha256, &member, member_sha256, limits)?;
    println!(
        "archive_bytes={} archive_sha256={} member_bytes={} member_sha256={}",
        selected.archive_identity.bytes,
        selected.archive_identity.sha256,
        selected.member_identity.bytes,
        selected.member_identity.sha256
    );
    Ok(())
}

fn invalid_argument(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
