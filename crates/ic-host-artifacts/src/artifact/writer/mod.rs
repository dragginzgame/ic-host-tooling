//! Bound caller-owned output without choosing its encoding or publication policy.

#[cfg(test)]
mod tests;

use std::{
    fmt,
    io::{self, Write},
};

/// A write rejected by the output boundary, carried inside [`io::Error`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriterError {
    /// The complete offered buffer would exceed the caller's byte allowance.
    LimitExceeded {
        /// Maximum bytes permitted across successful writes.
        limit: u64,
    },
    /// The underlying writer violated [`Write::write`]'s accepted-byte contract.
    InvalidWriteCount {
        /// Number of bytes offered to the underlying writer.
        offered: usize,
        /// Number of bytes it claimed to have accepted.
        written: usize,
    },
}

impl fmt::Display for WriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded { limit } => write!(f, "output exceeds {limit} bytes"),
            Self::InvalidWriteCount { offered, written } => {
                write!(
                    f,
                    "writer accepted {written} bytes from a {offered}-byte buffer"
                )
            }
        }
    }
}

impl std::error::Error for WriterError {}

/// A byte allowance and successful-write counter around a caller-owned sink.
///
/// Oversized offered buffers are rejected in full before invoking the sink.
/// Short writes count only accepted bytes. Read limits, encoding, hashing,
/// synchronization, atomic publication and cleanup stay with their owners.
/// A sink's internal buffering/allocation and partial effects on error are not
/// controlled here. Do not publish partial output after a failed producer.
///
/// Use [`io::sink`] to count serialized bytes without retaining them, or a
/// hashing dependency's existing `Write` implementation to bound its input.
pub struct BoundedWriter<W> {
    inner: W,
    limit: u64,
    bytes: u64,
    exceeded: bool,
}

impl<W> BoundedWriter<W> {
    /// Select the sink and the inclusive total accepted-byte allowance.
    #[must_use]
    pub const fn new(inner: W, limit: u64) -> Self {
        Self {
            inner,
            limit,
            bytes: 0,
            exceeded: false,
        }
    }

    /// Bytes reported as accepted by successful underlying writes.
    ///
    /// This is neither a durability receipt nor an assertion of serialization
    /// success. Underlying errors may have effects not reported by `Write`.
    #[must_use]
    pub const fn bytes_written(&self) -> u64 {
        self.bytes
    }

    /// Whether any offered buffer has been rejected by this byte allowance.
    ///
    /// Retained even when an encoder hides the underlying typed IO error.
    /// A limit rejection leaves the sink and counter unchanged; later writes
    /// that fit are permitted, without clearing this observation.
    #[must_use]
    pub const fn limit_exceeded(&self) -> bool {
        self.exceeded
    }

    /// Recover the original sink, including any partial output after failure.
    ///
    /// Does not flush, synchronize, retry, publish or discard output. Callers
    /// must explicitly flush through this wrapper when their sink requires it.
    #[must_use]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for BoundedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if !u64::try_from(buffer.len()).is_ok_and(|length| length <= self.limit - self.bytes) {
            self.exceeded = true;
            return Err(io::Error::other(WriterError::LimitExceeded {
                limit: self.limit,
            }));
        }
        let written = self.inner.write(buffer)?;
        if written > buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                WriterError::InvalidWriteCount {
                    offered: buffer.len(),
                    written,
                },
            ));
        }
        // The admitted buffer length fits u64, and written never exceeds it.
        self.bytes += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
