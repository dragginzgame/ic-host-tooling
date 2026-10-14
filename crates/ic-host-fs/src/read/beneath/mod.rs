//! No-follow traversal relative to an explicitly supplied directory capability.

use super::{ArtifactError, check_metadata};
use rustix::fs::{AtFlags, FileType, Mode, OFlags, openat, statat};
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
        let stat = statat(directory, name, AtFlags::SYMLINK_NOFOLLOW).map_err(io::Error::from)?;
        if FileType::from_raw_mode(stat.st_mode) == FileType::Symlink {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "symlink component refused").into(),
            );
        }
        let final_component = parts.peek().is_none();
        let flags = OFlags::RDONLY
            | OFlags::NOFOLLOW
            | OFlags::NONBLOCK
            | OFlags::CLOEXEC
            | if final_component {
                OFlags::empty()
            } else {
                OFlags::DIRECTORY
            };
        let opened = openat(directory, name, flags, Mode::empty()).map_err(io::Error::from)?;
        if final_component {
            let file = File::from(opened);
            check_metadata(&file.metadata()?, max_bytes)?;
            return Ok(file);
        }
        parent = Some(opened);
    }
    unreachable!("validated nonempty normal path");
}

#[cfg(test)]
mod tests;
