//! Selected, revision-bound IC resource comparisons, not install admission.
//!
//! This reports code-body bytes, defined functions and all globals. Callers own
//! target qualification, overrides and warning thresholds. A passing report says
//! nothing about instruction/type validity, supported imports/features, metadata,
//! memory, module/upload size or other replica checks. Generic Wasm facts remain
//! in `ic-host-artifacts`; only this adapter owns the IC reference values.

use ic_host_artifacts::wasm::WasmFacts;

#[cfg(test)]
mod tests;

/// Exact replica source reviewed for [`REFERENCE_LIMITS`].
pub const REFERENCE_REVISION: &str = "9499f64bda8bcf087188dd8a1bb594115640ba9e";

/// Reviewed source defaults, selected explicitly by the caller.
///
/// Code bytes use `MAX_CODE_SECTION_SIZE_IN_BYTES` and the parser's remaining
/// code-section size in [validation.rs](https://github.com/dfinity/ic/blob/9499f64bda8bcf087188dd8a1bb594115640ba9e/rs/embedders/src/wasm_utils/validation.rs).
/// Function/global defaults come from [embedders.rs](https://github.com/dfinity/ic/blob/9499f64bda8bcf087188dd8a1bb594115640ba9e/rs/config/src/embedders.rs).
/// The validator counts local functions and both local/imported globals. These
/// are source observations, not a claim about every deployed subnet or release.
pub const REFERENCE_LIMITS: InstallLimits = InstallLimits {
    code_body_bytes: 12 * 1024 * 1024,
    defined_functions: 50_000,
    globals: 1_000,
};

/// Caller-selected limits for the three reported structural resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstallLimits {
    /// Code bytes excluding the encoded function-count prefix.
    pub code_body_bytes: u64,
    /// Defined functions, excluding imported functions.
    pub defined_functions: u64,
    /// Defined and imported globals together.
    pub globals: u64,
}

/// One exact observation and its inclusive selected limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LimitUsage {
    /// Observed bytes or entries.
    pub observed: u64,
    /// Inclusive maximum.
    pub limit: u64,
}

impl LimitUsage {
    /// Whether the observation strictly exceeds the selected limit.
    #[must_use]
    pub const fn exceeded(self) -> bool {
        self.observed > self.limit
    }

    /// Remaining allowance, negative when exceeded; arithmetic cannot wrap.
    #[must_use]
    pub fn headroom(self) -> i128 {
        i128::from(self.limit) - i128::from(self.observed)
    }
}

/// Structural headroom only; this is not evidence that installation will succeed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstallReport {
    /// Remaining code-body allowance.
    pub code_body_bytes: LimitUsage,
    /// Remaining defined-function allowance.
    pub defined_functions: LimitUsage,
    /// Remaining total-global allowance.
    pub globals: LimitUsage,
}

impl InstallLimits {
    /// Compare already-inspected facts without reparsing or allocating.
    #[must_use]
    pub fn report(self, facts: &WasmFacts<'_>) -> InstallReport {
        InstallReport {
            code_body_bytes: LimitUsage {
                observed: facts.code_body_bytes as u64,
                limit: self.code_body_bytes,
            },
            defined_functions: LimitUsage {
                observed: u64::from(facts.defined_functions),
                limit: self.defined_functions,
            },
            globals: LimitUsage {
                observed: u64::from(facts.defined_globals) + u64::from(facts.imported_globals),
                limit: self.globals,
            },
        }
    }
}
