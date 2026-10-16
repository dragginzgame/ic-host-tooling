//! No-replace publication of one caller-verified existing directory tree.

use std::{ffi::OsStr, fmt, fs, io, path::Path};

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
use rustix::{
    fd::AsFd,
    fs::{self as unix_fs, Mode, OFlags, RenameFlags},
};

/// Result of a directory publication or recovery of the same named tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryCommitOutcome {
    /// The verified staging directory acquired the canonical name.
    Published,
    /// The canonical directory already existed and passed caller verification.
    Recovered,
}

/// Observable filesystem boundaries for deterministic failure qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryCommitStep {
    /// Verification has passed, immediately before the no-replace rename.
    BeforePublication,
    /// The canonical name exists, immediately before the parent sync.
    BeforeParentSync,
    /// The parent sync has completed.
    PublicationDurable,
}

/// Failure to publish or recover one existing directory tree.
#[derive(Debug)]
pub enum DirectoryCommitError<E> {
    /// Paths were equal, nameless or not direct siblings.
    InvalidPaths,
    /// Both staging and canonical entries exist.
    Conflict,
    /// Neither staging nor canonical entry exists.
    Missing,
    /// Caller verification or tree synchronization rejected the selected tree.
    Prepare(E),
    /// Filesystem admission or a hook failed before publication.
    BeforePublication(io::Error),
    /// A canonical name may be visible, but its parent sync or final hook failed.
    AfterPublication(io::Error),
}

impl<E: fmt::Display> fmt::Display for DirectoryCommitError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPaths => {
                formatter.write_str("directory commit requires distinct sibling paths")
            }
            Self::Conflict => formatter.write_str("directory commit has two occupied paths"),
            Self::Missing => formatter.write_str("directory commit has no existing path"),
            Self::Prepare(error) => write!(formatter, "directory preparation failed: {error}"),
            Self::BeforePublication(error) => {
                write!(formatter, "directory publication failed: {error}")
            }
            Self::AfterPublication(error) => {
                write!(formatter, "directory publication may be visible: {error}")
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for DirectoryCommitError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Prepare(error) => Some(error),
            Self::BeforePublication(error) | Self::AfterPublication(error) => Some(error),
            Self::InvalidPaths | Self::Conflict | Self::Missing => None,
        }
    }
}

/// Admit, prepare and publish one existing directory without replacing another name.
///
/// `prepare` receives the held no-follow directory descriptor and must verify and
/// synchronize every required child and the root before returning success. It is
/// invoked exactly once, on staging for a fresh commit or canonical for recovery.
/// The parent is then synchronized after publication or recovery. No temporary
/// tree is created or removed, and no failed evidence is cleaned up. A failed
/// post-rename sync or hook requires caller reconciliation before retrying.
///
/// The caller owns stable parent namespace, exclusive writers, complete byte
/// verification, tree traversal limits and the meaning of a recovered tree.
/// This is not root confinement or a lock. `at_step` is for deterministic failure
/// qualification; it must not issue other publication effects.
///
/// # Errors
/// Returns path shape/conflict/missing, caller preparation or phase-aware I/O errors.
pub fn commit_existing_directory_with<E>(
    staging: &Path,
    canonical: &Path,
    prepare: impl FnOnce(&fs::File, &Path) -> Result<(), E>,
    mut at_step: impl FnMut(DirectoryCommitStep, &Path) -> io::Result<()>,
) -> Result<DirectoryCommitOutcome, DirectoryCommitError<E>> {
    let (parent, staging_name, canonical_name) = sibling_paths(staging, canonical)?;
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        let staging_exists =
            exists_without_following(staging).map_err(DirectoryCommitError::BeforePublication)?;
        let canonical_exists =
            exists_without_following(canonical).map_err(DirectoryCommitError::BeforePublication)?;
        let parent_fd = unix_fs::open(
            parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io::Error::from)
        .map_err(DirectoryCommitError::BeforePublication)?;

        let (selected_name, selected_path, publish) = match (staging_exists, canonical_exists) {
            (true, false) => (staging_name, staging, true),
            (false, true) => (canonical_name, canonical, false),
            (true, true) => return Err(DirectoryCommitError::Conflict),
            (false, false) => return Err(DirectoryCommitError::Missing),
        };
        let selected = open_directory_at(&parent_fd, selected_name)
            .map_err(DirectoryCommitError::BeforePublication)?;
        prepare(&selected, selected_path).map_err(DirectoryCommitError::Prepare)?;
        if publish {
            at_step(DirectoryCommitStep::BeforePublication, canonical)
                .map_err(DirectoryCommitError::BeforePublication)?;
            unix_fs::renameat_with(
                &parent_fd,
                staging_name,
                &parent_fd,
                canonical_name,
                RenameFlags::NOREPLACE,
            )
            .map_err(io::Error::from)
            .map_err(DirectoryCommitError::BeforePublication)?;
        }
        at_step(DirectoryCommitStep::BeforeParentSync, parent)
            .map_err(DirectoryCommitError::AfterPublication)?;
        unix_fs::fsync(&parent_fd)
            .map_err(io::Error::from)
            .map_err(DirectoryCommitError::AfterPublication)?;
        at_step(DirectoryCommitStep::PublicationDurable, canonical)
            .map_err(DirectoryCommitError::AfterPublication)?;
        Ok(if publish {
            DirectoryCommitOutcome::Published
        } else {
            DirectoryCommitOutcome::Recovered
        })
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (parent, staging_name, canonical_name, prepare, at_step);
        Err(DirectoryCommitError::BeforePublication(io::Error::from(
            io::ErrorKind::Unsupported,
        )))
    }
}

fn sibling_paths<'a, E>(
    staging: &'a Path,
    canonical: &'a Path,
) -> Result<(&'a Path, &'a OsStr, &'a OsStr), DirectoryCommitError<E>> {
    let staging_parent = staging.parent().ok_or(DirectoryCommitError::InvalidPaths)?;
    let canonical_parent = canonical
        .parent()
        .ok_or(DirectoryCommitError::InvalidPaths)?;
    let staging_name = staging
        .file_name()
        .ok_or(DirectoryCommitError::InvalidPaths)?;
    let canonical_name = canonical
        .file_name()
        .ok_or(DirectoryCommitError::InvalidPaths)?;
    if staging == canonical || staging_parent != canonical_parent {
        return Err(DirectoryCommitError::InvalidPaths);
    }
    Ok((staging_parent, staging_name, canonical_name))
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn exists_without_following(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn open_directory_at(parent: &impl AsFd, name: &OsStr) -> io::Result<fs::File> {
    let fd = unix_fs::openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io::Error::from)?;
    Ok(fs::File::from(fd))
}
