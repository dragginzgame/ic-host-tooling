//! No-follow traversal relative to an explicitly supplied directory capability.

use super::{
    ArtifactError, admit_file,
    entry::{entry_type, open_child, opened_type},
};
use rustix::fs::{FileType, Mode, OFlags};
use std::{
    fs::File,
    io,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
    path::{Component, Path},
};

/// Open an existing directory without following its final symlink.
///
/// DIRECTORY and NONBLOCK refuse non-directories without waiting on a FIFO.
/// Ancestor custody and root selection remain caller-owned; use the returned
/// capability with [`open_regular_file_beneath`] for descendant traversal.
/// No entry is created or modified.
///
/// # Errors
/// Returns native open failures, including final symlink/non-directory refusal.
pub fn open_directory_no_follow(path: &Path) -> io::Result<File> {
    rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(io::Error::from)
}

/// One admitted root-relative entry with its original descriptor retained.
#[derive(Debug)]
pub enum OpenedEntry {
    /// A regular file, opened without following any selected component.
    RegularFile(File),
    /// A directory, opened without following any selected component.
    Directory(File),
}

/// Refusal or native failure when selecting a root-relative entry.
#[derive(Debug, thiserror::Error)]
pub enum OpenEntryError {
    /// A native open or metadata operation failed.
    #[error("root-relative opening failed: {0}")]
    Io(#[from] io::Error),
    /// The selected entry is neither a regular file nor a directory.
    #[error("unsupported entry: {kind:?}")]
    UnsupportedEntry {
        /// Observed descriptor kind.
        kind: FileType,
    },
}

/// Open one regular file or directory beneath a caller-held root descriptor.
///
/// Every component must be normal and is selected through its held parent.
/// Symlinks and special final entries refuse. A directory result composes with
/// [`super::tree::walk_tree`]; a file result still needs a caller-selected byte
/// allowance before reading. The caller owns root custody, stable content and
/// protection against concurrent namespace mutation. An open descriptor does
/// not prove that its original pathname still names the same entry.
///
/// # Errors
/// Rejects invalid paths, observed symlinks and unsupported final entry types,
/// or returns native traversal/open/metadata errors.
pub fn open_entry_beneath(
    root: BorrowedFd<'_>,
    relative: &Path,
) -> Result<OpenedEntry, OpenEntryError> {
    let mut parts = relative.components().peekable();
    if parts.peek().is_none() || parts.clone().any(|p| !matches!(p, Component::Normal(_))) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "relative normal components required",
        )
        .into());
    }
    let mut parent: Option<OwnedFd> = None;
    while let Some(Component::Normal(name)) = parts.next() {
        let directory = parent.as_ref().map_or(root, AsFd::as_fd);
        let observed = entry_type(directory, name)?;
        if observed == FileType::Symlink {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "symlink component refused").into(),
            );
        }
        let final_component = parts.peek().is_none();
        if final_component && !matches!(observed, FileType::RegularFile | FileType::Directory) {
            return Err(OpenEntryError::UnsupportedEntry { kind: observed });
        }
        let opened = open_child(
            directory,
            name,
            !final_component || observed == FileType::Directory,
        )?;
        if final_component {
            let actual = opened_type(&opened)?;
            let file = File::from(opened);
            return match (observed, actual) {
                (FileType::RegularFile, FileType::RegularFile) => {
                    Ok(OpenedEntry::RegularFile(file))
                }
                (FileType::Directory, FileType::Directory) => Ok(OpenedEntry::Directory(file)),
                _ => Err(OpenEntryError::UnsupportedEntry { kind: actual }),
            };
        }
        parent = Some(opened);
    }
    unreachable!("validated nonempty normal path");
}

/// Open one bounded regular file through a caller-owned directory descriptor.
///
/// Rejects empty/absolute paths, parent traversal and symlinks at every selected
/// component. Each admitted parent descriptor is retained until its child opens;
/// path replacement cannot redirect later traversal through an ancestor alias.
/// Final opening uses NONBLOCK before type/size admission, so a FIFO without a
/// writer is refused without waiting. Reads can use [`super::read_opened_file`].
///
/// The caller owns the root's admission/custody, relative-name bounds, credentials,
/// deadlines and content validation. An opened inode is not immutable content;
/// concurrent writers and hard links are not excluded. Directory renames retain
/// the selected capability; this does not prove current pathname containment.
/// No file or directory is created, modified, synced or removed.
///
/// # Errors
/// Syntax and observed symlink refusals return an `InvalidInput` I/O cause.
/// Other traversal/open errors retain their native cause. Opened type/size and
/// metadata failures use the existing artifact admission errors.
pub fn open_regular_file_beneath(
    root: BorrowedFd<'_>,
    relative: &Path,
    max_bytes: u64,
) -> Result<File, ArtifactError> {
    let opened = open_entry_beneath(root, relative).map_err(|error| match error {
        OpenEntryError::Io(error) => ArtifactError::Io(error),
        OpenEntryError::UnsupportedEntry { .. } => ArtifactError::NotRegularFile,
    })?;
    match opened {
        OpenedEntry::RegularFile(file) => {
            admit_file(&file, max_bytes)?;
            Ok(file)
        }
        OpenedEntry::Directory(_) => Err(ArtifactError::NotRegularFile),
    }
}

#[cfg(test)]
mod tests;
