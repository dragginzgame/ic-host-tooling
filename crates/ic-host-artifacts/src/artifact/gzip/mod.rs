//! Bounded single-member gzip encoding and decoding for host artifacts.

#[cfg(test)]
mod tests;

use super::{ArtifactError, BoundedWriter, read_reader};
use std::{
    fmt,
    io::{self, Write},
};

/// Encode one gzip member with a zero timestamp into a caller-owned bounded sink.
///
/// The caller selects compression, input custody and the inclusive compressed
/// byte budget. The budget includes the header and trailer. No file is created,
/// synchronized or published. On error the sink can contain partial bytes; the
/// caller must discard or reconcile them before publication.
///
/// Repeatable bytes require the same input, compression settings and backend;
/// this does not promise identical output across dependency versions. The
/// compressor's internal working memory is outside the output byte budget.
///
/// # Errors
/// Returns compressor, sink or output-limit errors. Limit errors carry
/// [`super::WriterError`] inside [`io::Error`].
pub fn encode_gzip(
    bytes: &[u8],
    writer: impl Write,
    compression: flate2::Compression,
    max_compressed_bytes: u64,
) -> io::Result<()> {
    let mut encoder = flate2::GzBuilder::new().mtime(0).write(
        BoundedWriter::new(writer, max_compressed_bytes),
        compression,
    );
    encoder.write_all(bytes)?;
    encoder.finish()?;
    Ok(())
}

/// A gzip input, decoding or complete-consumption failure.
#[derive(Debug)]
pub enum GzipError {
    /// Complete compressed input exceeds its allowance before decoding.
    InputLimit {
        /// Observed compressed byte count.
        actual: usize,
        /// Maximum compressed bytes permitted by the caller.
        limit: usize,
    },
    /// Gzip decoding, payload integrity, decoded size or allocation failed.
    Decode(ArtifactError),
    /// Bytes follow the first member, including another gzip member.
    TrailingData,
}

impl fmt::Display for GzipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLimit { actual, limit } => {
                write!(f, "compressed input has {actual} bytes, exceeding {limit}")
            }
            Self::Decode(_) => f.write_str("bounded gzip decoding failed"),
            Self::TrailingData => f.write_str("bytes follow the single gzip member"),
        }
    }
}

impl std::error::Error for GzipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(source) => Some(source),
            _ => None,
        }
    }
}

/// Decode exactly one complete gzip member into bounded, fallible storage.
///
/// Checks the complete compressed input allowance before parsing. Independently
/// bounds decoded bytes, checks payload CRC and length, and rejects concatenated
/// members or any trailing bytes. A valid empty member is accepted with a zero
/// decoded allowance. Input remains unchanged on every return path.
///
/// Compressed and decoded bytes may be resident together. Callers own limits,
/// digest admission and payload interpretation: decoding establishes neither
/// Wasm validity nor an admitted executable identity. Verify downloaded archive
/// digests before decoding, as `archive::extract_tar_gz` (when the archive feature is enabled) does.
///
/// # Errors
/// Returns typed input overflow, gzip/read/integrity, decoded overflow,
/// allocation or trailing-data failures. No partial decoded bytes are returned.
pub fn decode_gzip(
    bytes: &[u8],
    max_compressed_bytes: usize,
    max_decoded_bytes: usize,
) -> Result<Vec<u8>, GzipError> {
    if bytes.len() > max_compressed_bytes {
        return Err(GzipError::InputLimit {
            actual: bytes.len(),
            limit: max_compressed_bytes,
        });
    }
    let mut decoder = flate2::bufread::GzDecoder::new(bytes);
    let payload = read_reader(&mut decoder, max_decoded_bytes).map_err(GzipError::Decode)?;
    if !decoder.into_inner().is_empty() {
        return Err(GzipError::TrailingData);
    }
    Ok(payload)
}
