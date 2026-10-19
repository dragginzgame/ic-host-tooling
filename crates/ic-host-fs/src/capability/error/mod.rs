//! Context preserved for consumer-owned error projection.

use std::{io, path::PathBuf};

/// Filesystem operation whose original native failure is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Open a selected directory or file.
    Open,
    /// Observe a held descriptor or directory entry.
    Metadata,
    /// Create one directory component.
    CreateDirectory,
    /// Exclusively create a file.
    CreateFile,
    /// Read directory entries.
    ReadDirectory,
    /// Unlink the selected file name.
    RemoveFile,
    /// Synchronize a held directory.
    SyncDirectory,
}

/// Structural refusal, independent of consumer schemas and public diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// The path does not have the required normal components.
    InvalidPath,
    /// An observed component is a symbolic link.
    SymbolicLink,
    /// An observed directory component has another type.
    NotDirectory,
    /// An observed file component is not a regular file.
    NotRegularFile,
}

/// Capability selection and operation failures with their relevant paths.
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    /// A native failure, without discarding its original cause.
    #[error("{operation:?} failed at {path:?}: {source}")]
    Io {
        /// Failed operation.
        operation: Operation,
        /// Caller-facing path associated with the operation.
        path: PathBuf,
        /// Original native cause.
        #[source]
        source: io::Error,
    },
    /// A path or observed entry was refused.
    #[error("{reason:?} at {path:?}")]
    Refused {
        /// Refused path.
        path: PathBuf,
        /// Structural reason.
        reason: Refusal,
    },
    /// Directory permission bits violate the caller's selected mask.
    #[error("permissions {actual:o} at {path:?} include forbidden bits {forbidden:o}")]
    Permissions {
        /// Admitted directory's path.
        path: PathBuf,
        /// Observed low nine permission bits.
        actual: u32,
        /// Caller-selected forbidden bits.
        forbidden: u32,
    },
    /// Another directory entry existed beyond the caller's allowance.
    #[error("directory entries at {path:?} exceed {maximum}")]
    EntryLimit {
        /// Directory being inspected.
        path: PathBuf,
        /// Caller-selected entry ceiling.
        maximum: usize,
    },
}

/// Unlink failure distinguishes a completed unlink from failed synchronization.
#[derive(Debug, thiserror::Error)]
#[error("file removal failed (removed={removed}): {source}")]
pub struct RemovalError {
    /// Whether unlink completed before the error.
    pub removed: bool,
    /// Original removal or parent-sync failure.
    #[source]
    pub source: CapabilityError,
}
