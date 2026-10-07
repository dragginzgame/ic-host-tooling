//! Hash accepted bytes from a push producer without buffering its complete output.

#[cfg(test)]
mod tests;

use super::{ArtifactIdentity, BoundedWriter, Sha256Digest};
use sha2::{Digest, Sha256};
use std::io::{self, Write};

/// A bounded caller-owned sink with a raw SHA-256 of successful writes.
///
/// Counts and hashes only the bytes reported as accepted by the sink. Short
/// writes, interrupted writes and byte allowances use [`BoundedWriter`]'s
/// existing contract; [`Write::write_all`] handles retries through this wrapper.
/// Sink errors can have unreported partial effects, which cannot be identified.
/// Encoding, flush/sync, publication and recovery remain caller-owned.
///
/// Accept a complete artifact identity only after the producer/serializer
/// succeeds. Recovering parts after failure describes an observed prefix,
/// not a completed or durable artifact. This wrapper never publishes output.
pub struct HashingWriter<W> {
    writer: BoundedWriter<W>,
    hasher: Sha256,
}

impl<W> HashingWriter<W> {
    /// Select the sink and inclusive allowance for accepted bytes.
    #[must_use]
    pub fn new(writer: W, max_bytes: u64) -> Self {
        Self {
            writer: BoundedWriter::new(writer, max_bytes),
            hasher: Sha256::new(),
        }
    }

    /// Number of bytes reported as accepted by successful writes.
    #[must_use]
    pub const fn bytes_written(&self) -> u64 {
        self.writer.bytes_written()
    }

    /// Whether an offered buffer has exceeded the selected allowance.
    #[must_use]
    pub const fn limit_exceeded(&self) -> bool {
        self.writer.limit_exceeded()
    }

    /// Recover the sink and identity of the observed accepted bytes.
    ///
    /// Does not flush, certify producer success or discard partial output.
    /// A failed producer must not use this prefix identity to publish an artifact.
    #[must_use]
    pub fn into_parts(self) -> (W, ArtifactIdentity) {
        let identity = ArtifactIdentity {
            bytes: self.writer.bytes_written(),
            sha256: Sha256Digest::from_bytes(self.hasher.finalize().into()),
        };
        (self.writer.into_inner(), identity)
    }
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let accepted = self.writer.write(buffer)?;
        self.hasher.update(&buffer[..accepted]);
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
