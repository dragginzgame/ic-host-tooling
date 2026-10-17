//! Typed streamed publication through the existing durable engine.

use super::NamedWriteError;
use std::{fs, path::Path};

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
use super::supported::WriterCustody;

/// How complete staging becomes the selected directory entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationMode {
    /// Atomically replace the selected entry without following its final symlink.
    Replace,
    /// Publish only if the selected entry is absent, including at the final race.
    CreateNew,
}

/// Caller-selected publication and staging permissions; no policy defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriteOptions {
    /// Whether an existing destination can be replaced.
    pub mode: PublicationMode,
    /// Unix permission bits for the created file, reduced by the process umask.
    ///
    /// Only `0o000..=0o777` is accepted. Use `0o600` for owner-only staging.
    /// Existing target permissions are not copied; the producer may further
    /// restrict the held file's permissions before successful completion.
    pub permissions: u32,
}

/// Stream a typed producer into durable publication at a caller-selected path.
///
/// Owns parent creation, exclusive staging, file synchronization,
/// publication and parent synchronization for pathname writes. The
/// callback may serialize through a bounded writer and retain its own error
/// type, without buffering a complete document or converting that error to I/O.
/// Missing parents are created with the existing ordinary directory semantics;
/// owner-only file permissions do not establish private parent custody.
///
/// Callers own input admission, output limits, target policy and parent hierarchy
/// control. Exclude concurrent namespace writers, keep the stage a regular file
/// with one link and finish all writable handles before returning success.
/// Panics/interruption may retain staging; publication is not a multi-file
/// transaction, sandbox or retry mechanism.
///
/// # Errors
/// Retains the original producer error and separate cleanup failures in the
/// shared [`NamedWriteError`]. Before/after-publication filesystem failures stay
/// distinguishable. Invalid permissions and unsupported hosts fail before the
/// producer is called. Paths containing NUL and directory-required targets ending
/// in `/` or `/.` are rejected before parent creation or staging. Reconcile
/// after-publication failures before retrying.
pub fn write_with<T, E>(
    path: &Path,
    options: WriteOptions,
    produce: impl FnOnce(&mut fs::File) -> Result<T, E>,
) -> Result<T, NamedWriteError<E>> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        super::supported::commit_path_with_options(
            path,
            options,
            |_, file| produce(file),
            (WriterCustody::Writable, |_, _| Ok(())),
            |_, _| Ok(()),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (path, options, produce);
        Err(NamedWriteError::before(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "durable atomic file publication is unsupported",
        )))
    }
}

/// Stream through an existing parent, admitting the result after file sync.
///
/// Missing parents are never created. Opens the selected parent once as a
/// directory, following parent aliases; all staging/publication then uses its held
/// descriptor, even after a rename. This is pathname convenience, not root
/// confinement or private-parent admission. The final entry uses the selected
/// publication mode and is never followed.
///
/// Shares [`write_at_validated_with`]'s producer, permissions, custody, admission,
/// cleanup and before/after-publication contracts. Admission runs after file sync
/// and before publication. Use `|_| Ok(())` when no additional admission is needed.
/// Callers retain concurrency, cancellation, validation and retry policy.
///
/// # Errors
/// Missing/special-file parents and invalid paths/permissions refuse before
/// production. Retains typed producer/admission and independent cleanup failures;
/// filesystem syscall latency remains unbounded. Reconcile published failures.
pub fn write_existing_parent_validated_with<T, E>(
    path: &Path,
    options: WriteOptions,
    produce: impl FnOnce(&mut fs::File) -> Result<T, E>,
    admit: impl FnOnce(&T) -> Result<(), E>,
) -> Result<T, NamedWriteError<E>> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        use rustix::fs::{Mode, OFlags, open};
        use std::os::fd::AsFd as _;
        let permissions = super::supported::native_permissions(options.permissions)
            .map_err(NamedWriteError::before)?;
        let (parent, name) =
            super::supported::split_target(path).map_err(NamedWriteError::before)?;
        let parent = open(
            parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|source| NamedWriteError::before(source.into()))?;
        super::supported::commit_at_with_hook(
            parent.as_fd(),
            None,
            name,
            (options.mode, permissions),
            |_, file| produce(file),
            (WriterCustody::Writable, |_, value| admit(value)),
            |_, _| Ok(()),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (path, options, produce, admit);
        Err(NamedWriteError::before(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "durable atomic file publication is unsupported",
        )))
    }
}

/// Stream beneath an admitted directory, then admit the result before publication.
///
/// `name` is one nonempty filename component excluding dot, dot-dot, slash and
/// NUL. The held parent descriptor must name a directory open for synchronization.
/// Staging, identity checks, cleanup, publication and parent sync remain relative
/// to it across namespace renames. No parent is created or pathname reopened.
///
/// Caller-selected options control replace/create-only publication and permissions.
/// Replace affects the final entry itself, including symlinks; create-only refuses
/// an occupied name. This is neither root confinement nor a compare-and-swap.
/// After production and file sync, `admit` receives the result and may reject it
/// while the old destination remains untouched. Use `|_| Ok(())` for no additional
/// admission. It must not mutate staging or issue publication effects.
///
/// Callers retain directory/namespace custody, target policy, output limits,
/// cancellation and retry policy. Finish all writable handles/external producers
/// before returning. The callback/interruption obligations of [`write_with`] apply.
///
/// # Errors
/// Invalid names/permissions or non-directory descriptors refuse before production.
/// Filesystem-specific name encoding may still refuse at publication. Retains
/// typed producer/admission, cleanup and before/after-publication failures;
/// reconcile already-published results before retrying.
#[cfg(unix)]
pub fn write_at_validated_with<T, E>(
    parent: std::os::fd::BorrowedFd<'_>,
    name: &std::ffi::OsStr,
    options: WriteOptions,
    produce: impl FnOnce(&mut fs::File) -> Result<T, E>,
    admit: impl FnOnce(&T) -> Result<(), E>,
) -> Result<T, NamedWriteError<E>> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        let permissions = super::supported::native_permissions(options.permissions)
            .map_err(NamedWriteError::before)?;
        super::supported::commit_at_with_hook(
            parent,
            None,
            name,
            (options.mode, permissions),
            |_, file| produce(file),
            (WriterCustody::Writable, |_, value| admit(value)),
            |_, _| Ok(()),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (parent, name, options, produce, admit);
        Err(NamedWriteError::before(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "durable atomic file publication is unsupported",
        )))
    }
}

/// Stream, synchronize and close a writer before admitting its staged pathname.
///
/// Shares [`write_with`]'s engine, permissions and publication modes. After
/// successful production and file sync, a verified read-only descriptor retains
/// inode custody while the library's writable descriptor is closed. `admit` then
/// receives the absolute staged path and the producer's value before publication.
/// This allows executable admission on hosts that reject executing a writable inode.
///
/// The producer must close all writable clones and finish external writers before
/// returning. Admission must not modify the file and must finish its child processes
/// before returning. The library does not choose executable versions, arguments,
/// environment, byte limits or retries. Read permission is required when reopening
/// the stage; execute permission is caller-selected. All parent-custody and
/// interruption obligations of [`write_with`] apply. A read-only descriptor
/// is not a sandbox or a guarantee against concurrent content modification.
///
/// # Errors
/// Producer/admission failures retain `E` in [`NamedWriteError::Producer`].
/// Reopening, identity, sync and publication failures retain their filesystem
/// phase. Admission rejection preserves the prior destination and attempts
/// owned-entry cleanup. Reconcile after-publication failures before retrying.
pub fn write_validated_with<T, E>(
    path: &Path,
    options: WriteOptions,
    produce: impl FnOnce(&mut fs::File) -> Result<T, E>,
    admit: impl FnOnce(&Path, &T) -> Result<(), E>,
) -> Result<T, NamedWriteError<E>> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        super::supported::validate_target_path(path).map_err(NamedWriteError::before)?;
        let absolute = std::path::absolute(path).map_err(NamedWriteError::before)?;
        super::supported::commit_path_with_options(
            &absolute,
            options,
            |_, file| produce(file),
            (WriterCustody::ReadOnly, admit),
            |_, _| Ok(()),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (path, options, produce, admit);
        Err(NamedWriteError::before(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "durable atomic file publication is unsupported",
        )))
    }
}

#[cfg(all(
    test,
    any(target_os = "linux", target_os = "android", target_vendor = "apple")
))]
mod tests;
