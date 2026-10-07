//! Unix final-component symlink rejection followed by descriptor-based reading.

#[cfg(test)]
mod tests;

use super::{ArtifactError, read_opened_file};
use rustix::fs::{Mode, OFlags, open};
use std::{
    fs::{self, File},
    io,
    path::Path,
};

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
    read_opened_file(open_no_follow(path)?, max_bytes)
}

/// Read an optional bounded regular file, rejecting final-component symlinks.
///
/// `None` means the initial pathname observation was missing. A dangling symlink
/// is rejected as [`ArtifactError::NotRegularFile`], not treated as absent.
/// Subsequent open/read failures remain errors, including concurrent removal.
/// Metadata and the actual stream are bounded independently. Allocation failures
/// retain their artifact error; no unbounded read or diagnostic conversion occurs.
///
/// Ancestor symlinks, concurrent writers and filesystem latency have the same
/// caller-owned constraints as [`read_file_no_follow`].
///
/// # Errors
/// Returns typed admission, open, metadata, size, read and allocation failures.
pub fn read_optional_file_no_follow(
    path: &Path,
    max_bytes: usize,
) -> Result<Option<Vec<u8>>, ArtifactError> {
    open_optional_regular_file(path)?
        .map(|file| read_opened_file(file, max_bytes))
        .transpose()
}

pub(super) fn open_optional_regular_file(path: &Path) -> Result<Option<File>, ArtifactError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() => return Err(ArtifactError::NotRegularFile),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ArtifactError::Io(error)),
    }
    let file = open_no_follow(path)?;
    if !file.metadata()?.is_file() {
        return Err(ArtifactError::NotRegularFile);
    }
    Ok(Some(file))
}

fn open_no_follow(path: &Path) -> Result<File, ArtifactError> {
    let fd = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| ArtifactError::Io(io::Error::from_raw_os_error(error.raw_os_error())))?;
    Ok(File::from(fd))
}
