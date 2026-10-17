//! Directory hashing over the existing no-follow descriptor walker.
//!
//! Extracted from IC Backup's directory checksum visitor; product staging,
//! publication, fixed layouts and recovery remain with Backup.

use super::{TreeVisitor, walk_tree};
use crate::read::{OpenEntryError, OpenedEntry, open_directory_no_follow, open_entry_beneath};
use ic_host_artifacts::artifact::{
    ArtifactError, Sha256Digest, hash_reader,
    tree::{DirectoryChecksumError, checksum_relative_files},
};
use rustix::{
    fd::{AsFd, OwnedFd},
    fs::FileType,
};
use std::{
    fs::File,
    io,
    path::{Component, Path, PathBuf},
};

/// Hash a directory selected by an exact normal-component path without following links.
///
/// Absolute paths open from `/`, relative paths from `.`. Empty, root-only,
/// non-UTF-8 and non-normal components reject. Every child uses the existing
/// no-follow/nonblocking descriptor walker. Each file stream is bounded by
/// `max_file_bytes`; callers own total bytes, entry count, depth, memory and time.
/// No files are written or synced. Hashes do not freeze bytes or path identities;
/// callers must exclude concurrent content/namespace mutation when needed.
///
/// # Errors
/// Rejects unsafe paths/entries, over-limit streams and IO failures.
pub fn checksum_directory(
    path: &Path,
    max_file_bytes: u64,
) -> Result<Sha256Digest, TreeChecksumError> {
    let (root, relative) = if path.is_absolute() {
        (
            Path::new("/"),
            path.strip_prefix("/").map_err(io::Error::other)?,
        )
    } else {
        (Path::new("."), path)
    };
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|part| matches!(part, Component::Normal(name) if name.to_str().is_some()))
    {
        return Err(TreeChecksumError::InvalidPath {
            path: path.to_path_buf(),
        });
    }
    let root = open_directory_no_follow(root)?;
    let directory = match open_entry_beneath(root.as_fd(), relative) {
        Ok(OpenedEntry::Directory(file)) => OwnedFd::from(file),
        Ok(OpenedEntry::RegularFile(_)) => {
            return Err(TreeChecksumError::UnsupportedEntry {
                path: path.to_path_buf(),
                kind: FileType::RegularFile,
            });
        }
        Err(OpenEntryError::UnsupportedEntry { kind }) => {
            return Err(TreeChecksumError::UnsupportedEntry {
                path: path.to_path_buf(),
                kind,
            });
        }
        Err(OpenEntryError::Io(error)) => return Err(error.into()),
    };
    checksum_directory_at(&directory, path, max_file_bytes)
}

/// Hash beneath a caller-held root using exact sorted path/NUL/digest/newline framing.
///
/// The root descriptor selects an inode; `display_root` is diagnostic only. Root
/// provenance, namespace/content exclusion, total tree budgets and future custody
/// remain caller-owned. Per-file stream limits apply independently. Files are
/// freshly opened by the walker; this operation neither synchronizes nor publishes.
///
/// # Errors
/// Rejects unsupported/non-UTF-8 children, file limits and descriptor/stream errors.
pub fn checksum_directory_at(
    directory: &OwnedFd,
    display_root: &Path,
    max_file_bytes: u64,
) -> Result<Sha256Digest, TreeChecksumError> {
    let mut visitor = ChecksumVisitor {
        checksums: Vec::new(),
        max_file_bytes,
    };
    walk_tree(directory, Path::new(""), display_root, &mut visitor)?;
    checksum_relative_files(visitor.checksums).map_err(TreeChecksumError::Declaration)
}

struct ChecksumVisitor {
    checksums: Vec<(PathBuf, Sha256Digest)>,
    max_file_bytes: u64,
}
impl TreeVisitor for ChecksumVisitor {
    type Error = TreeChecksumError;
    fn file(&mut self, fd: &OwnedFd, relative: &Path, _display: &Path) -> Result<(), Self::Error> {
        let identity = hash_reader(File::from(fd.try_clone()?), self.max_file_bytes)
            .map_err(TreeChecksumError::Input)?;
        self.checksums
            .push((relative.to_path_buf(), identity.sha256));
        Ok(())
    }
    fn enter_directory(&mut self, _relative: &Path, _display: &Path) -> Result<(), Self::Error> {
        Ok(())
    }
    fn leave_directory(
        &mut self,
        _fd: &OwnedFd,
        _relative: &Path,
        _display: &Path,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn unsupported(&self, display: &Path, kind: FileType) -> Self::Error {
        TreeChecksumError::UnsupportedEntry {
            path: display.to_path_buf(),
            kind,
        }
    }
    fn non_utf8(&self, display: &Path) -> Self::Error {
        TreeChecksumError::NonUtf8Path {
            path: display.to_path_buf(),
        }
    }
}

/// Directory-path, descriptor, bounded-stream or declared-identity failure.
#[derive(Debug)]
pub enum TreeChecksumError {
    /// A filesystem observation or descriptor operation failed.
    Io(io::Error),
    /// A regular-file stream failed or exceeded its caller-selected bound.
    Input(ArtifactError),
    /// Declared file identities cannot produce the canonical checksum.
    Declaration(DirectoryChecksumError),
    /// A selected path cannot be opened through normal UTF-8 components.
    InvalidPath {
        /// Exact rejected path.
        path: PathBuf,
    },
    /// A child name is not UTF-8.
    NonUtf8Path {
        /// Exact rejected path.
        path: PathBuf,
    },
    /// A tree entry is not a regular file or directory.
    UnsupportedEntry {
        /// Rejected entry.
        path: PathBuf,
        /// Observed filesystem entry type.
        kind: FileType,
    },
}

impl std::fmt::Display for TreeChecksumError {
    #[expect(
        clippy::unnecessary_debug_formatting,
        reason = "retain escaped exact path identities, including control characters and non-UTF-8 bytes"
    )]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Input(error) => error.fmt(formatter),
            Self::Declaration(error) => error.fmt(formatter),
            Self::InvalidPath { path } => {
                write!(formatter, "invalid directory checksum path: {path:?}")
            }
            Self::NonUtf8Path { path } => {
                write!(formatter, "directory checksum path is not UTF-8: {path:?}")
            }
            Self::UnsupportedEntry { path, kind } => write!(
                formatter,
                "unsupported directory checksum entry at {path:?}: {kind:?}"
            ),
        }
    }
}
impl std::error::Error for TreeChecksumError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Input(error) => Some(error),
            Self::Declaration(error) => Some(error),
            Self::InvalidPath { .. } | Self::NonUtf8Path { .. } | Self::UnsupportedEntry { .. } => {
                None
            }
        }
    }
}
impl From<io::Error> for TreeChecksumError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(test)]
mod tests;
