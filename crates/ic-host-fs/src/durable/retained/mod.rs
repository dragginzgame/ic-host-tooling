//! Caller-named staging retained across failed or interrupted production.
use super::NamedWriteError;
use std::{convert::Infallible, ffi::OsStr, fmt, fs::File, io, os::fd::BorrowedFd};

/// A retained write failed; an error never authorizes deleting or retrying its name.
#[derive(Debug)]
pub enum RetainedWriteError<E> {
    /// Admission, exclusive creation or the initial directory sync failed.
    /// A newly created entry can remain even when this phase fails.
    Create(io::Error),
    /// The producer failed; its original error and any partial entry are retained.
    Produce(E),
    /// Production completed but file durability is unconfirmed.
    SyncFile(io::Error),
    /// File synchronization completed but final directory durability is unconfirmed.
    SyncDirectory(io::Error),
}

impl<E> fmt::Display for RetainedWriteError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Create(_) => "retained file creation failed",
            Self::Produce(_) => "retained file production failed",
            Self::SyncFile(_) => "retained file synchronization failed",
            Self::SyncDirectory(_) => "retained directory synchronization failed",
        })
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RetainedWriteError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Create(source) | Self::SyncFile(source) | Self::SyncDirectory(source) => source,
            Self::Produce(source) => source,
        })
    }
}

/// Write one caller-named entry that must survive incomplete production.
///
/// The name is exclusively created and directory-synchronized before invoking
/// the producer, using [`create_retained_at`]. Success synchronizes the held
/// file and directory after production. Failure or panic never unlinks the entry;
/// partial contents can remain and are not promised durable on failure.
/// There is no staging rename, replacement, cleanup or automatic retry.
///
/// Callers own the admitted directory, single-component name, permissions, byte
/// budgets, schema and recovery decision. Exclude concurrent namespace writers,
/// keep the entry bound to its held file and finish other writable handles before
/// returning success. A visible name alone is not proof of successful production.
///
/// # Errors
/// Preserves creation, producer, file-sync and final directory-sync failures.
/// Invalid inputs and unsupported hosts refuse before the producer runs.
pub fn write_retained_at_with<T, E>(
    parent: BorrowedFd<'_>,
    name: &OsStr,
    permissions: u32,
    produce: impl FnOnce(&mut File) -> Result<T, E>,
) -> Result<T, RetainedWriteError<E>> {
    let mut file =
        create_retained_at(parent, name, permissions).map_err(RetainedWriteError::Create)?;
    let output = produce(&mut file).map_err(RetainedWriteError::Produce)?;
    file.sync_all().map_err(RetainedWriteError::SyncFile)?;
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    rustix::fs::fsync(parent).map_err(|error| RetainedWriteError::SyncDirectory(error.into()))?;
    Ok(output)
}

/// Exclusively create a named regular staging file beneath an admitted directory.
///
/// The caller selects a single-component name and permissions, owns directory
/// custody and excludes concurrent namespace writers. No parents are created.
/// Synchronizes the directory before returning the writable file. Dropping the
/// file, producer failure or interruption never removes the staging entry.
/// The caller owns retention, content verification and eventual cleanup.
///
/// # Errors
/// Invalid names/permissions, occupied entries and filesystem failures refuse.
/// A directory-sync failure can leave the newly created stage; inspect the
/// original name rather than automatically creating or retrying another stage.
pub fn create_retained_at(
    parent: BorrowedFd<'_>,
    name: &OsStr,
    permissions: u32,
) -> io::Result<File> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        use rustix::fs::{OFlags, fsync, openat};
        super::supported::validate_file_name(name)?;
        let permissions = super::supported::native_permissions(permissions)?;
        let file = File::from(openat(
            parent,
            name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            permissions,
        )?);
        fsync(parent)?;
        Ok(file)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (parent, name, permissions);
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

/// Synchronize and publish a verified retained stage without replacing a target.
///
/// The stage must still name the held regular file with exactly one link.
/// The caller verifies content before calling, finishes other writable handles,
/// retains exclusive directory custody and selects two single-component names.
/// No content policy, recovery decision or automatic retry is supplied here.
/// Uses the same no-replace publication engine as streamed durable writes.
/// Success consumes the original stage name and synchronizes the directory.
///
/// # Errors
/// Before-publication errors preserve staging without cleanup. An occupied
/// destination is never replaced. After-publication errors mean the target is
/// visible but final durability/cleanup is unconfirmed; reconcile before retrying.
/// The writable handle is consumed on success or failure; evidence remains named.
pub fn publish_retained_at(
    parent: BorrowedFd<'_>,
    stage: &OsStr,
    target: &OsStr,
    file: File,
) -> Result<(), NamedWriteError<Infallible>> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        use super::supported::{publish_create_new, validate_file_name, verify_staging};
        validate_file_name(stage).map_err(NamedWriteError::before)?;
        validate_file_name(target).map_err(NamedWriteError::before)?;
        verify_staging(&parent, None, stage, &file).map_err(NamedWriteError::before)?;
        file.sync_all().map_err(NamedWriteError::before)?;
        publish_create_new(&parent, stage, target, &file)?;
        drop(file);
        rustix::fs::fsync(parent).map_err(|source| NamedWriteError::AfterPublication {
            source: source.into(),
        })
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (parent, stage, target, file);
        Err(NamedWriteError::before(io::Error::from(
            io::ErrorKind::Unsupported,
        )))
    }
}

#[cfg(all(
    test,
    any(target_os = "linux", target_os = "android", target_vendor = "apple")
))]
mod tests;
