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

/// Incremental newline framing for process-output callbacks.
///
/// The byte allowance includes the newline. Completed frames borrow reusable
/// scratch until the callback returns. JSON, EOF acceptance, process lifetime
/// and deadlines remain caller policy. After any failure, stop using the framer;
/// no later input is admitted or implicitly drained.
pub struct LineFramer {
    maximum: u64,
    pending: Option<Vec<u8>>,
}

impl LineFramer {
    /// Create an empty framer with the caller's per-frame byte allowance.
    #[must_use]
    pub const fn new(maximum: u64) -> Self {
        Self {
            maximum,
            pending: Some(Vec::new()),
        }
    }

    /// Deliver each complete newline-terminated frame in order, retaining a tail.
    ///
    /// Oversized frames retain at most the allowance plus one refused byte.
    /// On failure, the rest of this supplied chunk is abandoned; the caller must
    /// stop the stream or choose its own recovery. Allocator capacity is not a
    /// byte allowance, and already emitted frames are not rolled back.
    ///
    /// # Errors
    /// Returns overflow, the original allocation/sink failure, or an invalid-input
    /// I/O error when called again after refusal. Display never includes bytes.
    pub fn feed(
        &mut self,
        chunk: &[u8],
        mut emit: impl FnMut(&[u8]) -> io::Result<()>,
    ) -> Result<(), FrameReadError> {
        let mut pending = self.pending.take().ok_or_else(refused)?;
        for fragment in chunk.split_inclusive(|byte| *byte == b'\n') {
            let remaining = self.maximum.saturating_sub(pending.len() as u64);
            if fragment.len() as u64 > remaining {
                let prefix = usize::try_from(remaining)
                    .map_err(|error| FrameReadError::Io(io::Error::other(error)))?
                    + 1;
                pending
                    .try_reserve_exact(prefix)
                    .map_err(|error| FrameReadError::Io(io::Error::other(error)))?;
                pending.extend_from_slice(&fragment[..prefix]);
                return Err(FrameReadError::TooLarge(pending));
            }
            pending
                .try_reserve_exact(fragment.len())
                .map_err(|error| FrameReadError::Io(io::Error::other(error)))?;
            pending.extend_from_slice(fragment);
            if pending.ends_with(b"\n") {
                emit(&pending).map_err(FrameReadError::Io)?;
                pending.clear();
            }
        }
        self.pending = Some(pending);
        Ok(())
    }

    /// Finish cleanly at a frame boundary, or return the owned unterminated tail.
    ///
    /// Callers may explicitly accept that tail when their protocol permits it.
    ///
    /// # Errors
    /// Returns an unterminated frame, or invalid input after an earlier refusal.
    pub fn finish(self) -> Result<(), FrameReadError> {
        let pending = self.pending.ok_or_else(refused)?;
        if pending.is_empty() {
            Ok(())
        } else {
            Err(FrameReadError::Unterminated(pending))
        }
    }
}

fn refused() -> FrameReadError {
    FrameReadError::Io(io::Error::new(
        io::ErrorKind::InvalidInput,
        "framer previously refused the stream",
    ))
}

#[cfg(test)]
mod tests;
