//! Bounded Candid extractor invocation and Canic-compatible text normalization.
//!
//! Tool admission is owned by `tool`. Consumers own Candid grammar/service
//! validation, expected methods, metadata policy and artifact publication.
//! The library does not publish sidecars or clean up source files. The selected
//! extractor must be consumer-governed and read-only; observed tool-side source
//! changes are rejected, but the library does not sandbox or undo those effects.

use ic_host_artifacts::artifact::{ArtifactError, ArtifactIdentity};
use ic_host_fs::read::hash_file;
use ic_host_process::tool::{
    AdmittedTool, ExecutionContext, ExecutionEvidence, OutputLimits, ToolError,
};
use std::{fmt, path::Path};

#[cfg(test)]
mod tests;

/// UTF-8 or resource failure while normalizing captured extractor output.
#[derive(Debug)]
pub enum NormalizationError {
    /// Input bytes exceed the caller's bound before decoding.
    InputLimit {
        /// Observed byte count.
        actual: usize,
        /// Maximum permitted bytes.
        limit: usize,
    },
    /// Captured output is not valid UTF-8; no lossy decoding is performed.
    Utf8(std::str::Utf8Error),
    /// Normalized text, including added line endings, would exceed the bound.
    OutputLimit {
        /// Maximum permitted normalized bytes.
        limit: usize,
    },
    /// Storage for normalized text could not be allocated.
    Allocation(std::collections::TryReserveError),
}

impl fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLimit { actual, limit } => {
                write!(f, "Candid output has {actual} bytes, exceeding {limit}")
            }
            Self::Utf8(_) => f.write_str("Candid extractor output is not UTF-8"),
            Self::OutputLimit { limit } => write!(f, "normalized Candid exceeds {limit} bytes"),
            Self::Allocation(_) => f.write_str("Candid normalization allocation failed"),
        }
    }
}
impl std::error::Error for NormalizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Utf8(source) => Some(source),
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

/// Normalize UTF-8 extractor bytes with an explicit input/output byte bound.
///
/// Matches Canic's `str::lines` / `trim_end` contract: remove each line's
/// trailing Unicode whitespace, emit LF endings, and preserve blank lines.
/// Empty input stays empty; a nonempty unterminated line gains one LF.
/// This does not parse Candid or select an expected service contract.
///
/// # Errors
/// Returns input overflow, invalid UTF-8, normalized-output overflow or
/// allocation failure. No partial normalized text is returned.
pub fn normalize(bytes: &[u8], max_bytes: usize) -> Result<String, NormalizationError> {
    if bytes.len() > max_bytes {
        return Err(NormalizationError::InputLimit {
            actual: bytes.len(),
            limit: max_bytes,
        });
    }
    let text = std::str::from_utf8(bytes).map_err(NormalizationError::Utf8)?;
    let mut length = 0_usize;
    for line in text.lines() {
        length = length
            .checked_add(line.trim_end().len())
            .and_then(|length| length.checked_add(1))
            .filter(|&length| length <= max_bytes)
            .ok_or(NormalizationError::OutputLimit { limit: max_bytes })?;
    }
    let mut normalized = String::new();
    normalized
        .try_reserve_exact(length)
        .map_err(NormalizationError::Allocation)?;
    for line in text.lines() {
        normalized.push_str(line.trim_end());
        normalized.push('\n');
    }
    Ok(normalized)
}

/// A Candid extraction failed, retaining process bytes where invocation occurred.
#[derive(Debug)]
pub enum ExtractionError {
    /// The source path was relative; no tool was invoked.
    SourcePath,
    /// Source inspection failed before invocation.
    Input(ArtifactError),
    /// Tool verification or execution failed; execution failures retain evidence.
    Tool(ToolError),
    /// Source could not be re-inspected after successful execution.
    SourceInspection {
        /// Inspection failure.
        source: ArtifactError,
        /// Bounded successful process output.
        evidence: Box<ExecutionEvidence>,
    },
    /// Source bytes changed between preflight and post-extraction observation.
    SourceChanged {
        /// Identity observed before invocation.
        before: ArtifactIdentity,
        /// Identity observed after invocation.
        after: ArtifactIdentity,
        /// Bounded process output that must not be published as valid extraction.
        evidence: Box<ExecutionEvidence>,
    },
    /// Successful output could not be normalized under the selected bound.
    Normalize {
        /// Normalization failure.
        source: NormalizationError,
        /// Exact bounded process output, including invalid UTF-8 when present.
        evidence: Box<ExecutionEvidence>,
    },
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourcePath => f.write_str("Candid source path must be absolute"),
            Self::Input(source) => write!(f, "Candid source inspection failed: {source}"),
            Self::Tool(source) => source.fmt(f),
            Self::SourceInspection { .. } => f.write_str("Candid source re-inspection failed"),
            Self::SourceChanged { .. } => f.write_str("Candid source changed during extraction"),
            Self::Normalize { source, .. } => source.fmt(f),
        }
    }
}
impl std::error::Error for ExtractionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(source) | Self::SourceInspection { source, .. } => Some(source),
            Self::Tool(source) => Some(source),
            Self::Normalize { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Normalized text, source/tool identities, and exact bounded process evidence.
pub struct ExtractedCandid {
    /// Normalized UTF-8 declaration; grammar acceptance remains caller-owned.
    pub text: String,
    /// Matching source identity observed before and after invocation.
    pub source_identity: ArtifactIdentity,
    /// Tool bytes admitted by the consumer.
    pub tool_identity: ArtifactIdentity,
    /// Original stdout/stderr and successful direct-child exit status.
    pub evidence: ExecutionEvidence,
}

impl fmt::Debug for ExtractedCandid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtractedCandid")
            .field("text_bytes", &self.text.len())
            .field("source_identity", &self.source_identity)
            .field("tool_identity", &self.tool_identity)
            .field("evidence", &self.evidence)
            .finish()
    }
}

/// Invoke the admitted extractor once with the absolute source path as one arg.
///
/// Input regular-file hashing is bounded by `source_bytes`. Normalized text is
/// bounded by `output.stdout.retained_bytes()`, independently of capture storage. The
/// source is inspected again on successful completion; observed changes reject
/// publication. Callers must exclude concurrent writers to source/tool paths:
/// before/after digests cannot prove that transient changes did not occur.
///
/// # Errors
/// Returns typed path/read, tool-execution, source-change or normalization
/// failures. After invocation, process failure/validation evidence is retained.
pub fn extract(
    tool: &AdmittedTool,
    source: &Path,
    context: &ExecutionContext<'_>,
    source_bytes: u64,
    output: OutputLimits,
) -> Result<ExtractedCandid, ExtractionError> {
    if !source.is_absolute() {
        return Err(ExtractionError::SourcePath);
    }
    let before = hash_file(source, source_bytes).map_err(ExtractionError::Input)?;
    let evidence = tool
        .run(&[source.as_os_str().to_owned()], context, output)
        .and_then(ExecutionEvidence::require_complete)
        .map_err(ExtractionError::Tool)?;
    let after = match hash_file(source, source_bytes) {
        Ok(identity) => identity,
        Err(source) => {
            return Err(ExtractionError::SourceInspection {
                source,
                evidence: Box::new(evidence),
            });
        }
    };
    if before != after {
        return Err(ExtractionError::SourceChanged {
            before,
            after,
            evidence: Box::new(evidence),
        });
    }
    let text = match normalize(&evidence.stdout, output.stdout.retained_bytes()) {
        Ok(text) => text,
        Err(source) => {
            return Err(ExtractionError::Normalize {
                source,
                evidence: Box::new(evidence),
            });
        }
    };
    Ok(ExtractedCandid {
        text,
        source_identity: before,
        tool_identity: tool.identity(),
        evidence,
    })
}
