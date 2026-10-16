//! Explicit invocation-owned cleanup without allocation, publication or pruning.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Cleanup responsibility for one caller-owned pathname.
///
/// Construction creates nothing and admits no ownership. The caller must own the
/// selected child and exclude ancestor/namespace replacement through every use.
/// Drop attempts cleanup while armed; explicit disarming transfers recovery.
#[derive(Debug)]
pub struct OwnedDirectory {
    path: PathBuf,
    armed: bool,
}

impl OwnedDirectory {
    /// Assert exclusive cleanup custody of this path; no filesystem operation runs.
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    /// Borrow the selected pathname.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether Drop still has cleanup responsibility.
    #[must_use]
    pub const fn is_armed(&self) -> bool {
        self.armed
    }

    /// Transfer cleanup/recovery responsibility to the caller.
    pub const fn disarm(&mut self) {
        self.armed = false;
    }

    /// Consume the guard without removing the selected child.
    pub fn preserve(mut self) {
        self.disarm();
    }

    /// Remove the selected child and disarm on success. Failure leaves Drop armed.
    ///
    /// # Errors
    /// Returns original metadata/removal I/O failures.
    pub fn try_cleanup(&mut self) -> io::Result<()> {
        remove_path_if_present(&self.path)?;
        self.disarm();
        Ok(())
    }

    /// Consume the guard, attempting cleanup; failure retries best-effort on Drop.
    ///
    /// # Errors
    /// Returns the first cleanup I/O failure.
    pub fn cleanup(mut self) -> io::Result<()> {
        self.try_cleanup()
    }
}

impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        if self.armed {
            let _ = remove_path_if_present(&self.path);
            self.disarm();
        }
    }
}

/// Remove one caller-owned path if present, without following a final symlink.
///
/// A directory is removed recursively; other entries are unlinked. Callers own
/// destructive-operation policy and trusted ancestor/namespace custody.
///
/// # Errors
/// Returns metadata or removal failures other than an initially absent path.
pub fn remove_path_if_present(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests;
