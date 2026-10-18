//! Pure wire encodings, independent of transport, trust and application fields.

use std::fmt;

#[cfg(test)]
mod tests;

/// A complete canonical unsigned `u64` LEB128 value could not be admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsignedLeb128Error {
    /// Input ended before a terminating byte.
    Truncated,
    /// The value or continuation exceeds the ten-byte `u64` representation.
    Overflow,
    /// Bytes follow the first terminating byte.
    TrailingData,
    /// The representation contains an unnecessary leading zero group.
    NonCanonical,
}

impl fmt::Display for UnsignedLeb128Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Truncated => "truncated unsigned LEB128",
            Self::Overflow => "unsigned LEB128 exceeds u64",
            Self::TrailingData => "bytes follow unsigned LEB128",
            Self::NonCanonical => "noncanonical unsigned LEB128",
        })
    }
}

impl std::error::Error for UnsignedLeb128Error {}

/// Decode exactly one minimally encoded unsigned `u64` LEB128 value.
///
/// Inspects at most ten bytes and allocates nothing. Trailing data takes
/// precedence over nonminimal encoding; a tenth group outside `u64` or with a
/// continuation bit is overflow. Callers own field labels and semantic limits.
///
/// # Errors
/// Returns a typed grammar, range or complete-input admission failure.
pub fn decode_canonical_unsigned_u64(bytes: &[u8]) -> Result<u64, UnsignedLeb128Error> {
    let mut value = 0_u64;
    for (index, byte) in bytes.iter().copied().take(10).enumerate() {
        let payload = byte & 0x7f;
        if index == 9 && (payload > 1 || byte & 0x80 != 0) {
            return Err(UnsignedLeb128Error::Overflow);
        }
        value |= u64::from(payload) << (index * 7);
        if byte & 0x80 == 0 {
            if index + 1 != bytes.len() {
                return Err(UnsignedLeb128Error::TrailingData);
            }
            if index > 0 && payload == 0 {
                return Err(UnsignedLeb128Error::NonCanonical);
            }
            return Ok(value);
        }
    }
    Err(UnsignedLeb128Error::Truncated)
}
