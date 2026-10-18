//! Strict owned IC certificate/hash-tree decoding, without authentication.
//!
//! Caller owns trust roots, signatures, combined evidence budgets, commitment
//! verification and application leaf meanings. Original bytes stay caller-owned.

use ic_platform::certification::{
    Certificate, Delegation, HashTree, empty, fork, labeled, leaf, pruned,
};
use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeOwned, IgnoredAny, SeqAccess, Visitor},
};
use std::{fmt, io};

#[cfg(test)]
mod tests;

/// Explicit complete-input byte and CBOR container-depth allowances.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    /// Maximum bytes supplied to one decoding operation.
    pub bytes: usize,
    /// Maximum nesting, using Ciborium's recursion-limit contract.
    /// Callers must choose a stack-safe bound; this is not a heap/CPU quota.
    pub depth: u8,
}

/// CBOR grammar, field/type, hash-tree shape or nesting failure.
#[derive(Debug, thiserror::Error)]
#[error("IC CBOR decoding failed")]
pub struct ParseError {
    source: ciborium::de::Error<io::Error>,
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
    let wire: CertificateWire = decode(bytes, limits)?;
    Ok(Certificate {
        tree: wire.tree.0,
        signature: wire.signature.0,
        delegation: wire.delegation.map(|wire| Delegation {
            subnet_id: wire.subnet_id.0,
            certificate: wire.certificate.0,
        }),
    })
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
    decode::<OwnedTree>(bytes, limits).map(|tree| tree.0)
}

fn decode<T: DeserializeOwned>(mut bytes: &[u8], limits: DecodeLimits) -> Result<T, DecodeError> {
    if bytes.len() > limits.bytes {
        return Err(DecodeError::ByteLimit {
            actual: bytes.len(),
            limit: limits.bytes,
        });
    }
    let value =
        ciborium::de::from_reader_with_recursion_limit(&mut bytes, usize::from(limits.depth))
            .map_err(|source| DecodeError::Cbor(ParseError { source }))?;
    if !bytes.is_empty() {
        return Err(DecodeError::TrailingData);
    }
    Ok(value)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CertificateWire {
    tree: OwnedTree,
    signature: ByteString,
    #[serde(default)]
    delegation: Option<DelegationWire>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DelegationWire {
    subnet_id: ByteString,
    certificate: ByteString,
}

struct ByteString(Vec<u8>);
struct ByteVisitor;
impl<'de> Deserialize<'de> for ByteString {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // deserialize_bytes permits integer arrays in some codecs; any + a
        // byte-only visitor admits the actual CBOR major type instead.
        d.deserialize_any(ByteVisitor)
    }
}
impl Visitor<'_> for ByteVisitor {
    type Value = ByteString;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CBOR byte string")
    }
    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        Ok(ByteString(bytes.to_vec()))
    }
    fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
        Ok(ByteString(bytes))
    }
}

struct Tag(u64);
struct TagVisitor;
impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(TagVisitor)
    }
}
impl Visitor<'_> for TagVisitor {
    type Value = Tag;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unsigned IC hash-tree tag")
    }
    fn visit_u64<E: de::Error>(self, tag: u64) -> Result<Self::Value, E> {
        Ok(Tag(tag))
    }
}

struct OwnedTree(HashTree);
struct TreeVisitor;
impl<'de> Deserialize<'de> for OwnedTree {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Ciborium's deserialize_seq can expose byte strings as integer arrays.
        // The any visitor accepts only actual arrays and rejects that coercion.
        d.deserialize_any(TreeVisitor)
    }
}
impl<'de> Visitor<'de> for TreeVisitor {
    type Value = OwnedTree;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IC hash-tree array")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let Tag(tag) = required(&mut seq)?;
        let tree = match tag {
            0 => empty(),
            1 => fork(
                required::<_, OwnedTree>(&mut seq)?.0,
                required::<_, OwnedTree>(&mut seq)?.0,
            ),
            2 => labeled(
                required::<_, ByteString>(&mut seq)?.0,
                required::<_, OwnedTree>(&mut seq)?.0,
            ),
            3 => leaf(required::<_, ByteString>(&mut seq)?.0),
            4 => pruned(
                <[u8; 32]>::try_from(required::<_, ByteString>(&mut seq)?.0)
                    .map_err(|_| de::Error::custom("pruned IC digest must have 32 bytes"))?,
            ),
            _ => return Err(de::Error::custom("unknown IC hash-tree tag")),
        };
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(de::Error::custom("extra IC hash-tree child"));
        }
        Ok(OwnedTree(tree))
    }
}
fn required<'de, A: SeqAccess<'de>, T: Deserialize<'de>>(seq: &mut A) -> Result<T, A::Error> {
    seq.next_element()?
        .ok_or_else(|| de::Error::custom("missing IC hash-tree element"))
}
