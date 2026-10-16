//! Create-new copies from held Unix descriptors, without publication policy.

use crate::{
    durable::{DirectoryCreationError, create_private_directory},
    read::tree::{TreeVisitor, walk_tree},
};
use ic_host_artifacts::artifact::{ArtifactIdentity, copy_reader};
use rustix::{
    fd::{AsFd, OwnedFd},
    fs::FileType,
};
use std::{
    fmt,
    fs::{File, OpenOptions},
    io,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

/// Copy refusal; partial output and its namespace remain caller-owned.
#[derive(Debug)]
pub enum TreeCopyError {
    /// Native filesystem or bounded stream failure.
    Io(io::Error),
    /// Private directory creation failed before or after creation.
    Directory(DirectoryCreationError),
    /// A symlink, special entry or changed descriptor type was observed.
    UnsupportedEntry {
        /// Source display path, never used for opening.
        path: PathBuf,
        /// Observed or descriptor-admitted unsupported type.
        kind: FileType,
    },
    /// An entry name could not retain UTF-8 identity.
    NonUtf8Path {
        /// Exact rejected source display path.
        path: PathBuf,
    },
}

impl fmt::Display for TreeCopyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Directory(error) => error.fmt(f),
            Self::UnsupportedEntry { path, kind } => {
                write!(f, "unsupported {kind:?}: {}", path.display())
            }
            Self::NonUtf8Path { path } => write!(f, "non-UTF-8 path: {}", path.display()),
        }
    }
}

impl std::error::Error for TreeCopyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Directory(error) => Some(error),
            Self::UnsupportedEntry { .. } | Self::NonUtf8Path { .. } => None,
        }
    }
}

impl From<io::Error> for TreeCopyError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Copy an opened regular file at its current offset to a new private file.
///
/// Uses a bounded transfer buffer and returns the identity of bytes written.
/// The file is created with mode `0600` subject to umask. Source descriptor
/// clones share their cursor. This does not freeze bytes, rewind, flush, sync,
/// publish or remove partial output. The caller must hold source content and
/// destination namespace custody, including all destination ancestors.
///
/// # Errors
/// Refuses nonregular descriptors, occupied destinations and streams exceeding
/// `max_bytes`. Native IO causes retain their identity; other bounded stream
/// failures remain downcastable causes through `io::Error`.
pub fn copy_file(
    source: &impl AsFd,
    destination: &Path,
    max_bytes: u64,
) -> io::Result<ArtifactIdentity> {
    let metadata = rustix::fs::fstat(source)?;
    if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "copy source is not a regular file",
        ));
    }
    let source = File::from(source.as_fd().try_clone_to_owned()?);
    let mut destination = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)?;
    copy_reader(source, &mut destination, max_bytes).map_err(io::Error::from)
}

/// Copy a held directory tree into a new private directory.
///
/// Reuses the no-follow descriptor walker and [`copy_file`]. Every directory is
/// created with `0700` and its parent link synchronized; file contents are not
/// synchronized. Returns unsorted relative file paths and stream identities;
/// empty directories are copied but add no file identity. No aggregate digest,
/// publication, cleanup, retry, tree size/depth or aggregate memory bound is
/// chosen. The caller owns those policies and both namespaces throughout.
///
/// `display_root` labels source failures only; it is never opened. The source
/// descriptor must identify a directory. Each file has `max_file_bytes` allowance.
///
/// # Errors
/// Refuses occupied output, symlinks, special entries, non-UTF-8 names and native
/// traversal/copy failures. Created directories and partial files remain in place.
pub fn copy_directory(
    source: &OwnedFd,
    destination: &Path,
    display_root: &Path,
    max_file_bytes: u64,
) -> Result<Vec<(PathBuf, ArtifactIdentity)>, TreeCopyError> {
    create_private_directory(destination).map_err(TreeCopyError::Directory)?;
    let mut visitor = CopyVisitor {
        destination,
        max_file_bytes,
        files: Vec::new(),
    };
    walk_tree(source, Path::new(""), display_root, &mut visitor)?;
    Ok(visitor.files)
}

struct CopyVisitor<'a> {
    destination: &'a Path,
    max_file_bytes: u64,
    files: Vec<(PathBuf, ArtifactIdentity)>,
}

impl TreeVisitor for CopyVisitor<'_> {
    type Error = TreeCopyError;

    fn file(&mut self, fd: &OwnedFd, relative: &Path, _display: &Path) -> Result<(), Self::Error> {
        let identity = copy_file(fd, &self.destination.join(relative), self.max_file_bytes)?;
        self.files.push((relative.to_owned(), identity));
        Ok(())
    }

    fn enter_directory(&mut self, relative: &Path, _display: &Path) -> Result<(), Self::Error> {
        create_private_directory(&self.destination.join(relative))
            .map_err(TreeCopyError::Directory)?;
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
        TreeCopyError::UnsupportedEntry {
            path: display.to_owned(),
            kind,
        }
    }

    fn non_utf8(&self, display: &Path) -> Self::Error {
        TreeCopyError::NonUtf8Path {
            path: display.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests;
