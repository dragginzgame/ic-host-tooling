//! Digest-verified copying through the existing durable publication engine.

use super::{NamedWriteError, WriteOptions, write_with};
use ic_host_artifacts::artifact::{ArtifactIdentity, CopyError, Sha256Digest, copy_reader};
use std::path::Path;

/// Copy one regular artifact and verify its digest before durable publication.
///
/// The source opener follows symlinks; on Unix, it refuses FIFOs without blocking.
/// Callers own trusted paths, source custody, expected digest, byte allowance,
/// destination reuse and publication options. The copy uses constant memory and
/// observes at most `max_bytes + 1` source bytes. A mismatch or copy failure never
/// publishes the staging file. This does not freeze source contents or rehash the
/// published destination; callers may independently verify retained evidence.
///
/// # Errors
/// Retains typed input, digest, copy and publication errors, including cleanup
/// failures. After-publication sync failure means complete bytes are visible.
pub fn copy_verified(
    source: &Path,
    destination: &Path,
    expected: Sha256Digest,
    max_bytes: u64,
    options: WriteOptions,
) -> Result<ArtifactIdentity, NamedWriteError<CopyError>> {
    write_with(destination, options, |sink| {
        let source = crate::read::open_regular_file(source)?;
        copy_reader(source, sink, max_bytes)?
            .verify(expected)
            .map_err(CopyError::Input)
    })
}

#[cfg(test)]
mod tests;
