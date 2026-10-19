//! Opt-in native fixtures that build small canisters returning caller-selected raw bytes.

use ic_host_artifacts::wasm_encoding::{
    Call, CallModule, DataSegment, ExportedFunction, FunctionImport, Memory32, encode,
};
use std::collections::BTreeSet;
use thiserror::Error;

///
/// ReplyMode
///
/// Explicit IC entry-point mode for one fixture method.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReplyMode {
    /// Export a non-replicated query entry point.
    Query,
    /// Export a replicated update entry point.
    Update,
}

///
/// RawReplyMethod
///
/// One method and its unmodified reply, including malformed Candid when required.
///

#[derive(Clone, Copy, Debug)]
pub struct RawReplyMethod<'a> {
    /// Nonempty method name without control characters; no export prefix.
    pub name: &'a str,
    /// Explicit query or update export.
    pub mode: ReplyMode,
    /// Exact bytes appended before replying; no decoding or normalization.
    pub reply: &'a [u8],
}

///
/// RawReplyLimits
///
/// Caller-selected aggregate admission ceilings checked before module construction.
///

#[derive(Clone, Copy, Debug)]
pub struct RawReplyLimits {
    /// Maximum number of exported methods.
    pub methods: usize,
    /// Maximum sum of all raw reply byte lengths.
    pub reply_bytes: usize,
    /// Maximum sum of all method-name UTF-8 byte lengths, excluding export prefixes.
    pub method_name_bytes: usize,
}

///
/// RawReplyError
///
/// Typed refusal before any simulator, network, or filesystem effects.
///

#[derive(Debug, Eq, Error, PartialEq)]
pub enum RawReplyError {
    /// An aggregate caller ceiling was exceeded, including length overflow.
    #[error("raw reply fixture exceeds {resource} limit {limit}")]
    Budget {
        /// Admission resource name.
        resource: &'static str,
        /// Selected ceiling.
        limit: usize,
    },
    /// Method names must be nonempty and contain no control characters.
    #[error("invalid method name at index {index}")]
    InvalidName {
        /// Input method position.
        index: usize,
    },
    /// Method names must be unique across query and update exports.
    #[error("duplicate method export at index {index}")]
    DuplicateExport {
        /// Input method position.
        index: usize,
    },
    /// A function index, reply extent or memory offset exceeds this fixture's Wasm32 model.
    #[error("raw reply fixture cannot represent its lengths in Wasm32")]
    Wasm32Limits,
}

/// Build a canister whose selected query/update methods return exact caller bytes.
///
/// No startup, installation, Candid policy or funding is selected. Reply storage
/// is contiguous and memory cannot grow. The caller owns all method/payload policy.
/// All caller ceilings and Wasm32 extents are checked before encoding the module.
///
/// ```
/// use ic_testkit::pic::fixtures::{
///     raw_reply_canister, RawReplyLimits, RawReplyMethod, ReplyMode,
/// };
/// let wasm = raw_reply_canister(
///     &[RawReplyMethod { name: "probe", mode: ReplyMode::Query, reply: b"raw" }],
///     RawReplyLimits { methods: 1, reply_bytes: 3, method_name_bytes: 5 },
/// )?;
/// assert!(wasm.starts_with(b"\0asm"));
/// # Ok::<(), ic_testkit::pic::fixtures::RawReplyError>(())
/// ```
///
/// # Errors
/// Returns a typed error for exceeded limits, invalid/duplicate names or unsupported extents.
pub fn raw_reply_canister(
    methods: &[RawReplyMethod<'_>],
    limits: RawReplyLimits,
) -> Result<Vec<u8>, RawReplyError> {
    if methods.len() > limits.methods {
        return Err(RawReplyError::Budget {
            resource: "methods",
            limit: limits.methods,
        });
    }
    let count = u32::try_from(methods.len()).map_err(|_| RawReplyError::Wasm32Limits)?;
    count.checked_add(2).ok_or(RawReplyError::Wasm32Limits)?;
    let mut exports = BTreeSet::new();
    let mut bytes = 0_usize;
    let mut names = 0_usize;
    for (index, method) in methods.iter().enumerate() {
        if method.name.is_empty() || method.name.chars().any(char::is_control) {
            return Err(RawReplyError::InvalidName { index });
        }
        if !exports.insert(method.name) {
            return Err(RawReplyError::DuplicateExport { index });
        }
        names = charge(
            names,
            method.name.len(),
            limits.method_name_bytes,
            "method names",
        )?;
        bytes = charge(bytes, method.reply.len(), limits.reply_bytes, "reply bytes")?;
    }
    i32::try_from(bytes).map_err(|_| RawReplyError::Wasm32Limits)?;
    let pages =
        u32::try_from(bytes.div_ceil(65_536).max(1)).map_err(|_| RawReplyError::Wasm32Limits)?;
    let export_names: Vec<_> = methods
        .iter()
        .map(|method| {
            let prefix = match method.mode {
                ReplyMode::Query => "canister_query",
                ReplyMode::Update => "canister_update",
            };
            format!("{prefix} {}", method.name)
        })
        .collect();
    let mut functions = Vec::with_capacity(methods.len());
    let mut data = Vec::with_capacity(methods.len());
    let mut offset = 0_i32;
    for (name, method) in export_names.iter().zip(methods) {
        let length = i32::try_from(method.reply.len()).map_err(|_| RawReplyError::Wasm32Limits)?;
        functions.push(ExportedFunction {
            name,
            calls: vec![
                Call {
                    import: 0,
                    arguments: vec![offset, length],
                },
                Call {
                    import: 1,
                    arguments: vec![],
                },
            ],
        });
        data.push(DataSegment {
            offset: offset.cast_unsigned(),
            bytes: method.reply,
        });
        offset += length;
    }
    // Testkit admits names, calls and memory/data extents above. Host owns the
    // remaining complete-module Wasm32 extent check; no smaller output cap is added.
    encode(
        &CallModule {
            imports: &[
                FunctionImport {
                    module: "ic0",
                    name: "msg_reply_data_append",
                    parameters: 2,
                },
                FunctionImport {
                    module: "ic0",
                    name: "msg_reply",
                    parameters: 0,
                },
            ],
            functions: &functions,
            memory: Memory32 {
                pages,
                export: "memory",
            },
            data: &data,
        },
        usize::MAX,
    )
    .map_err(|_| RawReplyError::Wasm32Limits)
}

fn charge(
    total: usize,
    amount: usize,
    limit: usize,
    resource: &'static str,
) -> Result<usize, RawReplyError> {
    total
        .checked_add(amount)
        .filter(|sum| *sum <= limit)
        .ok_or(RawReplyError::Budget { resource, limit })
}

#[cfg(test)]
mod tests;
