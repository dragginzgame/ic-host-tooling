//! Unix final-component symlink rejection followed by descriptor-based reading.

#[cfg(test)]
mod tests;

use super::{ArtifactError, ArtifactIdentity, admit_file, hash_reader, read_opened_file};
use std::{
    fs::{self, File, Metadata},
    io,
    path::Path,
};

/// Open a read-only regular file without following the final component.
///
/// Shares the reader's NOFOLLOW, NONBLOCK and CLOEXEC opening implementation.
/// Descriptor metadata admits the file kind before returning; no bytes are read
/// and the cursor starts at zero. Callers can inspect, synchronize or read that
/// same descriptor without repeating Unix opening/regular-file admission.
/// Ancestor links are followed. Namespace custody, permissions, content bounds,
/// concurrent writers and filesystem latency remain caller-owned.
///
/// # Errors
/// Preserves native open/metadata errors, including final-symlink refusal, or
/// returns [`ArtifactError::NotRegularFile`] for directories, FIFOs and devices.
pub fn open_regular_file_no_follow(path: &Path) -> Result<File, ArtifactError> {
    let file = open_no_follow(path)?;
    admit_file(&file, u64::MAX)?;
    Ok(file)
}

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

/// Hash a regular file without following a final-component symlink.
///
/// Uses the same opening flags and caller-owned confinement constraints as
/// [`read_file_no_follow`], then validates the opened descriptor's type and size.
/// Hashes from its initial position with constant working memory through
/// [`hash_reader`]. Metadata is an early rejection only; the stream is bounded
/// independently to detect growth beyond `max_bytes`. Empty files accept zero.
///
/// Ancestor symlinks are followed. An open descriptor selects an inode, not
/// immutable contents; concurrent-writer custody and any exact expected-length
/// check remain caller-owned. No source bytes are written or removed.
///
/// # Errors
/// Returns [`ArtifactError::Io`] with the original open/metadata/read cause,
/// including final-symlink rejection, or typed non-regular-file/byte-limit errors.
pub fn hash_file_no_follow(path: &Path, max_bytes: u64) -> Result<ArtifactIdentity, ArtifactError> {
    hash_file_no_follow_with_metadata(path, max_bytes).map(|(identity, _)| identity)
}

/// Hash a final-component no-follow file and retain its descriptor metadata.
///
/// Shares [`hash_file_no_follow`]'s opening, regular-file and byte admission.
/// Metadata is observed before the bounded hash, without another pathname lookup.
/// Ancestors are followed; callers retain namespace/writer exclusion, permission
/// and digest policy. This establishes neither immutable bytes nor atomic execution.
///
/// # Errors
/// Returns the same open, metadata, type, byte-limit and read failures as [`hash_file_no_follow`].
pub fn hash_file_no_follow_with_metadata(
    path: &Path,
    max_bytes: u64,
) -> Result<(ArtifactIdentity, Metadata), ArtifactError> {
    let file = open_no_follow(path)?;
    let metadata = admit_file(&file, max_bytes)?;
    Ok((hash_reader(file, max_bytes)?, metadata))
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
    open_optional_no_follow(path)?
        .map(|file| read_opened_file(file, max_bytes))
        .transpose()
}

// The consuming reader must admit the selected descriptor before reading.
pub(super) fn open_optional_no_follow(path: &Path) -> Result<Option<File>, ArtifactError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() => return Err(ArtifactError::NotRegularFile),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ArtifactError::Io(error)),
    }
    open_no_follow(path).map(Some)
}

fn open_no_follow(path: &Path) -> Result<File, ArtifactError> {
    super::open_read_only(path, true)
}
