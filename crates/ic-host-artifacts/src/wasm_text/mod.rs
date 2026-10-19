//! Optional WebAssembly Text compilation with private upstream tooling.

use std::fmt;

/// Explicit source and complete binary-output allowances.
#[derive(Clone, Copy, Debug)]
pub struct CompileLimits {
    /// Maximum UTF-8 source length admitted before invoking the compiler.
    pub source_bytes: usize,
    /// Maximum complete returned core-module length.
    pub module_bytes: usize,
}

/// Compilation refused without returning a partial module.
#[derive(Debug)]
pub enum CompileError {
    /// Source exceeds its allowance before any parsing.
    SourceLimit {
        /// Complete UTF-8 input length.
        actual: usize,
        /// Caller-selected source allowance.
        limit: usize,
    },
    /// Encoded output exceeds its allowance.
    ModuleLimit {
        /// Complete encoded length.
        actual: usize,
        /// Caller-selected output allowance.
        limit: usize,
    },
    /// Upstream lexing, parsing, name resolution or binary encoding failed.
    Invalid {
        /// Human-readable diagnostic; may contain source excerpts and is not stable.
        diagnostic: String,
    },
    /// Compilation did not produce a version-one core module.
    UnsupportedEncoding,
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceLimit { actual, limit } => {
                write!(formatter, "WAT source has {actual} bytes, limit {limit}")
            }
            Self::ModuleLimit { actual, limit } => {
                write!(formatter, "compiled Wasm has {actual} bytes, limit {limit}")
            }
            Self::Invalid { diagnostic } => write!(formatter, "invalid WAT: {diagnostic}"),
            Self::UnsupportedEncoding => formatter.write_str("expected a core Wasm module"),
        }
    }
}

impl std::error::Error for CompileError {}

/// Compile in-memory WebAssembly Text into a complete core-module binary.
///
/// The private `wat` backend handles the full core text language, including
/// instruction bodies and symbolic names. Component syntax is disabled. No file
/// loading, IC naming, fixture policy, execution or installation is supplied.
/// Source admission precedes compilation; the complete output allowance is checked
/// afterwards. These byte allowances are not a peak-allocation or CPU quota.
/// Diagnostic text may include untrusted source; callers own its presentation.
///
/// Compilation is not instruction/type validation: select `wasm-validation` and
/// call `ic_host_artifacts::wasm::validate` separately when that proof is needed.
/// Neither that feature nor the binary inspector is implied by `wasm-text`.
///
/// # Errors
/// Returns source/output limits, a compiler diagnostic or non-core encoding refusal.
pub fn compile(source: &str, limits: CompileLimits) -> Result<Vec<u8>, CompileError> {
    if source.len() > limits.source_bytes {
        return Err(CompileError::SourceLimit {
            actual: source.len(),
            limit: limits.source_bytes,
        });
    }
    let module = wat::parse_str(source).map_err(|source| CompileError::Invalid {
        diagnostic: source.to_string(),
    })?;
    if module.len() > limits.module_bytes {
        return Err(CompileError::ModuleLimit {
            actual: module.len(),
            limit: limits.module_bytes,
        });
    }
    if !module.starts_with(b"\0asm\x01\0\0\0") {
        return Err(CompileError::UnsupportedEncoding);
    }
    Ok(module)
}

#[cfg(test)]
mod tests;
