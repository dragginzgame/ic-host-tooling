//! Scoped named files using tempfile's allocation and cleanup implementation.

use std::{fs::File, io, os::unix::ffi::OsStrExt as _, path::Path};

/// A private named file removed on normal drop, including unwinding.
///
/// The tempfile backend stays private. Callers select the existing parent and
/// prefix, control bytes and budgets, and must retain namespace custody. Keep
/// this guard alive through every child using its path. Dropping it unlinks the
/// name even when file clones remain open; it does not wait for children.
/// Cleanup on drop is best effort; use [`Self::close`] to observe removal errors.
/// Process termination without destructors can leave files. No disk sync,
/// hostile-directory confinement, retention or remote-effect recovery is supplied.
/// Use [`super::create_private_bytes`] when successful files must remain owned
/// by the caller after this scope for reconciliation or explicit cleanup.
#[derive(Debug)]
pub struct TemporaryFile {
    inner: tempfile::NamedTempFile,
}

impl TemporaryFile {
    /// Create an empty read/write file beneath the selected existing parent.
    ///
    /// The prefix must be one normal component without NUL/CR/LF. Parent aliases
    /// resolve before creation, yielding an absolute path; lexical or physical
    /// directory paths containing CR/LF are refused. No parent is created.
    /// tempfile owns random names, exclusive creation and collision handling.
    /// Unix permissions are 0600 subject to umask; existing entries are preserved.
    ///
    /// # Errors
    /// Returns invalid path/prefix, parent resolution or original creation errors.
    pub fn new_in(parent: &Path, prefix: &str) -> io::Result<Self> {
        super::validate_prefix(prefix)?;
        validate_parent(parent)?;
        let parent = parent.canonicalize()?;
        validate_parent(&parent)?;
        Ok(Self {
            inner: tempfile::Builder::new()
                .prefix(prefix)
                .tempfile_in(parent)?,
        })
    }

    /// Borrow the absolute path while keeping this cleanup guard alive.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    /// Borrow the standard file descriptor without transferring cleanup custody.
    #[must_use]
    pub fn as_file(&self) -> &File {
        self.inner.as_file()
    }

    /// Mutably borrow the standard file for caller-selected writes or reads.
    #[must_use]
    pub fn as_file_mut(&mut self) -> &mut File {
        self.inner.as_file_mut()
    }

    /// Consume the guard, close the file and attempt removal with error reporting.
    ///
    /// # Errors
    /// Returns the original removal error. The guard is consumed on either result;
    /// callers needing pathname evidence must retain it before calling close.
    pub fn close(self) -> io::Result<()> {
        self.inner.close()
    }
}

fn validate_parent(parent: &Path) -> io::Result<()> {
    if parent
        .as_os_str()
        .as_bytes()
        .iter()
        .any(|byte| matches!(byte, b'\r' | b'\n'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "temporary parent contains CR/LF",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
