//! Bounded regular-file reads; callers own path confinement and concurrent writers.

use ic_host_artifacts::artifact::{ArtifactError, ArtifactIdentity, hash_reader, read_reader};
use std::{fs::File, path::Path};

#[cfg(unix)]
mod beneath;
#[cfg(unix)]
pub use beneath::{
    OpenedEntry, open_directory_no_follow, open_entry_beneath, open_regular_file_beneath,
};

#[cfg(unix)]
mod no_follow;
#[cfg(unix)]
pub mod tree;
#[cfg(unix)]
pub use no_follow::hash_file_no_follow;
#[cfg(unix)]
pub use no_follow::read_file_no_follow;
#[cfg(unix)]
pub use no_follow::read_optional_file_no_follow;
#[cfg(unix)]
mod private;
#[cfg(unix)]
pub use private::{PrivateFileReadError, read_private_bytes};
#[cfg(test)]
mod tests;

/// Hash a regular file without loading it into memory.
///
/// Trusted path selection and protection against concurrent replacement remain
/// caller-owned. This does not admit a path for later execution.
/// Open errors precede descriptor type/size admission; the bounded stream then
/// detects growth beyond the allowance.
///
/// # Errors
/// Returns filesystem, non-regular-file, or byte-limit failures.
pub fn hash_file(path: &Path, max_bytes: u64) -> Result<ArtifactIdentity, ArtifactError> {
    hash_reader(open_file(path, max_bytes)?, max_bytes)
}

/// Read a regular artifact into bounded, fallibly allocated storage.
///
/// The opened descriptor supplies type and early size admission; the stream is
/// bounded independently to detect growth. Open errors precede that admission.
/// Paths must be selected from a caller-controlled filesystem.
///
/// # Errors
/// Returns filesystem, non-regular-file, allocation, or byte-limit failures.
pub fn read_file(path: &Path, max_bytes: usize) -> Result<Vec<u8>, ArtifactError> {
    read_reader(open_file(path, max_bytes as u64)?, max_bytes)
}

/// Validate an already-open regular file and read it into bounded storage.
///
/// Consumes the descriptor without reopening a pathname. Its metadata must
/// describe a regular file whose complete length fits `max_bytes`; the stream
/// is bounded independently to detect growth. Reading begins at the descriptor's
/// current position. Supply a newly opened or explicitly rewound descriptor to
/// read the complete file. Storage is allocated fallibly by [`read_reader`].
///
/// Opening policy, confinement, symlink provenance and concurrent modification
/// remain caller-owned. An already-open special file is rejected before reading;
/// this cannot undo effects or blocking that occurred when the caller opened it.
/// An open descriptor retains the selected inode, not immutable contents.
///
/// # Errors
/// Returns metadata, non-regular-file, byte-limit, read or allocation failures.
pub fn read_opened_file(file: File, max_bytes: usize) -> Result<Vec<u8>, ArtifactError> {
    check_metadata(&file.metadata()?, max_bytes as u64)?;
    read_reader(file, max_bytes)
}

/// Open a read-only regular file, following symlinks and retaining its descriptor.
///
/// Unix opens are nonblocking so a FIFO cannot block before descriptor admission.
/// The opened descriptor, rather than a pathname precheck, establishes file kind.
/// Other hosts use their ordinary read-only open. This supplies no byte limit,
/// root confinement, content immutability or filesystem latency deadline.
/// Callers own trusted path selection, expected length and concurrent writers;
/// use no-follow reads when final symlinks must be refused.
///
/// # Errors
/// Returns the original open/metadata error or [`ArtifactError::NotRegularFile`].
pub fn open_regular_file(path: &Path) -> Result<File, ArtifactError> {
    open_file(path, u64::MAX)
}

fn open_file(path: &Path, limit: u64) -> Result<File, ArtifactError> {
    #[cfg(unix)]
    let file = open_read_only(path, false)?;
    #[cfg(not(unix))]
    let file = File::open(path)?;
    check_metadata(&file.metadata()?, limit)?;
    Ok(file)
}

#[cfg(unix)]
fn open_read_only(path: &Path, no_follow: bool) -> Result<File, ArtifactError> {
    use rustix::fs::{Mode, OFlags, open};
    let mut flags = OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC;
    if no_follow {
        flags |= OFlags::NOFOLLOW;
    }
    Ok(File::from(
        open(path, flags, Mode::empty()).map_err(std::io::Error::from)?,
    ))
}

fn check_metadata(metadata: &std::fs::Metadata, limit: u64) -> Result<(), ArtifactError> {
    if !metadata.is_file() {
        return Err(ArtifactError::NotRegularFile);
    }
    if metadata.len() > limit {
        return Err(ArtifactError::LimitExceeded { limit });
    }
    Ok(())
}
