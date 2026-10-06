//! Compare a producer's complete byte stream without retaining another copy.

use std::io::{self, Write};

/// A write sink that compares offered bytes with a caller-owned byte slice.
///
/// Mismatches are sticky but writes continue successfully so producer errors
/// are still observed. The caller must check that production succeeded before
/// accepting [`Self::is_complete_match`]. This chooses no encoding, schema,
/// canonical ordering, digest or publication policy.
pub struct MatchingWriter<'a> {
    expected: &'a [u8],
    position: usize,
    matches: bool,
}

impl<'a> MatchingWriter<'a> {
    /// Borrow the complete expected output.
    #[must_use]
    pub const fn new(expected: &'a [u8]) -> Self {
        Self {
            expected,
            position: 0,
            matches: true,
        }
    }

    /// Whether all bytes written so far exactly equal the complete expected slice.
    ///
    /// Does not assert that the producer finished successfully.
    #[must_use]
    pub const fn is_complete_match(&self) -> bool {
        self.matches && self.position == self.expected.len()
    }
}

impl Write for MatchingWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let end = self.position.checked_add(buffer.len()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "byte position exceeds usize")
        })?;
        if self.expected.get(self.position..end) != Some(buffer) {
            self.matches = false;
        }
        self.position = end;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
