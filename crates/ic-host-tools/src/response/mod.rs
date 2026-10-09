//! Bounded decoding of caller-selected ICP CLI response formats.
//!
//! This boundary returns opaque bytes. Consumers own Candid decoding, canister
//! rejection semantics, CLI version admission, capture limits and replay policy.
//! No command is executed and no output format is inferred.

#[cfg(test)]
mod tests;

use serde::de::{Deserialize, Deserializer, IgnoredAny, MapAccess, Visitor};
use serde_json::error::Category;
use std::{collections::TryReserveError, fmt};

/// The output format explicitly requested by the caller's CLI command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseFormat {
    /// An object containing a top-level string `response_bytes` of compact hex.
    /// Unknown fields are ignored; a missing or null field is rejected. Empty
    /// hex represents empty bytes. Duplicate `response_bytes` fields are rejected.
    Json,
    /// Nonempty hex, permitting ASCII space, tab, CR, LF and form feed between
    /// any two digits (Rust's `is_ascii_whitespace` set).
    Hex,
    /// An exact `response (hex):` prefix followed by nonempty whitespace-tolerant
    /// hex. Leading ASCII whitespace is permitted; preambles and repeated labels
    /// are rejected rather than discarded.
    LabeledHex,
}

/// Caller-owned bounds for input parsing and decoded byte allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponseLimits {
    /// Maximum complete input length, including JSON metadata or the hex label.
    pub input_bytes: usize,
    /// Maximum decoded byte length, independent of the input allowance.
    pub decoded_bytes: usize,
}

/// JSON parser failure category without response values or parser error prose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsonErrorKind {
    /// JSON grammar or encoding is invalid.
    Syntax,
    /// The JSON shape does not match the envelope, including duplicate fields.
    Data,
    /// The input ends before the JSON value is complete.
    EndOfInput,
    /// The underlying JSON parser reported an I/O failure.
    Io,
}

/// Response decoding failed. Formatting never includes response contents.
#[derive(Debug)]
pub enum ResponseError {
    /// Complete input exceeds its allowance, before any parsing.
    InputLimit {
        /// Caller-selected input allowance.
        limit: usize,
    },
    /// JSON input contains invalid UTF-8, including in ignored metadata.
    InvalidUtf8 {
        /// First invalid byte offset in the complete input.
        offset: usize,
    },
    /// JSON could not be decoded as the canonical envelope.
    Json {
        /// Structured parser category.
        kind: JsonErrorKind,
        /// One-based parser line, or zero if unavailable.
        line: usize,
        /// Parser column, or zero if unavailable.
        column: usize,
    },
    /// The top-level response field is absent or null.
    MissingResponseBytes,
    /// Labeled hex does not begin with the exact admitted label.
    MissingHexLabel,
    /// The selected text hex format contains no digits.
    EmptyHex,
    /// A byte is not a hex digit or permitted whitespace.
    InvalidHex {
        /// Byte offset within the hex portion (within the decoded JSON string
        /// for JSON input), not necessarily an offset in the original input.
        offset: usize,
    },
    /// The hex portion contains an odd number of digits.
    OddHexLength,
    /// Valid hex would exceed the decoded byte allowance.
    DecodedLimit {
        /// Caller-selected decoded allowance.
        limit: usize,
    },
    /// Storage could not be reserved for decoded bytes.
    Allocation(TryReserveError),
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLimit { limit } => write!(f, "response input exceeds {limit} bytes"),
            Self::InvalidUtf8 { offset } => {
                write!(f, "response JSON is not UTF-8 at byte {offset}")
            }
            Self::Json { kind, line, column } => {
                write!(f, "response JSON failed ({kind:?}) at {line}:{column}")
            }
            Self::MissingResponseBytes => f.write_str("response JSON lacks response_bytes"),
            Self::MissingHexLabel => f.write_str("response hex label is missing"),
            Self::EmptyHex => f.write_str("response hex is empty"),
            Self::InvalidHex { offset } => write!(f, "invalid response hex at byte {offset}"),
            Self::OddHexLength => f.write_str("response hex has an odd digit count"),
            Self::DecodedLimit { limit } => write!(f, "decoded response exceeds {limit} bytes"),
            Self::Allocation(source) => write!(f, "response allocation failed: {source}"),
        }
    }
}

impl std::error::Error for ResponseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

/// Decode one complete response without dispatching a command or retrying it.
///
/// Input size is checked before parsing. `serde_json` owns JSON grammar and
/// escaped strings; unknown metadata is skipped without building a value tree.
/// JSON parser storage is bounded by the input allowance; its allocations are
/// owned by Serde. Decoded bytes are reserved fallibly only after validating all
/// hex and its independent allowance. Text hex permits only Rust ASCII whitespace;
/// JSON hex is compact, even at its edges. Candid validity is not checked.
///
/// # Errors
/// Returns typed size, JSON, envelope, hex or allocation failures. The caller
/// retains the original input for any explicitly authorized diagnostics.
pub fn decode(
    input: &[u8],
    format: ResponseFormat,
    limits: ResponseLimits,
) -> Result<Vec<u8>, ResponseError> {
    if input.len() > limits.input_bytes {
        return Err(ResponseError::InputLimit {
            limit: limits.input_bytes,
        });
    }
    match format {
        ResponseFormat::Json => {
            std::str::from_utf8(input).map_err(|error| ResponseError::InvalidUtf8 {
                offset: error.valid_up_to(),
            })?;
            let envelope: Envelope =
                serde_json::from_slice(input).map_err(|error| json_error(&error))?;
            let hex = envelope.0.ok_or(ResponseError::MissingResponseBytes)?;
            decode_hex(hex.as_bytes(), false, limits.decoded_bytes)
        }
        ResponseFormat::Hex => decode_hex(input, true, limits.decoded_bytes),
        ResponseFormat::LabeledHex => {
            let input = input.trim_ascii_start();
            let hex = input
                .strip_prefix(b"response (hex):")
                .ok_or(ResponseError::MissingHexLabel)?;
            decode_hex(hex, true, limits.decoded_bytes)
        }
    }
}

fn decode_hex(hex: &[u8], whitespace: bool, limit: usize) -> Result<Vec<u8>, ResponseError> {
    let mut digits = 0;
    for (offset, &byte) in hex.iter().enumerate() {
        if byte.is_ascii_hexdigit() {
            digits += 1;
        } else if !(whitespace && byte.is_ascii_whitespace()) {
            return Err(ResponseError::InvalidHex { offset });
        }
    }
    // Only text formats require a digit; JSON permits an empty byte response.
    if whitespace && digits == 0 {
        return Err(ResponseError::EmptyHex);
    }
    if digits % 2 != 0 {
        return Err(ResponseError::OddHexLength);
    }
    if digits / 2 > limit {
        return Err(ResponseError::DecodedLimit { limit });
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(digits / 2)
        .map_err(ResponseError::Allocation)?;
    let mut high = None;
    for byte in hex.iter().copied().filter(u8::is_ascii_hexdigit) {
        // The validation pass admits only these digit ranges.
        let nibble = if byte <= b'9' {
            byte - b'0'
        } else {
            byte.to_ascii_lowercase() - b'a' + 10
        };
        if let Some(previous) = high.take() {
            bytes.push((previous << 4) | nibble);
        } else {
            high = Some(nibble);
        }
    }
    Ok(bytes)
}

fn json_error(error: &serde_json::Error) -> ResponseError {
    let kind = match error.classify() {
        Category::Io => JsonErrorKind::Io,
        Category::Syntax => JsonErrorKind::Syntax,
        Category::Data => JsonErrorKind::Data,
        Category::Eof => JsonErrorKind::EndOfInput,
    };
    ResponseError::Json {
        kind,
        line: error.line(),
        column: error.column(),
    }
}

struct Envelope(Option<String>);

impl<'de> Deserialize<'de> for Envelope {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EnvelopeVisitor;
        impl<'de> Visitor<'de> for EnvelopeVisitor {
            type Value = Envelope;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an ICP response object")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Envelope, M::Error> {
                let mut response = None;
                let mut seen = false;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "response_bytes" {
                        if seen {
                            return Err(serde::de::Error::duplicate_field("response_bytes"));
                        }
                        seen = true;
                        response = map.next_value::<Option<String>>()?;
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
                Ok(Envelope(response))
            }
        }
        deserializer.deserialize_map(EnvelopeVisitor)
    }
}
