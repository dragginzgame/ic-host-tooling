//! Bounded ordered chunk identities from one source traversal.

#[cfg(test)]
mod tests;

use super::{ArtifactError, ArtifactIdentity, Sha256Digest, reserve_bounded, visit_reader};
use sha2::{Digest, Sha256};
use std::{collections::TryReserveError, fmt, io::Read, num::NonZeroUsize};

/// Chunk hashing failed without returning partial identities.
#[derive(Debug)]
pub enum ChunkDigestError {
    /// Source read or complete-input allowance failed.
    Input(ArtifactError),
    /// More chunks were needed than the caller permits retaining.
    ChunkLimit {
        /// Maximum number of retained chunk digests.
        limit: usize,
    },
    /// Storage for a chunk digest could not be allocated.
    Allocation(TryReserveError),
}

impl fmt::Display for ChunkDigestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(source) => write!(f, "chunk input failed: {source}"),
            Self::ChunkLimit { limit } => write!(f, "input exceeds {limit} chunks"),
            Self::Allocation(source) => write!(f, "chunk digest allocation failed: {source}"),
        }
    }
}

impl std::error::Error for ChunkDigestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(source) => Some(source),
            Self::Allocation(source) => Some(source),
            Self::ChunkLimit { .. } => None,
        }
    }
}

impl From<ArtifactError> for ChunkDigestError {
    fn from(source: ArtifactError) -> Self {
        Self::Input(source)
    }
}

/// Hash ordered fixed-size chunks and the complete input in one bounded pass.
///
/// Returns `(chunk_digests, complete_identity)`. Chunk boundaries depend only on
/// `chunk_bytes`, never on reader fragmentation. Only the final chunk may be
/// shorter; exact multiples add no empty chunk. Empty input has no chunks and
/// the ordinary SHA-256 of an empty stream. Zero chunk size is excluded by type.
///
/// Reuses the bounded reader engine: reads at most `max_bytes + 1` bytes,
/// retries interrupted reads, and rejects impossible reader byte counts.
/// Working storage is constant apart from the fallibly allocated digest vector,
/// bounded by `max_chunks`. No buffer proportional to chunk size is allocated.
/// A blocking reader's deadline remains caller-owned.
///
/// Hashes exactly the supplied bytes, with no gzip detection/decompression or
/// Wasm interpretation. A compressed artifact's upload chunks and whole identity
/// differ from its decoded module identity. Chunk sizes, schemas, trusted digest
/// comparison, source custody and upload/retry policy remain caller-owned.
///
/// # Errors
/// Returns typed input, byte/chunk-bound or allocation failures. No partial
/// digest vector or complete identity is returned on failure.
pub fn chunk_digests(
    mut reader: impl Read,
    chunk_bytes: NonZeroUsize,
    max_bytes: u64,
    max_chunks: usize,
) -> Result<(Vec<Sha256Digest>, ArtifactIdentity), ChunkDigestError> {
    let mut digests = Vec::new();
    let mut whole = Sha256::new();
    let mut chunk = Sha256::new();
    let mut filled = 0;
    let bytes = visit_reader::<ChunkDigestError>(&mut reader, max_bytes, |mut buffer| {
        whole.update(buffer);
        while !buffer.is_empty() {
            if filled == 0 {
                if digests.len() == max_chunks {
                    return Err(ChunkDigestError::ChunkLimit { limit: max_chunks });
                }
                reserve_bounded(&mut digests, 1, max_chunks)
                    .map_err(ChunkDigestError::Allocation)?;
            }
            let count = buffer.len().min(chunk_bytes.get() - filled);
            chunk.update(&buffer[..count]);
            filled += count;
            buffer = &buffer[count..];
            if filled == chunk_bytes.get() {
                digests.push(Sha256Digest::from_bytes(chunk.finalize_reset().into()));
                filled = 0;
            }
        }
        Ok(())
    })?;
    if filled != 0 {
        // Capacity was reserved before accepting the partial chunk's first byte.
        digests.push(Sha256Digest::from_bytes(chunk.finalize().into()));
    }
    Ok((
        digests,
        ArtifactIdentity {
            bytes,
            sha256: Sha256Digest::from_bytes(whole.finalize().into()),
        },
    ))
}
