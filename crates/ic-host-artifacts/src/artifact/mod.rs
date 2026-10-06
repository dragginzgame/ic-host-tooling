//! Bounded streams and raw SHA-256 identities for artifact bytes.
//!
//! Consumers own trusted paths, admitted digests, and byte limits. An identity
//! describes the bytes read; it does not freeze a path for later execution or
//! certify an executable's version. Copying writes only to a caller-owned sink;
//! paths, confinement, synchronization, publication and cleanup remain local.

use sha2::{Digest, Sha256};
use std::{
    fmt,
    io::{self, Read},
    str::FromStr,
};

mod copy;
#[cfg(feature = "gzip")]
mod gzip;
#[cfg(test)]
mod tests;
mod writer;

pub use copy::{CopyError, copy_reader};
#[cfg(feature = "gzip")]
pub use gzip::{GzipError, decode_gzip};
pub use writer::{BoundedWriter, WriterError};

/// Raw SHA-256 identity, without a product-specific prefix or wire format.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    /// Construct an identity from exact digest bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrow the digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Hash bytes already held by the caller.
    #[must_use]
    pub fn compute(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Invalid lowercase hexadecimal digest authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestParseError {
    /// The digest does not contain exactly 64 ASCII bytes.
    Length {
        /// Observed byte length.
        actual: usize,
    },
    /// A byte is not a lowercase hexadecimal digit.
    Digit {
        /// Offset of the invalid byte.
        offset: usize,
    },
}

impl fmt::Display for DigestParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { actual } => {
                write!(f, "SHA-256 requires 64 hex bytes, received {actual}")
            }
            Self::Digit { offset } => write!(f, "invalid lowercase SHA-256 digit at byte {offset}"),
        }
    }
}
impl std::error::Error for DigestParseError {}

impl FromStr for Sha256Digest {
    type Err = DigestParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.len() != 64 {
            return Err(DigestParseError::Length { actual: text.len() });
        }
        let mut bytes = [0; 32];
        for (offset, digit) in text.bytes().enumerate() {
            let nibble = match digit {
                b'0'..=b'9' => digit - b'0',
                b'a'..=b'f' => digit - b'a' + 10,
                _ => return Err(DigestParseError::Digit { offset }),
            };
            bytes[offset / 2] |= nibble << if offset % 2 == 0 { 4 } else { 0 };
        }
        Ok(Self(bytes))
    }
}

/// Exact size and digest of one bounded byte stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactIdentity {
    /// Number of bytes successfully read.
    pub bytes: u64,
    /// SHA-256 of those bytes.
    pub sha256: Sha256Digest,
}

/// A bounded read or digest verification failed.
#[derive(Debug)]
pub enum ArtifactError {
    /// The reader or filesystem failed. No partial identity is returned.
    Io(io::Error),
    /// The selected path is not a regular file.
    NotRegularFile,
    /// A stream exceeded the caller's byte allowance.
    LimitExceeded {
        /// Maximum permitted bytes.
        limit: u64,
    },
    /// The complete bounded stream does not match the admitted digest.
    DigestMismatch {
        /// Caller-selected authority.
        expected: Sha256Digest,
        /// Observed size and digest.
        actual: ArtifactIdentity,
    },
    /// Storage could not be allocated for a bounded file read.
    Allocation(std::collections::TryReserveError),
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => write!(f, "artifact read failed: {source}"),
            Self::NotRegularFile => f.write_str("artifact path is not a regular file"),
            Self::LimitExceeded { limit } => write!(f, "artifact exceeds {limit} bytes"),
            Self::DigestMismatch { expected, actual } => write!(
                f,
                "artifact SHA-256 is {}, expected {expected}",
                actual.sha256
            ),
            Self::Allocation(source) => write!(f, "artifact allocation failed: {source}"),
        }
    }
}
impl std::error::Error for ArtifactError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}
impl From<io::Error> for ArtifactError {
    fn from(source: io::Error) -> Self {
        Self::Io(source)
    }
}

/// Hash one stream with constant working memory and no reliance on metadata.
///
/// Reads at most `max_bytes + 1` bytes, using the extra byte to detect overflow.
/// Retries interrupted reads only; no subprocess or network retry is performed.
/// A blocking reader's timeouts remain the caller's responsibility.
///
/// # Errors
/// Returns [`ArtifactError::LimitExceeded`] or the underlying read error.
/// An impossible reader byte count returns IO [`io::ErrorKind::InvalidData`].
pub fn hash_reader(
    mut reader: impl Read,
    max_bytes: u64,
) -> Result<ArtifactIdentity, ArtifactError> {
    let mut hasher = Sha256::new();
    let bytes = visit_reader::<ArtifactError>(&mut reader, max_bytes, |chunk| {
        hasher.update(chunk);
        Ok(())
    })?;
    Ok(ArtifactIdentity {
        bytes,
        sha256: Sha256Digest(hasher.finalize().into()),
    })
}

/// Verify a complete bounded stream before the caller extracts or uses its bytes.
///
/// # Errors
/// Returns read/limit failures or a typed mismatch with the observed identity.
pub fn verify_reader(
    reader: impl Read,
    max_bytes: u64,
    expected: Sha256Digest,
) -> Result<ArtifactIdentity, ArtifactError> {
    let actual = hash_reader(reader, max_bytes)?;
    if actual.sha256 != expected {
        return Err(ArtifactError::DigestMismatch { expected, actual });
    }
    Ok(actual)
}

/// Read one stream into bounded, fallibly allocated storage.
///
/// Observes at most `max_bytes + 1` bytes to detect overflow. A blocking reader's
/// deadline remains caller-owned. Only interrupted reads are retried.
///
/// # Errors
/// Returns read, allocation, or byte-limit failures. An impossible reader byte
/// count returns IO [`io::ErrorKind::InvalidData`].
pub fn read_reader(mut reader: impl Read, max_bytes: usize) -> Result<Vec<u8>, ArtifactError> {
    let mut bytes = Vec::new();
    visit_reader::<ArtifactError>(&mut reader, max_bytes as u64, |chunk| {
        bytes
            .try_reserve_exact(chunk.len())
            .map_err(ArtifactError::Allocation)?;
        bytes.extend_from_slice(chunk);
        Ok(())
    })?;
    Ok(bytes)
}

fn visit_reader<E: From<ArtifactError>>(
    reader: &mut impl Read,
    limit: u64,
    mut visit: impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<u64, E> {
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let remaining = usize::try_from(limit - bytes).unwrap_or(usize::MAX);
        let allowance = remaining.saturating_add(1).min(buffer.len());
        let count = match reader.read(&mut buffer[..allowance]) {
            Ok(count) => count,
            Err(source) if source.kind() == io::ErrorKind::Interrupted => continue,
            Err(source) => return Err(ArtifactError::Io(source).into()),
        };
        if count > allowance {
            return Err(ArtifactError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "reader returned more bytes than its buffer can hold",
            ))
            .into());
        }
        if count == 0 {
            return Ok(bytes);
        }
        if count as u64 > limit - bytes {
            return Err(ArtifactError::LimitExceeded { limit }.into());
        }
        visit(&buffer[..count])?;
        bytes += count as u64;
    }
}
