//! Bounded Git observations through a consumer-admitted executable.
//!
//! Queries run separately, once each; they are not an atomic source snapshot.
//! Consumers own executable pins, environment, worktree selection, concurrency
//! exclusion and report schemas. No Git mutation or installation is performed.

#[cfg(test)]
mod tests;

use crate::tool::{AdmittedTool, CompletedExecution, ExecutionContext, OutputLimits, ToolError};
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
    pub completed: Box<[Option<CompletedExecution>; 3]>,
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
    /// Selected status scope, retained alongside the dirty observation.
    pub options: StatusOptions,
    /// Raw status byte count and SHA-256; not a canonical source-tree identity.
    pub status_identity: ArtifactIdentity,
    /// Admitted Git executable identity.
    pub tool_identity: ArtifactIdentity,
    revision_evidence: CompletedExecution,
    tree_evidence: CompletedExecution,
    status_evidence: CompletedExecution,
}

impl GitObservations {
    /// Borrow the validated lowercase HEAD object ID, without its final newline.
    #[must_use]
    pub fn revision(&self) -> &str {
        object_id(&self.revision_evidence.stdout)
    }

    /// Borrow the validated lowercase HEAD tree ID, without its final newline.
    #[must_use]
    pub fn tree(&self) -> &str {
        object_id(&self.tree_evidence.stdout)
    }

    /// Original bounded revision output and diagnostics, retained without copying.
    #[must_use]
    pub const fn revision_evidence(&self) -> &CompletedExecution {
        &self.revision_evidence
    }

    /// Original bounded tree output and diagnostics, retained without copying.
    #[must_use]
    pub const fn tree_evidence(&self) -> &CompletedExecution {
        &self.tree_evidence
    }

    /// Original bounded status output, including arbitrary pathname bytes.
    #[must_use]
    pub const fn status_evidence(&self) -> &CompletedExecution {
        &self.status_evidence
    }

    /// Whether status reported bytes under the selected scope. Ignored files,
    /// excluded untracked files and concurrent changes are not ruled out.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        !self.status_evidence.stdout.is_empty()
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
    let mut completed = [None, None, None];
    for (index, query) in [GitQuery::Revision, GitQuery::Tree, GitQuery::Status]
        .into_iter()
        .enumerate()
    {
        let evidence = match git
            .run(&arguments(query, options), context, limits)
            .and_then(CompletedExecution::require_success)
            .and_then(CompletedExecution::require_complete)
        {
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
            GitQuery::Status
                if !evidence.stdout.is_empty() && evidence.stdout.last() != Some(&0) =>
            {
                Some(GitFailure::UnterminatedStatus)
            }
            _ => None,
        };
        completed[index] = Some(evidence);
        if let Some(failure) = failure {
            return Err(GitError {
                query,
                failure,
                completed: Box::new(completed),
            });
        }
    }
    let [Some(revision), Some(tree), Some(status)] = completed else {
        unreachable!("every ordered query completed successfully");
    };
    let status_identity = ArtifactIdentity {
        bytes: status.stdout.len() as u64,
        sha256: Sha256Digest::compute(&status.stdout),
    };
    Ok(GitObservations {
        options,
        status_identity,
        tool_identity: git.identity(),
        revision_evidence: revision,
        tree_evidence: tree,
        status_evidence: status,
    })
}

fn valid_object_id(bytes: &[u8]) -> bool {
    matches!(bytes.len(), 41 | 65)
        && bytes.last() == Some(&b'\n')
        && bytes[..bytes.len() - 1]
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn object_id(bytes: &[u8]) -> &str {
    // Only capture_git constructs observations; their admitted captures are private.
    std::str::from_utf8(&bytes[..bytes.len() - 1])
        .expect("captured object ID was validated as ASCII plus LF")
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
