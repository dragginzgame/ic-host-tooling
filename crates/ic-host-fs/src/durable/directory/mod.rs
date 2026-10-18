//! Retained private directory creation and pathname admission.
use std::{fs::File, io, path::Path};

mod publish;

pub use publish::{
    DirectoryCommitError, DirectoryCommitOutcome, DirectoryCommitStep,
    commit_existing_directory_with,
};

/// Creation failure distinguishes a refused claim from a possibly visible directory.
#[derive(Debug, thiserror::Error)]
pub enum DirectoryCreationError {
    /// No directory was created by this call (including an occupied name).
    #[error("private directory creation refused")]
    BeforeCreation(#[source] io::Error),
    /// The directory was created; admission or durability could not be confirmed.
    /// Retain and reconcile the original name rather than automatically retrying.
    #[error("created private directory admission failed")]
    AfterCreation(#[source] io::Error),
}

/// Create one owner-only directory, synchronize its parent link and retain its descriptor.
///
/// Uses `0700` reduced by umask; never creates parents, follows a final symlink,
/// replaces an occupied entry or removes evidence on error/Drop. Callers own
/// trusted ancestors, root confinement and exclusion of concurrent namespace writers.
/// The descriptor remains the original directory after a rename; use
/// [`admit_directory`] before effects that require the original pathname claim.
///
/// # Errors
/// Returns creation phase and native I/O cause, including unsupported hosts.
pub fn create_private_directory(path: &Path) -> Result<File, DirectoryCreationError> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        // Admission precedes creation so an invalid namespace leaves no new claim.
        for directory in [
            path.to_path_buf(),
            parent
                .canonicalize()
                .map_err(DirectoryCreationError::BeforeCreation)?,
        ] {
            if directory
                .as_os_str()
                .as_encoded_bytes()
                .iter()
                .any(|b| matches!(b, b'\r' | b'\n'))
            {
                return Err(DirectoryCreationError::BeforeCreation(
                    io::ErrorKind::InvalidInput.into(),
                ));
            }
        }
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(DirectoryCreationError::BeforeCreation)?;
        let finish = || -> io::Result<File> {
            use rustix::fs::{Mode, OFlags, open};
            File::open(parent)?.sync_all()?;
            let file = File::from(open(
                path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )?);
            admit_directory(path, &file)?;
            Ok(file)
        };
        finish().map_err(DirectoryCreationError::AfterCreation)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = path;
        Err(DirectoryCreationError::BeforeCreation(
            io::ErrorKind::Unsupported.into(),
        ))
    }
}

/// Require a no-follow directory pathname to still identify the held directory.
///
/// This is an observation, not protection against a subsequent concurrent rename.
/// Callers must retain trusted ancestors and exclusive namespace custody.
/// # Errors
/// Missing, symlink, non-directory, replaced, non-directory descriptor and unsupported hosts refuse.
pub fn admit_directory(path: &Path, directory: &File) -> io::Result<()> {
    #[cfg(unix)]
    {
        use crate::identity::FileIdentity;
        let current = std::fs::symlink_metadata(path)?;
        let held = directory.metadata()?;
        if !current.is_dir()
            || !held.is_dir()
            || FileIdentity::from_metadata(&current) != FileIdentity::from_metadata(&held)
        {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (path, directory);
        Err(io::ErrorKind::Unsupported.into())
    }
}

#[cfg(test)]
mod tests;
