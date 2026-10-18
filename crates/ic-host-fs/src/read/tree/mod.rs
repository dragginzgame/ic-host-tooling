//! No-follow traversal of a caller-held Unix directory descriptor.

pub mod checksum;

use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use rustix::{
    fd::OwnedFd,
    fs::{Dir, FileType},
};

use super::entry::{entry_type, open_child, opened_type};

/// Caller-selected actions around a no-follow descriptor walk.
///
/// The visitor owns its entry policy, byte limits, copy/sync effects and error
/// projection. The caller must hold the root and prevent concurrent writers or
/// namespace replacement if it needs a stable tree. An open descriptor retains
/// an inode, not immutable bytes or a path's future identity.
pub trait TreeVisitor {
    /// Caller error that can retain filesystem failures.
    type Error: From<io::Error>;

    /// Visit one opened regular file.
    ///
    /// # Errors
    /// Returns the caller's file-processing error.
    fn file(&mut self, fd: &OwnedFd, relative: &Path, display: &Path) -> Result<(), Self::Error>;
    /// Enter a directory before visiting its children.
    ///
    /// # Errors
    /// Returns the caller's directory-entry error.
    fn enter_directory(&mut self, relative: &Path, display: &Path) -> Result<(), Self::Error>;
    /// Leave a directory after visiting its children, including the root.
    ///
    /// # Errors
    /// Returns the caller's directory-completion error.
    fn leave_directory(
        &mut self,
        fd: &OwnedFd,
        relative: &Path,
        display: &Path,
    ) -> Result<(), Self::Error>;
    /// Project an unsupported or changed entry type to the caller's error.
    fn unsupported(&self, display: &Path, kind: FileType) -> Self::Error;
    /// Project a non-UTF-8 entry path to the caller's error.
    fn non_utf8(&self, display: &Path) -> Self::Error;
}

/// Walk regular files and directories beneath an already-open root.
///
/// Every child is observed without following symlinks, opened with no-follow
/// and nonblocking flags, then checked again through its descriptor. Observed
/// directories always use directory-only opens; replacement with a nondirectory
/// can therefore return a native open error before the visitor's type callback.
/// Symlinks, special entries and non-UTF-8 names go through the visitor's rejection
/// callbacks. This does not open or confine the root, sort entries, bound tree
/// size/depth, synchronize files, or guard against concurrent content mutation.
///
/// # Errors
/// Returns directory read/open/stat failures or the visitor's own error.
pub fn walk_tree<V: TreeVisitor>(
    directory_fd: &OwnedFd,
    relative_directory: &Path,
    display_root: &Path,
    visitor: &mut V,
) -> Result<(), V::Error> {
    let mut directory = Dir::read_from(directory_fd).map_err(io::Error::from)?;
    while let Some(entry) = directory.read() {
        let entry = entry.map_err(io::Error::from)?;
        let name_bytes = entry.file_name().to_bytes();
        if matches!(name_bytes, b"." | b"..") {
            continue;
        }
        let name = std::ffi::OsStr::from_bytes(name_bytes);
        if name.to_str().is_none() {
            return Err(visitor.non_utf8(&display_root.join(name)));
        }
        let relative_path: PathBuf = relative_directory.join(name);
        let display_path = display_root.join(&relative_path);
        let kind = entry_type(directory_fd, entry.file_name())?;
        match kind {
            FileType::RegularFile | FileType::Directory => {
                let child =
                    open_child(directory_fd, entry.file_name(), kind == FileType::Directory)?;
                let opened = opened_type(&child)?;
                if opened != kind {
                    return Err(visitor.unsupported(&display_path, opened));
                }
                if kind == FileType::RegularFile {
                    visitor.file(&child, &relative_path, &display_path)?;
                } else {
                    visitor.enter_directory(&relative_path, &display_path)?;
                    walk_tree(&child, &relative_path, display_root, visitor)?;
                }
            }
            kind => return Err(visitor.unsupported(&display_path, kind)),
        }
    }
    let display_path = if relative_directory.as_os_str().is_empty() {
        display_root.to_path_buf()
    } else {
        display_root.join(relative_directory)
    };
    visitor.leave_directory(directory_fd, relative_directory, &display_path)
}

#[cfg(test)]
mod tests;
