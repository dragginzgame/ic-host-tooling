//! Pure directory identities from exact relative paths and caller-admitted digests.
//!
//! Extracted from IC Backup's maintained directory checksum implementation.
//! This module performs no IO and proves no tree completeness, byte custody or durability.

use super::{Digest as _, Sha256, Sha256Digest};
use std::{
    fmt,
    path::{Component, PathBuf},
};

/// Compose a directory checksum from sorted relative-path/file-digest pairs.
///
/// Paths must be exact UTF-8, slash-separated normal components, without NUL,
/// empty components, aliases or duplicates. Newlines and Unix backslashes retain
/// their literal meaning. Sorting uses `PathBuf` component ordering, not complete
/// text-byte ordering. Hash each path, NUL, lowercase SHA-256 hex and newline;
/// an empty set hashes empty bytes. Inputs are declarations, not authenticated bytes.
/// Callers own file-count/memory limits and native path semantics.
///
/// # Errors
/// Rejects non-UTF-8, malformed and duplicate relative identities.
pub fn checksum_relative_files(
    mut files: Vec<(PathBuf, Sha256Digest)>,
) -> Result<Sha256Digest, DirectoryChecksumError> {
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    let mut previous = None;
    for (relative, checksum) in &files {
        let name = relative
            .to_str()
            .ok_or_else(|| DirectoryChecksumError::NonUtf8Path {
                path: relative.clone(),
            })?;
        if name.contains('\0')
            || name.split('/').any(|part| matches!(part, "" | "." | ".."))
            || !relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(DirectoryChecksumError::InvalidRelativePath {
                path: relative.clone(),
            });
        }
        if previous == Some(relative) {
            return Err(DirectoryChecksumError::DuplicatePath {
                path: relative.clone(),
            });
        }
        previous = Some(relative);
        hasher.update(name.as_bytes());
        hasher.update([0]);
        hasher.update(checksum.hex_bytes());
        hasher.update(b"\n");
    }
    Ok(Sha256Digest::from_bytes(hasher.finalize().into()))
}

/// Invalid declared file identity; no filesystem operation has occurred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectoryChecksumError {
    /// The exact identity cannot be represented in UTF-8.
    NonUtf8Path {
        /// Rejected path, without lossy conversion.
        path: PathBuf,
    },
    /// The path is absolute, malformed or contains normalization aliases.
    InvalidRelativePath {
        /// Rejected path, without normalization.
        path: PathBuf,
    },
    /// A canonical relative identity occurs twice, even with equal digests.
    DuplicatePath {
        /// Repeated path.
        path: PathBuf,
    },
}

impl fmt::Display for DirectoryChecksumError {
    #[expect(
        clippy::unnecessary_debug_formatting,
        reason = "retain escaped exact path identities, including control characters and non-UTF-8 bytes"
    )]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUtf8Path { path } => {
                write!(formatter, "directory checksum path is not UTF-8: {path:?}")
            }
            Self::InvalidRelativePath { path } => write!(
                formatter,
                "directory checksum path is not canonical and relative: {path:?}"
            ),
            Self::DuplicatePath { path } => {
                write!(formatter, "duplicate directory checksum path: {path:?}")
            }
        }
    }
}
impl std::error::Error for DirectoryChecksumError {}

#[cfg(test)]
mod tests;
