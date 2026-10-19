//! File reads and metadata observations; callers own bounds and namespace custody.

use ic_host_artifacts::artifact::{ArtifactError, ArtifactIdentity, hash_reader, read_reader};
use std::{
    fs::{File, Metadata},
    io::Read as _,
    path::Path,
};

mod size;
pub use size::directory_logical_size;

#[cfg(unix)]
mod beneath;
#[cfg(unix)]
mod entry;
#[cfg(unix)]
pub use beneath::{
    OpenEntryError, OpenedEntry, open_directory_no_follow, open_entry_beneath,
    open_regular_file_beneath,
};

#[cfg(unix)]
mod no_follow;
#[cfg(unix)]
pub mod tree;
#[cfg(unix)]
pub use no_follow::open_regular_file_no_follow;
#[cfg(unix)]
pub use no_follow::read_file_no_follow;
#[cfg(unix)]
pub use no_follow::read_optional_file_no_follow;
#[cfg(unix)]
pub use no_follow::{hash_file_no_follow, hash_file_no_follow_with_metadata};
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
    hash_file_with_metadata(path, max_bytes).map(|(identity, _)| identity)
}

/// Hash a regular file and retain metadata from the same opened descriptor.
///
/// Follows symlinks with [`hash_file`]'s admission and byte bounds. Metadata is
/// observed before hashing; it neither freezes permissions/contents nor proves
/// that the pathname still names this inode. Callers own namespace and writer
/// exclusion, permission/digest policy and any later execution or publication.
///
/// # Errors
/// Returns the same open, metadata, type, byte-limit and read failures as [`hash_file`].
pub fn hash_file_with_metadata(
    path: &Path,
    max_bytes: u64,
) -> Result<(ArtifactIdentity, Metadata), ArtifactError> {
    let (file, metadata) = open_file(path, max_bytes)?;
    Ok((hash_reader(file, max_bytes)?, metadata))
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
    read_reader(open_file(path, max_bytes as u64)?.0, max_bytes)
}

/// Read at most `max_bytes` from a regular file, returning its prefix and observed length.
///
/// Unlike [`read_file`], a longer file is permitted. The returned length is
/// observed through the opened descriptor before reading; concurrent growth,
/// truncation and writes can make it differ from the length after this call.
/// A zero byte allowance still opens and admits the file, returning empty bytes.
/// Storage is allocated fallibly and no bytes beyond the prefix are read.
///
/// Follows symlinks with [`open_regular_file`]'s descriptor admission, including
/// nonblocking FIFO rejection on Unix. Callers own path confinement, concurrent
/// writers, decoding, truncation presentation and any filesystem latency policy.
///
/// # Errors
/// Returns filesystem, non-regular-file, read or allocation failures.
pub fn read_file_prefix(path: &Path, max_bytes: usize) -> Result<(Vec<u8>, u64), ArtifactError> {
    let (file, metadata) = open_file(path, u64::MAX)?;
    let bytes = read_reader(file.take(max_bytes as u64), max_bytes)?;
    Ok((bytes, metadata.len()))
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
    admit_file(&file, max_bytes as u64)?;
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
    Ok(open_file(path, u64::MAX)?.0)
}

fn open_file(path: &Path, limit: u64) -> Result<(File, Metadata), ArtifactError> {
    #[cfg(unix)]
    let file = open_read_only(path, false)?;
    #[cfg(not(unix))]
    let file = File::open(path)?;
    let metadata = admit_file(&file, limit)?;
    Ok((file, metadata))
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

fn admit_file(file: &File, limit: u64) -> Result<Metadata, ArtifactError> {
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(ArtifactError::NotRegularFile);
    }
    if metadata.len() > limit {
        return Err(ArtifactError::LimitExceeded { limit });
    }
    Ok(metadata)
}
