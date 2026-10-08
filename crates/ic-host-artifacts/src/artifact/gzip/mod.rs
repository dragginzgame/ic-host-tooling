//! Bounded single-member gzip encoding and decoding for host artifacts.

#[cfg(test)]
mod tests;

use super::{
    ArtifactError, ArtifactIdentity, BoundedWriter, MatchingWriter, hash_reader, read_reader,
    visit_reader,
};
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
/// `compression_level` is 0 (stored) through 9 (best compression); 1 is fastest
/// and 6 is the usual balanced setting. The format-level value exposes no
/// compression-backend dependency type.
///
/// # Errors
/// Returns compressor, sink or output-limit errors. Limit errors carry
/// [`super::WriterError`] inside [`io::Error`].
/// Levels above 9 return `InvalidInput` before writing to the sink.
pub fn encode_gzip(
    bytes: &[u8],
    writer: impl Write,
    compression_level: u32,
    max_compressed_bytes: u64,
) -> io::Result<()> {
    if compression_level > 9 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "gzip compression level must be 0 through 9",
        ));
    }
    let mut encoder = flate2::GzBuilder::new().mtime(0).write(
        BoundedWriter::new(writer, max_compressed_bytes),
        flate2::Compression::new(compression_level),
    );
    encoder.write_all(bytes)?;
    encoder.finish()?;
    Ok(())
}

/// A gzip input, decoding or complete-consumption failure.
#[derive(Debug)]
pub enum GzipError {
    /// Complete supplied input exceeds its allowance before decoding or hashing.
    InputLimit {
        /// Observed input byte count.
        actual: usize,
        /// Maximum input bytes permitted by the caller.
        limit: usize,
    },
    /// Payload decoding, integrity, decoded/raw size or allocation failed.
    Decode(ArtifactError),
    /// Bytes follow the first member, including another gzip member.
    TrailingData,
}

impl fmt::Display for GzipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLimit { actual, limit } => {
                write!(f, "input has {actual} bytes, exceeding {limit}")
            }
            Self::Decode(_) => f.write_str("bounded payload processing failed"),
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
    with_decoder(bytes, max_compressed_bytes, |decoder| {
        read_reader(decoder, max_decoded_bytes)
    })
}

/// Hash one complete gzip member without allocating its decoded payload.
///
/// Applies the same header, CRC, length and single-member/trailing-data checks
/// as [`decode_gzip`]. Input is already borrowed in memory and admitted against
/// `max_compressed_bytes` before parsing. Decoding uses fixed-size traversal
/// storage plus the decoder's working memory, independently of payload size.
/// The returned identity describes decoded bytes, not compressed input.
///
/// # Errors
/// Returns [`GzipError`] for input/decoded limits, malformed or incomplete gzip,
/// failed integrity checks or trailing bytes. No partial identity is returned.
pub fn hash_gzip(
    bytes: &[u8],
    max_compressed_bytes: usize,
    max_decoded_bytes: u64,
) -> Result<ArtifactIdentity, GzipError> {
    with_decoder(bytes, max_compressed_bytes, |decoder| {
        hash_reader(decoder, max_decoded_bytes)
    })
}

/// Identify raw bytes or the decoded payload of a single gzip member.
///
/// Exactly the leading bytes `1f 8b` select gzip; malformed gzip never falls
/// back to raw hashing. All other inputs, including empty or one-byte inputs,
/// are raw. Both representations must fit `max_input_bytes`; the decoded or
/// raw payload must independently fit `max_payload_bytes`. Returns the same
/// identity for a raw payload and its valid gzip representation.
///
/// This is representation handling, not Wasm validation or IC install policy.
/// A caller comparing a canister module hash must independently admit Wasm and
/// select its limits. The complete supplied input is already resident; no
/// decoded payload buffer is allocated. Multi-member gzip is rejected.
///
/// # Errors
/// Returns the same errors as [`hash_gzip`]; raw payload-limit failures use
/// [`GzipError::Decode`]. No partial identity is returned.
pub fn hash_gzip_or_raw(
    bytes: &[u8],
    max_input_bytes: usize,
    max_payload_bytes: u64,
) -> Result<ArtifactIdentity, GzipError> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        hash_gzip(bytes, max_input_bytes, max_payload_bytes)
    } else {
        check_input_limit(bytes, max_input_bytes)?;
        hash_reader(bytes, max_payload_bytes).map_err(GzipError::Decode)
    }
}

/// Compare one complete gzip payload with borrowed expected bytes, without a copy.
///
/// Uses exact byte equality, not digest equality. `expected.len()` is also the
/// decoded byte budget: longer payloads fail with a decoded limit error; shorter
/// or different valid payloads return `false`. A mismatch still consumes the
/// bounded member so malformed trailers and trailing data remain errors.
/// Input is borrowed; no decoded payload is retained or published.
///
/// # Errors
/// Applies all [`decode_gzip`] input, integrity and complete-consumption checks.
/// A failed comparison does not return partial success or suppress gzip errors.
pub fn gzip_matches(
    bytes: &[u8],
    expected: &[u8],
    max_compressed_bytes: usize,
) -> Result<bool, GzipError> {
    with_decoder(bytes, max_compressed_bytes, |decoder| {
        let mut matching = MatchingWriter::new(expected);
        visit_reader::<ArtifactError>(decoder, expected.len() as u64, |chunk| {
            matching.write_all(chunk).map_err(ArtifactError::Io)
        })?;
        Ok(matching.is_complete_match())
    })
}

const fn check_input_limit(bytes: &[u8], max_compressed_bytes: usize) -> Result<(), GzipError> {
    if bytes.len() > max_compressed_bytes {
        return Err(GzipError::InputLimit {
            actual: bytes.len(),
            limit: max_compressed_bytes,
        });
    }
    Ok(())
}

// Every operation must consume through EOF (or fail) using the shared bounded
// traversal. One owner admits input and enforces exact single-member framing.
fn with_decoder<T>(
    bytes: &[u8],
    max_compressed_bytes: usize,
    consume: impl FnOnce(&mut flate2::bufread::GzDecoder<&[u8]>) -> Result<T, ArtifactError>,
) -> Result<T, GzipError> {
    check_input_limit(bytes, max_compressed_bytes)?;
    let mut decoder = flate2::bufread::GzDecoder::new(bytes);
    let payload = consume(&mut decoder).map_err(GzipError::Decode)?;
    if !decoder.into_inner().is_empty() {
        return Err(GzipError::TrailingData);
    }
    Ok(payload)
}
