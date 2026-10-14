//! Narrow no-follow directory selection and held-inode comparison.

use std::{fs::File, io, path::Path};

#[cfg(test)]
mod tests;

/// Open an existing directory without following its final symlink.
///
/// # Errors
/// Rejects missing, non-directory and symlink entries or unsupported hosts.
pub fn open_no_follow(path: &Path) -> io::Result<File> {
    #[cfg(unix)]
    {
        use rustix::fs::{Mode, OFlags, open};
        open(
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map(File::from)
        .map_err(io::Error::from)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

/// Compare a current path entry with an already held directory descriptor.
///
/// # Errors
/// Returns filesystem failures or unsupported-host admission errors.
pub fn same_held_inode(path: &Path, held: &File) -> io::Result<bool> {
    #[cfg(unix)]
    {
        use std::{fs, os::unix::fs::MetadataExt};
        let current = fs::symlink_metadata(path)?;
        let original = held.metadata()?;
        Ok(current.is_dir() && current.dev() == original.dev() && current.ino() == original.ino())
    }
    #[cfg(not(unix))]
    {
        let _ = (path, held);
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}
