//! Bounded single-value Candid decoding with caller-owned resource policy.
//!
//! This module does not authenticate replies or interpret application errors.

use candid_parser::candid::{CandidType, de::DecoderConfig, utils::decode_one_with_config};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[cfg(test)]
mod tests;

/// Explicit per-reply byte and decoder work limits. Zero remains a real limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    /// Maximum encoded reply bytes.
    pub reply_bytes: usize,
    /// Maximum Candid decoding work.
    pub decoding_quota: usize,
    /// Maximum Candid skipped-field work.
    pub skipping_quota: usize,
    /// Maximum number of entries in the type table.
    pub type_entries: usize,
    /// Maximum encoded header bytes.
    pub header_bytes: usize,
}

/// Resource admission failures are distinct from malformed or incompatible Candid.
#[derive(Debug, Error)]
pub enum DecodeError {
    /// The encoded reply exceeds the per-reply allowance.
    #[error("Candid reply exceeds its byte limit")]
    ReplyBytes,
    /// The reply cannot be admitted within the aggregate allowance.
    #[error("Candid reply exceeds the remaining aggregate byte budget")]
    AggregateBytes,
    /// Malformed, incompatible or decoder-quota-exhausting Candid.
    #[error("Candid reply could not be decoded: {0}")]
    Candid(#[source] candid_parser::candid::Error),
}

/// Decode exactly one value, charging admitted bytes even when decoding fails.
///
/// Oversized replies do not consume aggregate allowance. Once admitted, a reply
/// consumes its bytes before decoding, so malformed replies cannot evade accounting.
/// Caller policy owns quotas; this helper never authenticates the decoded value.
///
/// # Errors
/// Returns byte admission failure before decoding, or the original Candid error.
pub fn decode_one<T: CandidType + DeserializeOwned>(
    bytes: &[u8],
    limits: DecodeLimits,
    remaining: &mut usize,
) -> Result<T, DecodeError> {
    if bytes.len() > limits.reply_bytes {
        return Err(DecodeError::ReplyBytes);
    }
    *remaining = remaining
        .checked_sub(bytes.len())
        .ok_or(DecodeError::AggregateBytes)?;
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(limits.decoding_quota)
        .set_skipping_quota(limits.skipping_quota)
        .set_max_type_len(limits.type_entries)
        .set_max_header_len(limits.header_bytes);
    decode_one_with_config(bytes, &config).map_err(DecodeError::Candid)
}
