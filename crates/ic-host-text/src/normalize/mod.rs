//! Bounded line normalization without grammar, terminal or output policy.

use std::fmt;

#[cfg(test)]
mod tests;

/// UTF-8 or resource failure while normalizing text lines.
#[derive(Debug)]
pub enum NormalizationError {
    /// Input bytes exceed the caller's bound before decoding.
    InputLimit {
        /// Observed byte count.
        actual: usize,
        /// Maximum permitted bytes.
        limit: usize,
    },
    /// Input is not valid UTF-8; no lossy decoding is performed.
    Utf8(std::str::Utf8Error),
    /// Normalized text, including added line endings, would exceed the bound.
    OutputLimit {
        /// Maximum permitted normalized bytes.
        limit: usize,
    },
    /// Storage for normalized text could not be allocated.
    Allocation(std::collections::TryReserveError),
}

impl fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLimit { actual, limit } => {
                write!(
                    f,
                    "normalization input has {actual} bytes, exceeding {limit}"
                )
            }
            Self::Utf8(_) => f.write_str("normalization input is not UTF-8"),
            Self::OutputLimit { limit } => write!(f, "normalized text exceeds {limit} bytes"),
            Self::Allocation(_) => f.write_str("line normalization allocation failed"),
        }
    }
}
impl std::error::Error for NormalizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Utf8(source) => Some(source),
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

/// Normalize UTF-8 lines with an explicit input/output byte bound.
///
/// Uses `str::lines` / `trim_end`: remove each line's
/// trailing Unicode whitespace, emit LF endings, and preserve blank lines.
/// Empty input stays empty; a nonempty unterminated line gains one LF.
/// This does not parse a grammar, escape terminal controls or canonicalize a format.
///
/// # Errors
/// Returns input overflow, invalid UTF-8, normalized-output overflow or
/// allocation failure. No partial normalized text is returned.
pub fn normalize_lines(bytes: &[u8], max_bytes: usize) -> Result<String, NormalizationError> {
    if bytes.len() > max_bytes {
        return Err(NormalizationError::InputLimit {
            actual: bytes.len(),
            limit: max_bytes,
        });
    }
    let text = std::str::from_utf8(bytes).map_err(NormalizationError::Utf8)?;
    let mut length = 0_usize;
    for line in text.lines() {
        length = length
            .checked_add(line.trim_end().len())
            .and_then(|length| length.checked_add(1))
            .filter(|&length| length <= max_bytes)
            .ok_or(NormalizationError::OutputLimit { limit: max_bytes })?;
    }
    let mut normalized = String::new();
    normalized
        .try_reserve_exact(length)
        .map_err(NormalizationError::Allocation)?;
    for line in text.lines() {
        normalized.push_str(line.trim_end());
        normalized.push('\n');
    }
    Ok(normalized)
}
