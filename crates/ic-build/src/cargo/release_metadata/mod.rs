//! Pure, format-preserving Cargo release candidates.
//!
//! Responsibility: edit explicitly selected version literals and verify lock projections.
//! Does not own: release policy, source admission, filesystem custody or publication.
//! Boundary: candidates are returned only after complete parsing and validation.

use super::lock_projection::{
    LockProjectionError, WorkspaceVersionProjection, verify_workspace_release_projection,
};
use std::{collections::HashSet, fmt, ops::Range};
use toml_edit::{Document, Item, Value};

/// One explicitly admitted workspace dependency alias and its exact requirements.
#[derive(Clone, Copy, Debug)]
pub struct DependencyVersionProjection<'a> {
    /// Key beneath `workspace.dependencies`, including renamed dependencies.
    pub alias: &'a str,
    /// Exact requirement in the admitted input.
    pub previous: &'a str,
    /// Caller-selected replacement requirement.
    pub next: &'a str,
}

/// A candidate could not be rendered; no caller input is changed.
#[derive(Debug)]
pub enum ReleaseMetadataError {
    /// Missing, ambiguous or conflicting selected literals.
    InvalidSelection(&'static str),
    /// Complete TOML parsing failed.
    Parse(toml_edit::TomlError),
    /// The independent semantic lock projection refused the candidate.
    Projection(LockProjectionError),
}
impl fmt::Display for ReleaseMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSelection(reason) => f.write_str(reason),
            Self::Parse(error) => error.fmt(f),
            Self::Projection(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ReleaseMetadataError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidSelection(_) => None,
            Self::Parse(error) => Some(error),
            Self::Projection(error) => Some(error),
        }
    }
}

const fn refuse(reason: &'static str) -> ReleaseMetadataError {
    ReleaseMetadataError::InvalidSelection(reason)
}
fn stable(version: &str) -> bool {
    let mut count = 0;
    version.split('.').all(|part| {
        count += 1;
        !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && (part == "0" || !part.starts_with('0'))
    }) && count == 3
}
fn edit(
    value: &Value,
    previous: &str,
    next: &str,
) -> Result<(Range<usize>, String), ReleaseMetadataError> {
    if value.as_str() != Some(previous) {
        return Err(refuse("selected version differs from the admitted input"));
    }
    let span = value
        .span()
        .ok_or_else(|| refuse("selected literal has no source span"))?;
    Ok((span, Value::from(next).to_string()))
}
fn apply(
    input: &str,
    mut edits: Vec<(Range<usize>, String)>,
) -> Result<String, ReleaseMetadataError> {
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0.start));
    let mut output = input.to_owned();
    let mut boundary = input.len();
    for (span, replacement) in edits {
        if span.end > boundary || input.get(span.clone()).is_none() {
            return Err(refuse("overlapping or invalid selected literal spans"));
        }
        boundary = span.start;
        output.replace_range(span, &replacement);
    }
    Document::parse(output.as_str()).map_err(ReleaseMetadataError::Parse)?;
    Ok(output)
}

/// Render a workspace version and explicitly admitted local dependency requirements.
///
/// TOML owns syntax and spans. Every other byte (including comments, line endings,
/// Unicode and external requirements) is retained. The caller establishes dependency
/// ownership and selects replacement requirements; no filesystem or process runs.
///
/// # Errors
/// Refuses invalid stable versions, malformed TOML, missing/duplicate aliases,
/// conflicting prior literals or unrepresentable edits. No partial candidate escapes.
pub fn render_workspace_manifest(
    input: &str,
    previous: &str,
    next: &str,
    dependencies: &[DependencyVersionProjection<'_>],
) -> Result<String, ReleaseMetadataError> {
    if !stable(previous) || !stable(next) {
        return Err(refuse("expected canonical stable workspace versions"));
    }
    let document = Document::parse(input).map_err(ReleaseMetadataError::Parse)?;
    let workspace = document
        .as_item()
        .get("workspace")
        .ok_or_else(|| refuse("missing workspace"))?;
    let version = workspace
        .get("package")
        .and_then(|item| item.get("version"))
        .and_then(Item::as_value)
        .ok_or_else(|| refuse("missing workspace.package.version"))?;
    let mut edits = vec![edit(version, previous, next)?];
    let mut seen = HashSet::new();
    for dependency in dependencies {
        if dependency.alias.is_empty()
            || !seen.insert(dependency.alias)
            || dependency.previous.is_empty()
            || dependency.next.is_empty()
        {
            return Err(refuse("invalid or duplicate dependency selection"));
        }
        let version = workspace
            .get("dependencies")
            .and_then(|item| item.get(dependency.alias))
            .and_then(|item| item.get("version"))
            .and_then(Item::as_value)
            .ok_or_else(|| refuse("missing selected dependency version"))?;
        edits.push(edit(version, dependency.previous, dependency.next)?);
    }
    apply(input, edits)
}

/// Render exact unsourced package versions and unqualified local dependency references.
///
/// Source-qualified registry/Git references, checksums and all other bytes are
/// preserved. The separate semantic projection checker admits the complete result.
/// Callers retain input byte limits, saved originals, Cargo graph checks and rollback.
///
/// # Errors
/// Refuses invalid stable versions, empty/duplicate selections, unsupported lock
/// formats, malformed TOML, ambiguous/missing packages or conflicting prior values.
pub fn render_workspace_lock(
    input: &str,
    projections: &[WorkspaceVersionProjection<'_>],
) -> Result<String, ReleaseMetadataError> {
    if projections.is_empty()
        || projections
            .iter()
            .any(|item| !stable(item.previous) || !stable(item.next) || item.previous == item.next)
    {
        return Err(refuse("expected distinct canonical stable lock versions"));
    }
    // Establish package identities and prior values independently before rendering.
    // The verifier will report the expected ChangedInputs for a nontrivial valid selection.
    match verify_workspace_release_projection(input, input, projections) {
        Err(LockProjectionError::ChangedInputs) => {}
        Err(error) => return Err(ReleaseMetadataError::Projection(error)),
        Ok(()) => return Err(refuse("lock selection changes no versions")),
    }
    let document = Document::parse(input).map_err(ReleaseMetadataError::Parse)?;
    if !matches!(
        document.as_item().get("version").and_then(Item::as_integer),
        Some(3 | 4)
    ) {
        return Err(refuse("expected Cargo lock format 3 or 4"));
    }
    let packages = document
        .as_item()
        .get("package")
        .and_then(Item::as_array_of_tables)
        .ok_or_else(|| refuse("missing lock packages"))?;
    let mut edits = Vec::new();
    for package in packages {
        if package.get("source").is_none()
            && let Some(projection) = projections
                .iter()
                .find(|item| package.get("name").and_then(Item::as_str) == Some(item.package))
        {
            let value = package
                .get("version")
                .and_then(Item::as_value)
                .ok_or_else(|| refuse("missing local package version"))?;
            edits.push(edit(value, projection.previous, projection.next)?);
        }
        if let Some(dependencies) = package.get("dependencies").and_then(Item::as_array) {
            for dependency in dependencies {
                for projection in projections {
                    let previous = format!("{} {}", projection.package, projection.previous);
                    if dependency.as_str() == Some(previous.as_str()) {
                        edits.push(edit(
                            dependency,
                            &previous,
                            &format!("{} {}", projection.package, projection.next),
                        )?);
                    }
                }
            }
        }
    }
    let output = apply(input, edits)?;
    verify_workspace_release_projection(input, &output, projections)
        .map_err(ReleaseMetadataError::Projection)?;
    Ok(output)
}

#[cfg(test)]
mod tests;
