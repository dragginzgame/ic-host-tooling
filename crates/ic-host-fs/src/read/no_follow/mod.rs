//! Unix final-component symlink rejection followed by descriptor-based reading.

#[cfg(test)]
mod tests;

use super::{ArtifactError, read_opened_file};
use rustix::fs::{Mode, OFlags, open};
use std::{fs::File, io, path::Path};

/// Read a bounded regular file without following a final-component symlink.
///
/// Uses Unix `NOFOLLOW`, `NONBLOCK` and `CLOEXEC` flags, then validates descriptor
/// metadata with [`read_opened_file`]. A FIFO without a writer can be opened
/// without waiting and is rejected before reading. Directories and other opened
/// special files are also rejected. No source bytes are written or removed.
///
/// This is a final-component boundary, not root confinement: ancestor symlinks
/// are followed. Consumers must supply trusted ancestors and retain their root,
/// permission, ownership and missing-file policy. Filesystem syscalls and regular
/// reads can still block; deadlines remain caller-owned. Concurrent writers can
/// change contents even after this descriptor is selected.
/// Existing [`super::read_file`] and [`super::hash_file`] continue to follow links.
///
/// # Errors
/// Returns [`ArtifactError::Io`] for open failures (including final symlinks),
/// or typed non-regular-file, metadata, size, read and allocation failures.
pub fn read_file_no_follow(path: &Path, max_bytes: usize) -> Result<Vec<u8>, ArtifactError> {
    let fd = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| ArtifactError::Io(io::Error::from_raw_os_error(error.raw_os_error())))?;
    read_opened_file(File::from(fd), max_bytes)
}
