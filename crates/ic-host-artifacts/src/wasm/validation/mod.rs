//! Full core-module validation, separate from structural inspection.

use super::ParseError;
use std::fmt;
use wasmparser::{Parser, Validator};

/// Full validation failed without exposing upstream parser or type identities.
#[derive(Debug)]
pub enum ValidationError {
    /// Input bytes exceed the caller's allowance, checked before validation.
    LimitExceeded {
        /// Observed complete input length.
        actual: usize,
        /// Caller-selected maximum input length.
        limit: usize,
    },
    /// A component was supplied where a core module was required.
    UnsupportedEncoding,
    /// Malformed structure, invalid types, instructions or unsupported features.
    Invalid(ParseError),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded { actual, limit } => {
                write!(formatter, "Wasm module has {actual} bytes, limit {limit}")
            }
            Self::UnsupportedEncoding => {
                formatter.write_str("expected a core Wasm module, received a component")
            }
            Self::Invalid(source) => write!(formatter, "invalid Wasm module: {source}"),
        }
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid(source) => Some(source),
            Self::LimitExceeded { .. } | Self::UnsupportedEncoding => None,
        }
    }
}

/// Validate a complete core Wasm module, including types and every function body.
///
/// Uses the selected parser's default supported proposal set and full validator,
/// not [`super::inspect`] or an instruction scan. Requires `wasm-validation`.
/// No upstream validator, parser, feature flags or validated types escape this API.
///
/// The byte allowance is checked before parsing or validator allocation. It is
/// not a CPU/heap quota. Consumers retain input acquisition, execution deadlines,
/// target-specific feature admission, IC install policy and deployment decisions.
/// No Wasm is executed. Inspection remains independently available and unchanged.
///
/// # Errors
/// Returns byte-limit or encoding refusals, or an owned validation diagnostic
/// with its original byte offset. Diagnostic wording is not a stable category.
pub fn validate(bytes: &[u8], max_bytes: usize) -> Result<(), ValidationError> {
    if bytes.len() > max_bytes {
        return Err(ValidationError::LimitExceeded {
            actual: bytes.len(),
            limit: max_bytes,
        });
    }
    if Parser::is_component(bytes) {
        return Err(ValidationError::UnsupportedEncoding);
    }
    Validator::new()
        .validate_all(bytes)
        .map(|_| ())
        .map_err(|source| ValidationError::Invalid(ParseError { source }))
}

#[cfg(test)]
mod tests;
