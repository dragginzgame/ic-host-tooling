//! IC method export interpretation over the artifact owner's structural facts.
//!
//! No expected service, internal-export allowlist, signature validation or
//! installation policy is implied. Source names are preserved without trimming.

use ic_host_artifacts::wasm::{ExportKind, WasmFacts};

#[cfg(test)]
mod tests;

/// Candid method mode, retaining the distinction between update and oneway.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MethodMode {
    /// Ordinary update call.
    Update,
    /// Query call.
    Query,
    /// Composite query call.
    CompositeQuery,
    /// Candid oneway update, with no reply.
    Oneway,
}

impl MethodMode {
    /// Project Candid call semantics to the corresponding Wasm export mode.
    /// Wasm has no separate oneway export namespace.
    #[must_use]
    pub const fn export_mode(self) -> Self {
        match self {
            Self::Oneway => Self::Update,
            mode => mode,
        }
    }
}

/// Borrowed method identity for comparing declared and exported interfaces.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExportedMethod<'a> {
    /// Complete method name after its IC export prefix, including whitespace.
    pub name: &'a str,
    /// Export namespace: update, query or composite query, never oneway.
    pub mode: MethodMode,
}

/// Interpret function exports in the three IC method namespaces.
///
/// Uses already-inspected, caller-bounded Wasm facts; no source is reparsed or
/// allocation performed. Lifecycle and non-function exports are omitted. Internal
/// CDK methods, empty names and the same name in different modes are retained as
/// facts for explicit caller admission. This does not validate Wasm signatures.
pub fn exported_methods<'a>(
    facts: &'a WasmFacts<'_>,
) -> impl Iterator<Item = ExportedMethod<'a>> + 'a {
    facts.exports.iter().filter_map(|(name, export)| {
        if export.kind != ExportKind::Function {
            return None;
        }
        [
            ("canister_update ", MethodMode::Update),
            ("canister_query ", MethodMode::Query),
            ("canister_composite_query ", MethodMode::CompositeQuery),
        ]
        .into_iter()
        .find_map(|(prefix, mode)| {
            name.strip_prefix(prefix)
                .map(|name| ExportedMethod { name, mode })
        })
    })
}
