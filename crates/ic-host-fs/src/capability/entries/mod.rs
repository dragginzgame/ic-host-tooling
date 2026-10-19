//! Bounded observations without consumer candidate-selection policy.

use super::{CapabilityError, Operation, failure};
use std::{ffi::OsString, iter::FusedIterator, path::PathBuf};

/// An observed entry type; observation is not descriptor admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symbolic link, without following its target.
    SymbolicLink,
    /// Special or other unsupported entry.
    Other,
}

/// A native entry name and observed type, with no schema interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectoryEntry {
    /// Exact OS-native name.
    pub name: OsString,
    /// Type observed without following a symbolic link.
    pub kind: EntryKind,
}

/// Lazy directory observations with an explicit caller-selected entry ceiling.
///
/// Every entry consumes the allowance. Once exhausted, one lookahead establishes
/// whether another entry exists; its payload/errors are not admitted. A resulting
/// `EntryLimit` is reported once and fuses the iterator. Empty directories accept
/// zero. Callers own filtering, depth/aggregate budgets, ordering and presentation.
#[derive(Debug)]
pub struct DirectoryEntries {
    inner: cap_std::fs::ReadDir,
    path: PathBuf,
    maximum: usize,
    remaining: usize,
    done: bool,
}

impl DirectoryEntries {
    pub(super) const fn new(inner: cap_std::fs::ReadDir, path: PathBuf, maximum: usize) -> Self {
        Self {
            inner,
            path,
            maximum,
            remaining: maximum,
            done: false,
        }
    }
}

impl Iterator for DirectoryEntries {
    type Item = Result<DirectoryEntry, CapabilityError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let Some(entry) = self.inner.next() else {
            self.done = true;
            return None;
        };
        if self.remaining == 0 {
            self.done = true;
            return Some(Err(CapabilityError::EntryLimit {
                path: self.path.clone(),
                maximum: self.maximum,
            }));
        }
        self.remaining -= 1;
        Some(
            entry
                .map_err(|error| failure(Operation::ReadDirectory, &self.path, error))
                .and_then(|entry| {
                    let name = entry.file_name();
                    let kind = entry.file_type().map_err(|error| {
                        failure(Operation::Metadata, &self.path.join(&name), error)
                    })?;
                    let kind = if kind.is_symlink() {
                        EntryKind::SymbolicLink
                    } else if kind.is_dir() {
                        EntryKind::Directory
                    } else if kind.is_file() {
                        EntryKind::File
                    } else {
                        EntryKind::Other
                    };
                    Ok(DirectoryEntry { name, kind })
                }),
        )
    }
}

impl FusedIterator for DirectoryEntries {}
