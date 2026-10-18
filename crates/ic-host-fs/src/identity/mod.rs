//! Native inode identity observations, without pathname or lifetime custody.

use std::{
    fs::{File, Metadata},
    io,
    os::unix::fs::MetadataExt as _,
};

/// Native device/inode identity; callers own file-kind admission and persistence.
///
/// An identity does not freeze contents or prevent inode reuse after all held
/// descriptors close. It establishes neither confinement nor namespace custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    /// Filesystem device number.
    pub device: u64,
    /// Inode number on that device.
    pub inode: u64,
}

impl FileIdentity {
    /// Observe a caller-owned regular descriptor without reopening its path.
    ///
    /// # Errors
    /// Returns metadata failures or rejects a non-regular descriptor.
    pub fn from_file(file: &File) -> io::Result<Self> {
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        Ok(Self::from_metadata(&metadata))
    }

    /// Observe native identity from metadata of any file kind.
    ///
    /// Callers choose following or no-follow metadata and admit the file kind.
    #[must_use]
    pub fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }

    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    #[cfg_attr(target_vendor = "apple", allow(clippy::cast_sign_loss))]
    pub(crate) const fn from_stat(stat: &rustix::fs::Stat) -> Self {
        Self {
            // Match std's MetadataExt::dev projection of Darwin's signed dev_t.
            #[cfg(target_vendor = "apple")]
            device: stat.st_dev as u64,
            #[cfg(not(target_vendor = "apple"))]
            device: stat.st_dev,
            inode: stat.st_ino,
        }
    }
}

#[cfg(test)]
mod tests;
