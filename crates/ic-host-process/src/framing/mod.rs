//! Caller-bounded newline framing without message schemas or process ownership.
use std::{
    fmt,
    io::{self, BufRead, Read},
};

/// A refused frame; display never prints its potentially sensitive bytes.
#[derive(Debug)]
pub enum FrameReadError {
    /// The stream ended before any frame bytes arrived.
    EndOfStream,
    /// The frame exceeded the caller's limit, including its newline.
    TooLarge(Vec<u8>),
    /// The stream ended with a partial frame and no newline.
    Unterminated(Vec<u8>),
    /// The underlying reader failed.
    Io(io::Error),
}

impl FrameReadError {
    /// Borrow a refused prefix, retained up to the caller limit plus one byte.
    #[must_use]
    pub fn rejected_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::TooLarge(bytes) | Self::Unterminated(bytes) => Some(bytes),
            Self::EndOfStream | Self::Io(_) => None,
        }
    }
}

impl fmt::Display for FrameReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndOfStream => formatter.write_str("frame stream closed"),
            Self::TooLarge(_) => formatter.write_str("frame exceeds caller byte limit"),
            Self::Unterminated(_) => formatter.write_str("frame is missing its newline"),
            Self::Io(error) => write!(formatter, "frame read failed: {error}"),
        }
    }
}

impl std::error::Error for FrameReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Read one frame, retaining its newline and leaving later frames buffered.
///
/// `maximum` includes the newline. At most one excess byte is consumed to detect
/// overflow; this bounds retained bytes, not allocator capacity or elapsed time.
/// The caller owns parsing, deadlines, thread/process lifetime and diagnostics.
/// A refusal does not resynchronize the channel; callers must stop it or choose
/// their own recovery policy.
/// # Errors
/// Distinguishes clean EOF, missing newline, overflow and original I/O failures.
pub fn read_line(reader: &mut impl BufRead, maximum: u64) -> Result<Vec<u8>, FrameReadError> {
    let mut bytes = Vec::new();
    reader
        .take(maximum.saturating_add(1))
        .read_until(b'\n', &mut bytes)
        .map_err(FrameReadError::Io)?;
    if bytes.is_empty() {
        return Err(FrameReadError::EndOfStream);
    }
    if bytes.len() as u64 > maximum {
        return Err(FrameReadError::TooLarge(bytes));
    }
    if !bytes.ends_with(b"\n") {
        return Err(FrameReadError::Unterminated(bytes));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests;
