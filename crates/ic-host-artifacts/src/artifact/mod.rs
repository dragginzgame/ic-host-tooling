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

mod chunks;
mod copy;
#[cfg(feature = "gzip")]
mod gzip;
mod hashing;
mod matching;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod writer;

pub use chunks::{ChunkDigestError, chunk_digests};
pub use copy::{CopyError, copy_reader};
#[cfg(feature = "gzip")]
pub use gzip::{
    GzipError, copy_gzip, decode_gzip, encode_gzip, gzip_matches, hash_gzip, hash_gzip_or_raw,
};
pub use hashing::HashingWriter;
pub use matching::MatchingWriter;
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

impl ArtifactIdentity {
    /// Compare this observed digest with caller-selected authority.
    ///
    /// Does not read, hash, freeze or validate the origin of the identity.
    /// Size admission and selection of the expected digest remain caller-owned.
    ///
    /// # Errors
    /// Returns a typed mismatch retaining the expected digest and this identity.
    pub fn verify(self, expected: Sha256Digest) -> Result<Self, ArtifactError> {
        if self.sha256 != expected {
            return Err(ArtifactError::DigestMismatch {
                expected,
                actual: self,
            });
        }
        Ok(self)
    }
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
    /// A bounded stream ended before the caller's exact byte count.
    LengthMismatch {
        /// Required bytes.
        expected: u64,
        /// Successfully read bytes.
        actual: u64,
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
            Self::LengthMismatch { expected, actual } => {
                write!(f, "artifact has {actual} bytes, expected {expected}")
            }
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

/// Preserve native I/O identity or retain the typed artifact cause at an I/O boundary.
impl From<ArtifactError> for io::Error {
    fn from(error: ArtifactError) -> Self {
        match error {
            ArtifactError::Io(source) => source,
            other => Self::other(other),
        }
    }
}

/// Hash one stream with constant working memory and no reliance on metadata.
///
/// Reads at most `max_bytes + 1` bytes, using the extra byte to detect overflow.
/// Retries interrupted reads only; no subprocess or network retry is performed.
/// A blocking reader's timeouts remain the caller's responsibility.
/// Admit the returned identity against a selected digest with [`ArtifactIdentity::verify`].
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

/// Hash a stream whose complete length must equal `expected_bytes`.
///
/// Reads at most one byte beyond the selected length to distinguish overflow.
/// The caller owns the reader's origin, deadline and byte-budget selection.
///
/// # Errors
/// Returns a typed short-length or limit error, or the underlying read error.
pub fn hash_exact_reader(
    reader: impl Read,
    expected_bytes: u64,
) -> Result<ArtifactIdentity, ArtifactError> {
    let identity = hash_reader(reader, expected_bytes)?;
    if identity.bytes != expected_bytes {
        return Err(ArtifactError::LengthMismatch {
            expected: expected_bytes,
            actual: identity.bytes,
        });
    }
    Ok(identity)
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
        reserve_bounded(&mut bytes, chunk.len(), max_bytes).map_err(ArtifactError::Allocation)?;
        bytes.extend_from_slice(chunk);
        Ok(())
    })?;
    Ok(bytes)
}

/// Read a complete stream into bounded storage at an exact selected length.
///
/// The caller owns the reader's origin, deadline and byte-budget selection.
/// Reads at most one byte beyond the selected length to distinguish overflow.
///
/// # Errors
/// Returns a typed short-length or limit error, allocation failure, or read error.
pub fn read_exact_reader(
    reader: impl Read,
    expected_bytes: usize,
) -> Result<Vec<u8>, ArtifactError> {
    let bytes = read_reader(reader, expected_bytes)?;
    if bytes.len() != expected_bytes {
        return Err(ArtifactError::LengthMismatch {
            expected: expected_bytes as u64,
            actual: bytes.len() as u64,
        });
    }
    Ok(bytes)
}

// Callers admit the additional elements against `limit` before reserving.
// Grow geometrically without requesting capacity beyond that element budget.
fn reserve_bounded<T>(
    items: &mut Vec<T>,
    additional: usize,
    limit: usize,
) -> Result<(), std::collections::TryReserveError> {
    let required = items.len() + additional;
    if required > items.capacity() {
        let capacity = items.capacity().saturating_mul(2).max(required).min(limit);
        items.try_reserve_exact(capacity - items.len())?;
    }
    Ok(())
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
