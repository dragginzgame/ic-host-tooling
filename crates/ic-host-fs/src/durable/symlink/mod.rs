//! Admitted symbolic-link selection of caller-owned immutable artifacts.
use super::NamedWriteError;
use crate::identity::FileIdentity;
use std::{fs, io, path::Path};

/// Admit an absolute target, then atomically replace a selection with its symlink.
///
/// The caller supplies an unused staging name and owns both parent namespaces,
/// the immutable target and its durability. This function creates no directories,
/// downloads nothing and never deletes the target. Stage and destination must be
/// on the same filesystem. The callback receives the exact absolute target and
/// must not mutate either namespace. Concurrent namespace writers are excluded
/// by the caller; pathname observations do not establish custody.
/// Panics, interruption or failure to observe the newly created staging link can
/// leave staging behind; callers retain reconciliation of that unused name.
///
/// # Errors
/// Returns admission, cleanup or filesystem failures with publication phase.
/// After-publication failures leave the selection visible: reconcile before retry.
pub fn publish_symlink_with<E>(
    target: &Path,
    stage: &Path,
    destination: &Path,
    admit: impl FnOnce(&Path) -> Result<(), E>,
) -> Result<(), NamedWriteError<E>> {
    publish_with_sync(target, stage, destination, admit, fs::File::sync_all)
}

fn publish_with_sync<E>(
    target: &Path,
    stage: &Path,
    destination: &Path,
    admit: impl FnOnce(&Path) -> Result<(), E>,
    mut sync: impl FnMut(&fs::File) -> io::Result<()>,
) -> Result<(), NamedWriteError<E>> {
    if !target.is_absolute() || stage == destination {
        return Err(NamedWriteError::before(io::ErrorKind::InvalidInput.into()));
    }
    let stage_parent = stage
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let destination_parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stage_directory = fs::File::open(stage_parent).map_err(NamedWriteError::before)?;
    let destination_directory =
        fs::File::open(destination_parent).map_err(NamedWriteError::before)?;
    std::os::unix::fs::symlink(target, stage).map_err(NamedWriteError::before)?;
    let identity =
        FileIdentity::from_metadata(&fs::symlink_metadata(stage).map_err(NamedWriteError::before)?);
    let verify = || -> io::Result<()> {
        let current = fs::symlink_metadata(stage)?;
        if !current.file_type().is_symlink()
            || FileIdentity::from_metadata(&current) != identity
            || fs::read_link(stage)? != target
        {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(())
    };
    let cleanup = || verify().and_then(|()| fs::remove_file(stage)).err();
    if let Err(source) = admit(target) {
        return Err(NamedWriteError::Producer {
            source,
            cleanup_error: cleanup(),
        });
    }
    let mut publish = || -> io::Result<()> {
        verify()?;
        sync(&stage_directory)?;
        fs::rename(stage, destination)
    };
    if let Err(source) = publish() {
        return Err(NamedWriteError::BeforePublication {
            source,
            cleanup_error: cleanup(),
        });
    }
    for directory in [stage_directory, destination_directory] {
        sync(&directory).map_err(|source| NamedWriteError::AfterPublication { source })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
