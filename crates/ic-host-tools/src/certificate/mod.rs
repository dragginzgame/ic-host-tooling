//! Strict owned IC certificate/hash-tree decoding, without authentication.
//!
//! Caller owns trust roots, signatures, combined evidence budgets, commitment
//! verification and application leaf meanings. Original bytes stay caller-owned.

use ic_platform::certification::{
    Certificate, Delegation, HashTree, empty, fork, labeled, leaf, pruned,
};
use minicbor::{Decoder, data::Type, decode::Error as CborError};

#[cfg(test)]
mod tests;

/// Explicit complete-input byte and CBOR container-depth allowances.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    /// Maximum bytes supplied to one decoding operation.
    pub bytes: usize,
    /// Maximum nested certificate/delegation maps and hash-tree arrays.
    /// Transparent map/key tags and byte-string chunks consume no depth.
    /// Callers must choose a stack-safe bound; this is not a heap/CPU quota.
    pub depth: u8,
}

/// CBOR grammar, field/type, hash-tree shape or nesting failure.
#[derive(Debug, thiserror::Error)]
#[error("IC CBOR decoding failed")]
pub struct ParseError {
    source: CborError,
}

/// Decoding refused; no partial certificate or hash tree is returned.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// Complete input exceeds the allowance before parsing.
    #[error("IC CBOR input has {actual} bytes, exceeding {limit}")]
    ByteLimit {
        /// Observed input bytes.
        actual: usize,
        /// Caller-selected maximum.
        limit: usize,
    },
    /// Bytes remain after exactly one CBOR value.
    #[error("bytes follow the IC CBOR value")]
    TrailingData,
    /// CBOR syntax, structure, strict field/type or nesting admission failed.
    #[error("{0}")]
    Cbor(#[source] ParseError),
}

/// Decode one certificate into owned upstream IC-format values.
///
/// Required tree/signature and optional delegation fields are strict: duplicate,
/// unknown or incorrectly typed fields refuse. Signatures, labels, leaves and
/// delegation fields must be CBOR byte strings, not arrays or text. Delegated
/// certificate bytes remain opaque. No signature, timestamp, subnet, canister
/// commitment or trust-root validation is performed. A decoded certificate is
/// untrusted until authenticated by the caller.
///
/// # Errors
/// Returns byte-limit, CBOR/shape/nesting or trailing-data refusal.
pub fn decode_certificate(bytes: &[u8], limits: DecodeLimits) -> Result<Certificate, DecodeError> {
    decode(bytes, limits, read_certificate)
}

/// Decode one owned IC hash tree without a second generic CBOR payload tree.
///
/// Enforces exact tags 0–4 and node arities, byte-string payloads and exactly
/// 32 bytes for pruned digests. Both definite and indefinite arrays are checked.
/// Container nesting follows the selected CBOR bound. Storage uses ordinary
/// upstream Vec/Box allocations; the input bound is not an allocation quota.
/// Does not authenticate a tree or verify its commitment against a certificate.
///
/// # Errors
/// Returns byte-limit, CBOR/shape/nesting or trailing-data refusal.
pub fn decode_hash_tree(bytes: &[u8], limits: DecodeLimits) -> Result<HashTree, DecodeError> {
    decode(bytes, limits, read_tree)
}

fn decode<T>(
    bytes: &[u8],
    limits: DecodeLimits,
    read: impl FnOnce(&mut Decoder<'_>, u8) -> Result<T, CborError>,
) -> Result<T, DecodeError> {
    if bytes.len() > limits.bytes {
        return Err(DecodeError::ByteLimit {
            actual: bytes.len(),
            limit: limits.bytes,
        });
    }
    let mut decoder = Decoder::new(bytes);
    let value = read(&mut decoder, limits.depth)
        .map_err(|source| DecodeError::Cbor(ParseError { source }))?;
    if decoder.position() != bytes.len() {
        return Err(DecodeError::TrailingData);
    }
    Ok(value)
}

fn read_certificate(decoder: &mut Decoder<'_>, depth: u8) -> Result<Certificate, CborError> {
    let depth = descend(decoder, depth)?;
    skip_tags(decoder)?;
    let mut fields = decoder.map()?;
    let (mut tree, mut signature, mut delegation) = (None, None, None);
    while map_entry(decoder, &mut fields)? {
        match field(decoder)? {
            b"tree" if tree.is_none() => tree = Some(read_tree(decoder, depth)?),
            b"signature" if signature.is_none() => signature = Some(byte_string(decoder)?),
            b"delegation" if delegation.is_none() => {
                delegation = Some(match decoder.datatype()? {
                    Type::Null => {
                        decoder.null()?;
                        None
                    }
                    Type::Undefined => {
                        decoder.undefined()?;
                        None
                    }
                    _ => Some(read_delegation(decoder, depth)?),
                });
            }
            _ => return Err(invalid(decoder, "duplicate or unknown certificate field")),
        }
    }
    Ok(Certificate {
        tree: tree.ok_or_else(|| invalid(decoder, "missing certificate tree"))?,
        signature: signature.ok_or_else(|| invalid(decoder, "missing certificate signature"))?,
        delegation: delegation.flatten(),
    })
}

fn read_delegation(decoder: &mut Decoder<'_>, depth: u8) -> Result<Delegation, CborError> {
    let _ = descend(decoder, depth)?;
    skip_tags(decoder)?;
    let mut fields = decoder.map()?;
    let (mut subnet_id, mut certificate) = (None, None);
    while map_entry(decoder, &mut fields)? {
        match field(decoder)? {
            b"subnet_id" if subnet_id.is_none() => subnet_id = Some(byte_string(decoder)?),
            b"certificate" if certificate.is_none() => certificate = Some(byte_string(decoder)?),
            _ => return Err(invalid(decoder, "duplicate or unknown delegation field")),
        }
    }
    Ok(Delegation {
        subnet_id: subnet_id.ok_or_else(|| invalid(decoder, "missing delegation subnet"))?,
        certificate: certificate
            .ok_or_else(|| invalid(decoder, "missing delegation certificate"))?,
    })
}

fn read_tree(decoder: &mut Decoder<'_>, depth: u8) -> Result<HashTree, CborError> {
    let depth = descend(decoder, depth)?;
    let length = decoder.array()?;
    let tag = decoder.u64()?;
    let expected = match tag {
        0 => 1,
        1 | 2 => 3,
        3 | 4 => 2,
        _ => return Err(invalid(decoder, "unknown IC hash-tree tag")),
    };
    if length.is_some_and(|length| length != expected) {
        return Err(invalid(decoder, "incorrect IC hash-tree arity"));
    }
    let tree = match tag {
        0 => empty(),
        1 => fork(read_tree(decoder, depth)?, read_tree(decoder, depth)?),
        2 => labeled(byte_string(decoder)?, read_tree(decoder, depth)?),
        3 => leaf(byte_string(decoder)?),
        4 => pruned(
            <[u8; 32]>::try_from(byte_string(decoder)?)
                .map_err(|_| invalid(decoder, "pruned IC digest must have 32 bytes"))?,
        ),
        _ => return Err(invalid(decoder, "unknown IC hash-tree tag")),
    };
    if length.is_none() {
        if decoder.datatype()? != Type::Break {
            return Err(invalid(decoder, "extra IC hash-tree child"));
        }
        decoder.skip()?;
    }
    Ok(tree)
}

fn byte_string(decoder: &mut Decoder<'_>) -> Result<Vec<u8>, CborError> {
    let mut bytes = Vec::new();
    for chunk in decoder.bytes_iter()? {
        bytes.extend_from_slice(chunk?);
    }
    Ok(bytes)
}

fn field<'b>(decoder: &mut Decoder<'b>) -> Result<&'b [u8], CborError> {
    skip_tags(decoder)?;
    match decoder.datatype()? {
        Type::String => decoder.str().map(str::as_bytes),
        Type::Bytes => decoder.bytes(),
        _ => Err(invalid(
            decoder,
            "certificate field must be definite text or bytes",
        )),
    }
}

fn map_entry(decoder: &mut Decoder<'_>, remaining: &mut Option<u64>) -> Result<bool, CborError> {
    match remaining {
        Some(0) => Ok(false),
        Some(count) => {
            *count -= 1;
            Ok(true)
        }
        None if decoder.datatype()? == Type::Break => {
            decoder.skip()?;
            Ok(false)
        }
        None => Ok(true),
    }
}

fn skip_tags(decoder: &mut Decoder<'_>) -> Result<(), CborError> {
    // Typed map and field-name tags are transparent; tree/byte/tag values are not.
    while decoder.datatype()? == Type::Tag {
        decoder.tag()?;
    }
    Ok(())
}

fn descend(decoder: &Decoder<'_>, depth: u8) -> Result<u8, CborError> {
    depth
        .checked_sub(1)
        .ok_or_else(|| invalid(decoder, "IC CBOR nesting allowance exceeded"))
}

fn invalid(decoder: &Decoder<'_>, message: &'static str) -> CborError {
    CborError::message(message).at(decoder.position())
}
