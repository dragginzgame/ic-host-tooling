//! Bounded Git observations through a consumer-admitted executable.
//!
//! Queries run separately, once each; they are not an atomic source snapshot.
//! Consumers own executable pins, environment, worktree selection, concurrency
//! exclusion and report schemas. No Git mutation or installation is performed.

#[cfg(test)]
mod tests;

use crate::tool::{AdmittedTool, ExecutionContext, ExecutionEvidence, OutputLimits, ToolError};
use ic_host_artifacts::artifact::{ArtifactIdentity, Sha256Digest};
use std::{ffi::OsString, fmt};

/// Explicit untracked-file policy for the status observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UntrackedFiles {
    /// Exclude untracked files.
    No,
    /// Report untracked directories without enumerating their contents.
    Normal,
    /// Report individual untracked files.
    All,
}

/// Explicit submodule policy for the status observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IgnoreSubmodules {
    /// Observe all submodule changes, overriding configured exclusions.
    None,
    /// Ignore untracked submodule contents.
    Untracked,
    /// Ignore submodule working-tree changes, retaining commit differences.
    Dirty,
    /// Ignore all submodule changes.
    All,
}

/// Consumer-selected status scope; there is deliberately no default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatusOptions {
    /// Untracked-file scope.
    pub untracked: UntrackedFiles,
    /// Submodule scope.
    pub ignore_submodules: IgnoreSubmodules,
}

/// One of the ordered Git observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitQuery {
    /// `rev-parse --verify HEAD`.
    Revision,
    /// `rev-parse --verify HEAD^{tree}`.
    Tree,
    /// NUL-terminated porcelain-v1 status.
    Status,
}

/// Typed provenance failure independent of retained output.
#[derive(Debug)]
pub enum GitFailure {
    /// Admission recheck or bounded execution failed; current output is here.
    Tool(Box<ToolError>),
    /// Object stdout was not one lowercase 40- or 64-digit hex ID plus LF.
    InvalidObjectId,
    /// Nonempty status stdout was not terminated by NUL.
    UnterminatedStatus,
}

/// Failed query with all successfully captured output, including malformed output.
#[derive(Debug)]
pub struct GitError {
    /// Query that failed; later queries were not attempted.
    pub query: GitQuery,
    /// Failure category; tool failures retain their own current evidence.
    pub failure: GitFailure,
    /// Revision, tree and status captures in order. `None` means no successful
    /// process capture for that query. Malformed successful output is retained.
    pub completed: Box<[Option<ExecutionEvidence>; 3]>,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Git {:?} observation failed", self.query)
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            GitFailure::Tool(source) => Some(source.as_ref()),
            GitFailure::InvalidObjectId | GitFailure::UnterminatedStatus => None,
        }
    }
}

/// Raw observations, without a product report or source-cleanliness guarantee.
#[derive(Debug)]
pub struct GitObservations {
    /// Lowercase object ID returned for HEAD.
    pub revision: String,
    /// Lowercase object ID returned for the HEAD tree.
    pub tree: String,
    /// Selected status scope, retained alongside the dirty observation.
    pub options: StatusOptions,
    /// Raw status byte count and SHA-256; not a canonical source-tree identity.
    pub status_identity: ArtifactIdentity,
    /// Admitted Git executable identity.
    pub tool_identity: ArtifactIdentity,
    /// Original bounded revision output and diagnostics.
    pub revision_evidence: ExecutionEvidence,
    /// Original bounded tree output and diagnostics.
    pub tree_evidence: ExecutionEvidence,
    /// Original bounded status output, including arbitrary pathname bytes.
    pub status_evidence: ExecutionEvidence,
}

impl GitObservations {
    /// Whether status reported bytes under the selected scope. Ignored files,
    /// excluded untracked files and concurrent changes are not ruled out.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.status_identity.bytes != 0
    }
}

/// Observe HEAD, its tree and status, with per-query bounds and no retries.
///
/// Commands disable optional locks and the filesystem monitor. The complete
/// environment and working directory remain explicit; Git environment/config
/// can redirect repository selection, and callers own that admission policy.
/// Status is opaque porcelain-v1 `-z` output, checked only for NUL termination.
/// These separate observations cannot prove an atomic or reproducible build.
///
/// # Errors
/// Returns typed tool failures, malformed object IDs or unterminated status.
/// Retains earlier captures and the failed query's available evidence.
pub fn capture_git(
    git: &AdmittedTool,
    context: &ExecutionContext<'_>,
    options: StatusOptions,
    limits: OutputLimits,
) -> Result<GitObservations, GitError> {
    let revision = observe(git, context, options, limits, GitQuery::Revision)?;
    let tree = match observe(git, context, options, limits, GitQuery::Tree) {
        Ok(evidence) => evidence,
        Err(mut error) => {
            error.completed[0] = Some(revision);
            return Err(error);
        }
    };
    let status = match observe(git, context, options, limits, GitQuery::Status) {
        Ok(evidence) => evidence,
        Err(mut error) => {
            error.completed[0] = Some(revision);
            error.completed[1] = Some(tree);
            return Err(error);
        }
    };
    let status_identity = ArtifactIdentity {
        bytes: status.stdout.len() as u64,
        sha256: Sha256Digest::compute(&status.stdout),
    };
    // Object validation above admits only ASCII, so conversion is lossless.
    let revision_id = object_id(&revision.stdout);
    let tree_id = object_id(&tree.stdout);
    Ok(GitObservations {
        revision: revision_id,
        tree: tree_id,
        options,
        status_identity,
        tool_identity: git.identity(),
        revision_evidence: revision,
        tree_evidence: tree,
        status_evidence: status,
    })
}

fn observe(
    git: &AdmittedTool,
    context: &ExecutionContext<'_>,
    options: StatusOptions,
    limits: OutputLimits,
    query: GitQuery,
) -> Result<ExecutionEvidence, GitError> {
    let mut completed = [None, None, None];
    let index = match query {
        GitQuery::Revision => 0,
        GitQuery::Tree => 1,
        GitQuery::Status => 2,
    };
    let evidence = match git.run(&arguments(query, options), context, limits) {
        Ok(evidence) => evidence,
        Err(source) => {
            return Err(GitError {
                query,
                failure: GitFailure::Tool(Box::new(source)),
                completed: Box::new(completed),
            });
        }
    };
    let failure = match query {
        GitQuery::Revision | GitQuery::Tree if !valid_object_id(&evidence.stdout) => {
            Some(GitFailure::InvalidObjectId)
        }
        GitQuery::Status if !evidence.stdout.is_empty() && evidence.stdout.last() != Some(&0) => {
            Some(GitFailure::UnterminatedStatus)
        }
        _ => None,
    };
    if let Some(failure) = failure {
        completed[index] = Some(evidence);
        return Err(GitError {
            query,
            failure,
            completed: Box::new(completed),
        });
    }
    Ok(evidence)
}

fn valid_object_id(bytes: &[u8]) -> bool {
    matches!(bytes.len(), 41 | 65)
        && bytes.last() == Some(&b'\n')
        && bytes[..bytes.len() - 1]
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn object_id(bytes: &[u8]) -> String {
    bytes[..bytes.len() - 1]
        .iter()
        .map(|&byte| char::from(byte))
        .collect()
}

fn arguments(query: GitQuery, options: StatusOptions) -> Vec<OsString> {
    let mut arguments = vec![
        "--no-optional-locks".into(),
        "-c".into(),
        "core.fsmonitor=false".into(),
    ];
    match query {
        GitQuery::Revision => {
            arguments.extend(["rev-parse".into(), "--verify".into(), "HEAD".into()]);
        }
        GitQuery::Tree => {
            arguments.extend(["rev-parse".into(), "--verify".into(), "HEAD^{tree}".into()]);
        }
        GitQuery::Status => {
            let untracked = match options.untracked {
                UntrackedFiles::No => "no",
                UntrackedFiles::Normal => "normal",
                UntrackedFiles::All => "all",
            };
            let submodules = match options.ignore_submodules {
                IgnoreSubmodules::None => "none",
                IgnoreSubmodules::Untracked => "untracked",
                IgnoreSubmodules::Dirty => "dirty",
                IgnoreSubmodules::All => "all",
            };
            arguments.extend([
                "status".into(),
                "--porcelain=v1".into(),
                "-z".into(),
                format!("--untracked-files={untracked}").into(),
                format!("--ignore-submodules={submodules}").into(),
            ]);
        }
    }
    arguments
}
