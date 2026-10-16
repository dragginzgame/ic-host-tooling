//! Explicit captured-evidence rendering; ordinary error display stays unchanged.

#[cfg(test)]
mod tests;

use crate::tool::ToolError;
use std::fmt::Write as _;

/// Format bounded process output and cleanup evidence with caller-selected context.
///
/// Shared error display omits captured bytes; callers use this projection when
/// their diagnostic contract needs the original status, streams and cleanup errors.
/// Stream bytes use lossy UTF-8 with trailing whitespace trimmed. This is a raw
/// diagnostic projection, not terminal escaping, redaction or output admission.
#[must_use]
pub fn format_failure_evidence(context: &str, error: &ToolError) -> String {
    let mut message = format!("{context}: {error}");
    if let Some(evidence) = error.evidence() {
        let _ = write!(
            message,
            "\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
            evidence.status,
            String::from_utf8_lossy(&evidence.stdout).trim_end(),
            String::from_utf8_lossy(&evidence.stderr).trim_end(),
        );
    }
    if let Some(cleanup) = error
        .execution_error()
        .and_then(|failure| failure.cleanup.as_ref())
    {
        let _ = write!(message, "\ncleanup: {cleanup}");
    }
    message
}
