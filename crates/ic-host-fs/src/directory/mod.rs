//! Narrow no-follow directory selection; held identity admission is durable-owned.

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
