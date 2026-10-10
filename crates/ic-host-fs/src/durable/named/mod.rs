//! Named producer access to the existing durable publication engine.

use std::{fmt, io, path::Path};

/// Failure from a named or streamed producer and durable publication, retaining cleanup evidence.
#[derive(Debug)]
pub enum NamedWriteError<E> {
    /// The producer or its output validation failed; no publication was attempted.
    Producer {
        /// Original consumer error, including any bounded process evidence.
        source: E,
        /// Failure to remove the owned staging entry; replacements are left untouched.
        cleanup_error: Option<io::Error>,
    },
    /// Filesystem admission, synchronization or rename failed before publication.
    BeforePublication {
        /// Original filesystem error or an invalid-data identity rejection.
        source: io::Error,
        /// Failure to remove the owned staging entry; replacements are left untouched.
        cleanup_error: Option<io::Error>,
    },
    /// Publication succeeded, but staging cleanup or final directory sync failed.
    AfterPublication {
        /// Original cleanup or synchronization error. The new output is visible;
        /// final directory durability is not established. Reconcile before retrying.
        source: io::Error,
    },
}

impl<E> NamedWriteError<E> {
    pub(super) const fn before(source: io::Error) -> Self {
        Self::BeforePublication {
            source,
            cleanup_error: None,
        }
    }
}

impl<E: fmt::Display> fmt::Display for NamedWriteError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Producer { source, .. } => write!(formatter, "staged producer failed: {source}"),
            Self::BeforePublication { source, .. } => {
                write!(formatter, "publication failed before rename: {source}")
            }
            Self::AfterPublication { source } => write!(
                formatter,
                "output published but completion failed: {source}"
            ),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for NamedWriteError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Producer { source, .. } => Some(source),
            Self::BeforePublication { source, .. } | Self::AfterPublication { source } => {
                Some(source)
            }
        }
    }
}

/// Produce and validate output at an owned absolute sibling path, then durably replace a file.
///
/// Uses the same creation, synchronization and publication engine as [`super::write_with`].
/// The callback receives an exclusively created, empty regular file. An external
/// tool must write/truncate that inode, not unlink it or replace it by rename.
/// The callback owns executable admission, arguments, environment, output byte
/// limits and content validation. A precreated empty file is not proof of output:
/// validate it before returning `Ok`, even when the process exits successfully.
/// Its value is returned only after complete durable publication.
///
/// The caller must control the parent hierarchy and exclude concurrent namespace
/// mutation throughout the call. Identity checks reject observed staging/parent
/// replacement, symlinks, special files and additional hard links; they cannot
/// make pathname comparison and rename atomic against a hostile directory writer.
/// All producers and writable handles must finish before `Ok`. This is neither a
/// child sandbox nor a process-tree supervisor. No retries occur.
///
/// Ordinary failures attempt to unlink only the still-owned staging entry and
/// retain any cleanup error. Foreign replacements and files moved elsewhere are
/// never removed. Panics, interruption and uncooperative surviving children can
/// leave staging or other effects; caller-owned reconciliation remains necessary.
///
/// # Errors
/// Returns the original producer error or a filesystem failure with explicit
/// before/after-publication state. After-publication failures must be reconciled
/// before retrying. Unsupported hosts return a before-publication error.
pub fn write_named_with<T, E>(
    path: &Path,
    produce: impl FnOnce(&Path) -> Result<T, E>,
) -> Result<T, NamedWriteError<E>> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        super::supported::validate_target_path(path).map_err(NamedWriteError::before)?;
        let absolute = std::path::absolute(path).map_err(NamedWriteError::before)?;
        super::supported::commit_path_with_options(
            &absolute,
            super::REPLACE_OPTIONS,
            |stage, _| produce(stage),
            None::<fn(&Path, &T) -> Result<(), E>>,
            |_, _| Ok(()),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (path, produce);
        Err(NamedWriteError::before(io::Error::new(
            io::ErrorKind::Unsupported,
            "durable atomic file publication is unsupported",
        )))
    }
}

#[cfg(all(test, unix))]
mod tests;
