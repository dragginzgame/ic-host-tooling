//! Fixed-size private reads with explicit admission failures.

use super::no_follow::open_optional_regular_file;
use ic_host_artifacts::artifact::ArtifactError;
use std::{
    fmt,
    fs::File,
    io::{self, Read},
    os::unix::fs::MetadataExt as _,
    path::Path,
};

#[cfg(test)]
mod tests;

/// A private-file read failed without returning partial bytes.
#[derive(Debug)]
pub enum PrivateFileReadError {
    /// Opening, file admission, metadata or reading failed.
    Read(ArtifactError),
    /// The opened descriptor does not have exactly owner read/write permissions.
    Permissions {
        /// Observed Unix permission bits, masked to `0o777`.
        mode: u32,
    },
    /// The opened file does not have exactly one directory link.
    LinkCount {
        /// Observed number of links to the inode.
        actual: u64,
    },
    /// Metadata or a trailing byte proves the file is not the required length.
    Length {
        /// Required complete byte length.
        expected: usize,
    },
}

impl fmt::Display for PrivateFileReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(_) => formatter.write_str("private file read failed"),
            Self::Permissions { mode } => write!(
                formatter,
                "private file permissions are {mode:o}, expected 600"
            ),
            Self::LinkCount { actual } => {
                write!(formatter, "private file has {actual} links, expected one")
            }
            Self::Length { expected } => write!(
                formatter,
                "private file must contain exactly {expected} bytes"
            ),
        }
    }
}

impl std::error::Error for PrivateFileReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(source) => Some(source),
            _ => None,
        }
    }
}

/// Read exactly `N` private bytes through a no-follow regular descriptor.
///
/// Only an initially missing pathname returns `Ok(None)`. The opened descriptor
/// must have permission bits `0o600`, one link and length `N`; a trailing read
/// also rejects growth. I/O and admission failures remain distinct from absence.
/// No file is created, repaired, removed or replaced. Consumers own whether an
/// error disables an optional feature or stops an operation; it never means a
/// key can safely be regenerated. Contents are not included in error messages.
///
/// Ancestors must be trusted. Descriptor selection does not freeze contents or
/// permissions; concurrent writers and ownership policy remain caller-owned.
/// `N` is caller-selected and allocates a fixed array on the stack.
///
/// # Errors
/// Returns typed open/read failures, permission/link rejection or invalid length.
pub fn read_private_bytes<const N: usize>(
    path: &Path,
) -> Result<Option<[u8; N]>, PrivateFileReadError> {
    open_optional_regular_file(path)
        .map_err(PrivateFileReadError::Read)?
        .map(read_private_descriptor)
        .transpose()
}

fn read_private_descriptor<const N: usize>(
    mut file: File,
) -> Result<[u8; N], PrivateFileReadError> {
    let io_error = |source| PrivateFileReadError::Read(ArtifactError::Io(source));
    let metadata = file.metadata().map_err(io_error)?;
    if metadata.len() != N as u64 {
        return Err(PrivateFileReadError::Length { expected: N });
    }
    let mode = metadata.mode() & 0o777;
    if mode != 0o600 {
        return Err(PrivateFileReadError::Permissions { mode });
    }
    if metadata.nlink() != 1 {
        return Err(PrivateFileReadError::LinkCount {
            actual: metadata.nlink(),
        });
    }
    read_exact_private(&mut file)
}

fn read_exact_private<const N: usize>(
    reader: &mut impl Read,
) -> Result<[u8; N], PrivateFileReadError> {
    let io_error = |source| PrivateFileReadError::Read(ArtifactError::Io(source));
    let mut bytes = [0; N];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    let mut extra = [0];
    loop {
        match reader.read(&mut extra) {
            Ok(0) => return Ok(bytes),
            Ok(_) => return Err(PrivateFileReadError::Length { expected: N }),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(io_error(error)),
        }
    }
}
