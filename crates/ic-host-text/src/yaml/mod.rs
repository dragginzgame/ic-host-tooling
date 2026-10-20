//! Single-document UTF-8 YAML decoding with caller-owned byte admission and schemas.
//!
//! The private `serde_yaml` backend matches ICP CLI's package selection. Its built-in
//! recursion/repetition checks are retained; input-byte admission is not a bound
//! on parser memory, CPU or custom deserializer work. No IO or external expansion
//! is performed. Duplicate keys are rejected before typed schema projection.

use std::fmt;

use serde::de::DeserializeOwned;

/// Interpretation of YAML tags and merge-key spellings.
#[derive(Clone, Copy, Debug, Default)]
pub struct DecodePolicy {
    /// Accept tagged values, as needed for typed projections of ICP effective YAML.
    /// The default rejects non-core tags throughout the document.
    pub ignore_unknown_tags: bool,
    /// Preserve `<<` as an ordinary key instead of rejecting its spelling.
    /// The default rejects it, including quoted keys. Merges are never expanded.
    pub preserve_merge_keys: bool,
}

/// Refusal to decode one complete YAML document.
#[derive(Debug)]
pub enum DecodeError {
    /// The byte allowance was exceeded before parsing.
    InputTooLarge {
        /// Observed byte count.
        actual: usize,
        /// Caller-selected ceiling.
        maximum: usize,
    },
    /// No lossy or automatic character-set conversion is performed.
    Utf8(std::str::Utf8Error),
    /// Syntax, schema, document policy or a backend resource check failed.
    /// Diagnostic wording follows the private parser and is not a stable contract.
    InvalidDocument {
        /// Parser or policy diagnostic, which may contain input values.
        message: String,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge { actual, maximum } => {
                write!(
                    formatter,
                    "YAML input is {actual} bytes; maximum is {maximum}"
                )
            }
            Self::Utf8(error) => write!(formatter, "YAML input is not UTF-8: {error}"),
            Self::InvalidDocument { message } => write!(formatter, "invalid YAML: {message}"),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Utf8(error) => Some(error),
            _ => None,
        }
    }
}

/// Decode a single UTF-8 YAML document into a caller-owned Serde schema.
///
/// A private value tree admits duplicate keys and tag/merge policy throughout
/// the document before typed projection, including content ignored by the schema.
/// The result is owned. Parser recursion/repetition limits are backend-selected;
/// there are no caller-selected event, depth or allocation ceilings.
/// Diagnostics may contain offending values; callers retain redaction policy.
/// No files, environment variables or external includes are consulted.
///
/// # Errors
/// Refuses byte overflow, invalid UTF-8, malformed or multiple documents,
/// duplicate keys, disallowed extensions and schema mismatches.
pub fn from_slice<T: DeserializeOwned>(
    bytes: &[u8],
    maximum_input_bytes: usize,
    policy: DecodePolicy,
) -> Result<T, DecodeError> {
    if bytes.len() > maximum_input_bytes {
        return Err(DecodeError::InputTooLarge {
            actual: bytes.len(),
            maximum: maximum_input_bytes,
        });
    }
    let source = std::str::from_utf8(bytes).map_err(DecodeError::Utf8)?;
    let document: serde_yaml::Value =
        serde_yaml::from_str(source).map_err(|error| parse_error(&error))?;
    admit_document(&document, policy)?;
    serde_yaml::from_value(document).map_err(|error| parse_error(&error))
}

fn parse_error(error: &serde_yaml::Error) -> DecodeError {
    DecodeError::InvalidDocument {
        message: error.to_string(),
    }
}

fn admit_document(document: &serde_yaml::Value, policy: DecodePolicy) -> Result<(), DecodeError> {
    use serde_yaml::Value;

    let mut pending = vec![document];
    while let Some(value) = pending.pop() {
        match value {
            Value::Tagged(tagged) => {
                if !policy.ignore_unknown_tags {
                    return Err(DecodeError::InvalidDocument {
                        message: "unsupported YAML tag".into(),
                    });
                }
                pending.push(&tagged.value);
            }
            Value::Mapping(mapping) => {
                for (key, value) in mapping {
                    if !policy.preserve_merge_keys && key.as_str() == Some("<<") {
                        return Err(DecodeError::InvalidDocument {
                            message: "YAML merge keys are unsupported".into(),
                        });
                    }
                    pending.push(key);
                    pending.push(value);
                }
            }
            Value::Sequence(sequence) => pending.extend(sequence),
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
