//! Synchronize and checksum a caller-held directory before caller-owned publication.
//!
//! Extracted from IC Backup's ordered descriptor sync visitor. Product records,
//! expected-checksum admission, publication policy and recovery remain caller-owned.

use crate::read::tree::{
    TreeVisitor,
    checksum::{ChecksumVisitor, TreeChecksumError},
    walk_tree,
};
use ic_host_artifacts::artifact::Sha256Digest;
use rustix::{fd::OwnedFd, fs::FileType};
use std::{
    fs::File,
    io::{self, Seek, SeekFrom},
    os::fd::AsFd,
    path::Path,
};

/// Ordered pre-sync observations; an error stops before the selected sync.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeSyncStep {
    /// A regular file is about to be synchronized, then read for its digest.
    RegularFile,
    /// All observed children of a nested directory were processed before its sync.
    NestedDirectory,
    /// All observed children of the held root were processed before its sync.
    RootDirectory,
}

/// Sync each regular file before hashing it, then sync directories in post-order.
///
/// Uses the no-follow/nonblocking descriptor walker and exact path/NUL/lowercase-
/// digest/newline composition. The root descriptor selects an inode; `display_root`
/// is diagnostic only. File streams use the explicit per-file byte allowance.
/// Callers own total bytes, entries, depth, time, memory and stable namespace/content
/// custody. Sequential sync/read observations do not freeze bytes or path identities.
///
/// The hook runs before each sync and can refuse it. This operation neither creates,
/// renames nor removes entries, publishes a directory, syncs its parent link, compares
/// an expected digest nor decides recovery. Errors may follow earlier completed syncs.
///
/// # Errors
/// Rejects unsafe/non-UTF-8 children, bounded-stream failures, hook and filesystem
/// errors. Native I/O identity is retained in `TreeChecksumError::Io`.
pub fn sync_checksum_directory_at(
    directory: &File,
    display_root: &Path,
    max_file_bytes: u64,
    mut before_sync: impl FnMut(TreeSyncStep, &Path) -> io::Result<()>,
) -> Result<Sha256Digest, TreeChecksumError> {
    let directory_fd = directory.as_fd().try_clone_to_owned()?;
    let mut visitor = SyncVisitor {
        checksum: ChecksumVisitor::new(max_file_bytes),
        before_sync: &mut before_sync,
    };
    walk_tree(&directory_fd, Path::new(""), display_root, &mut visitor)?;
    visitor.checksum.finish()
}

struct SyncVisitor<'a, F> {
    checksum: ChecksumVisitor,
    before_sync: &'a mut F,
}

impl<F> TreeVisitor for SyncVisitor<'_, F>
where
    F: FnMut(TreeSyncStep, &Path) -> io::Result<()>,
{
    type Error = TreeChecksumError;

    fn file(&mut self, fd: &OwnedFd, relative: &Path, display: &Path) -> Result<(), Self::Error> {
        let mut file = File::from(fd.try_clone()?);
        (self.before_sync)(TreeSyncStep::RegularFile, display)?;
        file.sync_all()?;
        file.seek(SeekFrom::Start(0))?;
        self.checksum.record_file(file, relative)
    }

    fn enter_directory(&mut self, _relative: &Path, _display: &Path) -> Result<(), Self::Error> {
        Ok(())
    }

    fn leave_directory(
        &mut self,
        fd: &OwnedFd,
        relative: &Path,
        display: &Path,
    ) -> Result<(), Self::Error> {
        let step = if relative.as_os_str().is_empty() {
            TreeSyncStep::RootDirectory
        } else {
            TreeSyncStep::NestedDirectory
        };
        (self.before_sync)(step, display)?;
        rustix::fs::fsync(fd).map_err(io::Error::from)?;
        Ok(())
    }

    fn unsupported(&self, display: &Path, kind: FileType) -> Self::Error {
        self.checksum.unsupported(display, kind)
    }

    fn non_utf8(&self, display: &Path) -> Self::Error {
        self.checksum.non_utf8(display)
    }
}

#[cfg(test)]
mod tests;
