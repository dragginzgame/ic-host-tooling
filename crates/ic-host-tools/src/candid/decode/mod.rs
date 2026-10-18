//! Bounded single-value Candid decoding with caller-owned resource policy.
//!
//! This module does not authenticate replies or interpret application errors.

use ic_platform::candid::{self as codec, contract::CandidType};
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
    Candid(#[source] codec::contract::Error),
}

/// Decode one result using Candid's compatibility rules, charging admitted bytes.
///
/// Oversized replies do not consume aggregate allowance. Once admitted, a reply
/// consumes its bytes before decoding, so malformed replies cannot evade accounting.
/// Caller policy owns quotas; this helper never authenticates the decoded value.
/// Extra wire arguments are skipped under the selected work quotas, like Candid's
/// decoder; trailing undecodable bytes reject. These are byte and decoder-work
/// limits, not a heap, stack or elapsed-time guarantee.
/// Platform selects compact Candid diagnostics, which can still include small
/// inputs. Callers retain error-disclosure policy and the original typed cause.
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
    codec::decode_one(
        bytes,
        codec::DecodeLimits {
            reply_bytes: limits.reply_bytes,
            decoding_quota: limits.decoding_quota,
            skipping_quota: limits.skipping_quota,
            type_entries: limits.type_entries,
            header_bytes: limits.header_bytes,
        },
    )
    .map_err(|error| match error {
        codec::DecodeError::ReplyTooLarge { .. } => DecodeError::ReplyBytes,
        codec::DecodeError::Candid(source) => DecodeError::Candid(source),
    })
}
