//! Stream bytes to a caller-owned staging sink with bounded identity capture.

#[cfg(test)]
mod tests;

use super::{ArtifactError, ArtifactIdentity, BoundedWriter, Sha256Digest, visit_reader};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    io::{self, Read, Write},
};

/// A bounded copy failed before a complete source identity could be returned.
#[derive(Debug)]
pub enum CopyError {
    /// Source IO or complete-input byte allowance failed.
    Input(ArtifactError),
    /// The caller's sink failed, possibly after accepting a partial chunk.
    Output(io::Error),
}

impl fmt::Display for CopyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(source) => write!(f, "copy input failed: {source}"),
            Self::Output(source) => write!(f, "copy output failed: {source}"),
        }
    }
}

impl std::error::Error for CopyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(source) => Some(source),
            Self::Output(source) => Some(source),
        }
    }
}

impl From<ArtifactError> for CopyError {
    fn from(source: ArtifactError) -> Self {
        Self::Input(source)
    }
}

/// Copy one bounded source stream and identify its bytes with constant memory.
///
/// Shares the read traversal used by [`super::hash_reader`] and
/// [`super::read_reader`]: observes at most `max_bytes + 1` bytes, detects
/// overflow before writing the overflowing chunk, and retries interrupted reads
/// only. Writes each accepted chunk through `Write::write_all`, which handles
/// short writes and interrupted writes according to the standard IO contract.
/// The shared [`BoundedWriter`] validates accepted-byte counts before advancing
/// through output. Impossible reader/writer counts return IO
/// [`io::ErrorKind::InvalidData`] through the corresponding error variant.
/// A blocking reader or writer's deadlines remain caller-owned.
///
/// Bytes reach the sink before the complete source is known. Supply a private
/// staging sink, check the returned identity against your admitted digest, and
/// validate it before publication. Copying does not freeze source contents or
/// independently re-read the sink. On every failure, partial output remains
/// available to the caller; no identity for the complete input is returned.
///
/// This function never opens paths, flushes, synchronizes, renames or deletes
/// files. It chooses no filesystem authority or publication/recovery policy.
///
/// # Errors
/// Returns distinct typed source/limit and sink failures. Output may contain a
/// prefix even when the source is oversized or a write fails.
pub fn copy_reader(
    mut reader: impl Read,
    writer: &mut impl Write,
    max_bytes: u64,
) -> Result<ArtifactIdentity, CopyError> {
    let mut hasher = Sha256::new();
    let mut writer = BoundedWriter::new(writer, max_bytes);
    let bytes = visit_reader::<CopyError>(&mut reader, max_bytes, |chunk| {
        writer.write_all(chunk).map_err(CopyError::Output)?;
        hasher.update(chunk);
        Ok(())
    })?;
    Ok(ArtifactIdentity {
        bytes,
        sha256: Sha256Digest::from_bytes(hasher.finalize().into()),
    })
}
