//! Interpret Registry canister-range shard keys and explicit interval bounds.

use super::proto::{CanisterId, RoutingTable};
use candid::Principal;

/// Decode one nonempty lowercase-hex key suffix into a routable canister principal.
///
/// The caller supplies the prefix and owns namespace selection and Registry trust.
/// # Errors
/// Refuses malformed suffixes and non-routable principal encodings.
pub fn canister_range_start_from_key(prefix: &str, key: &str) -> Result<Principal, String> {
    let suffix = key
        .strip_prefix(prefix)
        .filter(|suffix| suffix.len() == 20)
        .filter(|suffix| {
            suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        })
        .ok_or_else(|| format!("invalid canister-ranges key {key:?}"))?;
    let mut bytes = [0; 10];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&suffix[index * 2..index * 2 + 2], 16)
            .map_err(|error| format!("invalid canister-ranges key {key:?}: {error}"))?;
    }
    validate_routable_canister_id(&bytes, key)?;
    Principal::try_from_slice(&bytes)
        .map_err(|error| format!("invalid range-start principal in key {key:?}: {error}"))
}

/// Admit nonempty shard ranges within the caller's inclusive lower/exclusive upper bounds.
///
/// This does not prove sorted/disjoint ranges, subnet validity, complete shard
/// discovery or certificate authenticity. The first range may exceed the key bound.
/// # Errors
/// Refuses missing, inverted, non-routable or out-of-interval range endpoints.
pub fn validate_routing_table_shard_bounds(
    key: &str,
    lower_bound: &Principal,
    shard: &RoutingTable,
    next_start: Option<&Principal>,
) -> Result<(), String> {
    if shard.entries.is_empty() {
        return Err(format!("routing shard {key:?} is empty"));
    }
    for entry in &shard.entries {
        let range = entry
            .range
            .as_ref()
            .ok_or_else(|| format!("routing shard {key:?} has an entry without a range"))?;
        let start = routing_canister_id_bytes(range.start_canister_id.as_ref(), key, "start")?;
        let end = routing_canister_id_bytes(range.end_canister_id.as_ref(), key, "end")?;
        if start > end {
            return Err(format!("routing shard {key:?} has an inverted range"));
        }
        if start < lower_bound.as_slice() {
            return Err(format!(
                "routing shard {key:?} starts below the lower bound encoded by its key"
            ));
        }
        if next_start.is_some_and(|next| end >= next.as_slice()) {
            return Err(format!(
                "routing shard {key:?} extends into the next shard's key interval"
            ));
        }
    }
    Ok(())
}

fn routing_canister_id_bytes<'a>(
    canister: Option<&'a CanisterId>,
    shard_key: &str,
    boundary: &str,
) -> Result<&'a [u8], String> {
    let bytes = canister
        .and_then(|canister| canister.principal_id.as_ref())
        .map(|principal| principal.raw.as_slice())
        .ok_or_else(|| {
            format!("routing shard {shard_key:?} has a range without a {boundary} canister id")
        })?;
    validate_routable_canister_id(bytes, shard_key)?;
    Ok(bytes)
}

fn validate_routable_canister_id(bytes: &[u8], key: &str) -> Result<(), String> {
    if bytes.len() != 10 || bytes[8..] != [1, 1] {
        return Err(format!("{key:?} contains a non-routable canister id"));
    }
    Ok(())
}
