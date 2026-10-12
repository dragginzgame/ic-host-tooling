//! Admitted extractor execution with source identity and bounded output evidence.
//! Callers own grammar validation, publication and exclusion of concurrent writers.

use super::text::{NormalizationError, normalize};
use ic_host_artifacts::artifact::{ArtifactError, ArtifactIdentity};
use ic_host_fs::read::hash_file;
use ic_host_process::tool::{
    AdmittedTool, CompletedExecution, ExecutionContext, OutputLimits, ToolError,
};
use std::{fmt, path::Path};

#[cfg(test)]
mod tests;

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
        evidence: Box<CompletedExecution>,
    },
    /// Source bytes changed between preflight and post-extraction observation.
    SourceChanged {
        /// Identity observed before invocation.
        before: ArtifactIdentity,
        /// Identity observed after invocation.
        after: ArtifactIdentity,
        /// Bounded process output that must not be published as valid extraction.
        evidence: Box<CompletedExecution>,
    },
    /// Successful output could not be normalized under the selected bound.
    Normalize {
        /// Normalization failure.
        source: NormalizationError,
        /// Exact bounded process output, including invalid UTF-8 when present.
        evidence: Box<CompletedExecution>,
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
    pub evidence: CompletedExecution,
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
        .and_then(CompletedExecution::require_success)
        .and_then(CompletedExecution::require_complete)
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
