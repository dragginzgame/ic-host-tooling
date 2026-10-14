//! Caller-named staging retained across failed or interrupted production.
use super::NamedWriteError;
use std::{convert::Infallible, ffi::OsStr, fs::File, io, os::fd::BorrowedFd};

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
