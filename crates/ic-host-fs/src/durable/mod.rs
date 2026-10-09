//! Durable filesystem publication and exclusive descriptor locks.
//!
//! Responsibility: own atomic durable regular-file publication for host consumers.
//! Does not own: domain schemas or transitions, path selection, ephemeral protocol files, open
//! command-result descriptors, backup persistence, or multi-file transactions.
//! Boundary: writes own sibling staging, publication, cleanup and filesystem syncs
//! behind replace and create-new modes. Reads belong to [`crate::read`].

mod lock_wait;
pub use lock_wait::lock_exclusive_with_wait;
mod named;
pub use named::{NamedWriteError, write_named_with};
mod stream;
#[cfg(unix)]
pub use stream::write_at_with;
pub use stream::{PublicationMode, WriteOptions, write_typed_with, write_validated_with};

#[cfg(test)]
mod tests;

use std::{fmt, fs, io, path::Path};

/// A no-follow regular-file lock could not be acquired.
#[derive(Debug)]
pub enum RegularFileLockError {
    /// The opened entry is not a regular file.
    NotRegular,
    /// The filesystem operation failed.
    Io(io::Error),
    #[cfg(windows)]
    /// This operation is unsupported on the selected host.
    UnsupportedPlatform,
}

impl fmt::Display for RegularFileLockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegular => formatter.write_str("lock target is not a regular file"),
            Self::Io(_) => formatter.write_str("regular file lock I/O failed"),
            #[cfg(windows)]
            Self::UnsupportedPlatform => formatter.write_str("regular file locking is unsupported"),
        }
    }
}

impl std::error::Error for RegularFileLockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::NotRegular => None,
            #[cfg(windows)]
            Self::UnsupportedPlatform => None,
        }
    }
}

/// Preserve native I/O identity or retain the typed admission cause at an I/O boundary.
impl From<RegularFileLockError> for io::Error {
    fn from(error: RegularFileLockError) -> Self {
        match error {
            RegularFileLockError::Io(source) => source,
            other @ RegularFileLockError::NotRegular => Self::other(other),
            #[cfg(windows)]
            other @ RegularFileLockError::UnsupportedPlatform => Self::other(other),
        }
    }
}

const REPLACE_OPTIONS: WriteOptions = WriteOptions {
    mode: PublicationMode::Replace,
    permissions: 0o666,
};
const CREATE_NEW_OPTIONS: WriteOptions = WriteOptions {
    mode: PublicationMode::CreateNew,
    permissions: 0o666,
};
const CREATE_PRIVATE_OPTIONS: WriteOptions = WriteOptions {
    mode: PublicationMode::CreateNew,
    permissions: 0o600,
};

/// Durably replace one file through atomic publication of complete bytes.
///
/// Missing parent directories are created and durably linked before the file
/// is published. Serialization must complete before calling this helper.
///
/// # Errors
/// Returns filesystem or sync failures, including sync failures after publication.
pub fn write_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    commit_bytes(path, bytes, REPLACE_OPTIONS)
}

/// Stream one complete file into durable atomic replacement without buffering its contents.
///
/// The producer writes only to the owned sibling staging file. Its return value
/// is returned after file synchronization, rename and parent synchronization.
/// If the producer returns an error, the previous destination is preserved and
/// removal of owned staging is attempted. Panics or process interruption can
/// leave staging behind.
/// Missing parents are created and synchronized through the same path as
/// [`write_bytes`]. Producers own encoding, input/output limits and source admission.
///
/// This follows caller-selected parent paths; it does not confine a root or
/// commit several files together. Do not change the staging file's identity or
/// retain writable descriptor clones beyond the callback.
///
/// # Errors
/// Returns producer, filesystem, sync or unsupported-host errors. A sync failure
/// after rename can leave the new complete file visible: reconcile before retrying.
pub fn write_with<T>(
    path: &Path,
    write: impl FnOnce(&mut fs::File) -> io::Result<T>,
) -> io::Result<T> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        supported::commit_with_writer_and_hook(path, REPLACE_OPTIONS, write, |_, _| Ok(()))
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (path, write);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "durable atomic file publication is unsupported",
        ))
    }
}

/// Durably create one file and its missing parent hierarchy without replacing
/// an existing destination.
///
/// # Errors
/// Returns filesystem or sync failures, including an existing destination or a
/// sync failure after publication.
pub fn create_new_bytes_with_parents(path: &Path, bytes: &[u8]) -> io::Result<()> {
    commit_bytes(path, bytes, CREATE_NEW_OPTIONS)
}

/// Create owner-only bytes atomically, without replacing an existing destination.
///
/// # Errors
/// Returns filesystem or sync failures, an existing destination, or unsupported
/// platform errors. Sync failure can occur after publication.
pub fn create_private_bytes_with_parents(path: &Path, bytes: &[u8]) -> io::Result<()> {
    commit_bytes(path, bytes, CREATE_PRIVATE_OPTIONS)
}

/// Open and exclusively lock one durable regular no-follow file.
///
/// The lock file and missing parent hierarchy are durably created first. The
/// returned descriptor owns the kernel lock and is close-on-exec.
///
/// # Errors
/// Returns typed file admission, creation, sync or lock acquisition failures.
pub fn lock_regular_file_with_parents(path: &Path) -> Result<fs::File, RegularFileLockError> {
    let file = open_regular_lock_file_with_parents(path)?;
    #[cfg(not(windows))]
    rustix::fs::flock(&file, rustix::fs::FlockOperation::LockExclusive)
        .map_err(errno_to_lock_error)?;
    Ok(file)
}

/// Open a durable regular no-follow lock file and attempt exclusive acquisition once.
///
/// Uses the same parent creation and file admission as
/// [`lock_regular_file_with_parents`], preserving existing file contents.
/// Contention returns [`RegularFileLockError::Io`] with [`io::ErrorKind::WouldBlock`]
/// immediately; there is no polling, retry or progress callback. The returned
/// close-on-exec descriptor holds the lock until unlocked or closed.
///
/// Callers own trusted parents, namespace stability and contention policy.
/// Nonblocking acquisition does not bound filesystem open, creation or sync latency.
///
/// # Errors
/// Returns typed admission, creation, sync or native lock errors, including contention.
pub fn try_lock_regular_file_with_parents(path: &Path) -> Result<fs::File, RegularFileLockError> {
    let file = open_regular_lock_file_with_parents(path)?;
    #[cfg(not(windows))]
    rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive)
        .map_err(errno_to_lock_error)?;
    Ok(file)
}

/// Acquire the same exclusive lock, reporting contention while preserving its lifetime.
///
/// Errors retain their admission category or original I/O cause, including a
/// failed progress callback. No callback or lock error is converted to text.
/// Consumers own diagnostic formatting and whether to retry.
///
/// # Errors
/// Returns file admission, creation, sync, lock acquisition or callback failures.
pub fn lock_file_with_progress(
    path: &Path,
    mut waiting: impl FnMut(&fs::File, std::time::Duration) -> io::Result<()>,
) -> Result<fs::File, RegularFileLockError> {
    let started = std::time::Instant::now();
    let file = open_regular_lock_file_with_parents(path)?;
    #[cfg(not(windows))]
    {
        let mut next_report = std::time::Duration::from_secs(1);
        lock_exclusive_with_wait(&file, std::time::Duration::from_millis(100), |_| {
            let elapsed = started.elapsed();
            if elapsed >= next_report {
                waiting(&file, elapsed)?;
                next_report = elapsed + std::time::Duration::from_secs(1);
            }
            Ok(())
        })
        .map_err(RegularFileLockError::Io)?;
    }
    Ok(file)
}

/// Open an unlocked regular no-follow lock file, durably creating missing parents
/// and the file first. Existing file bytes are preserved.
///
/// The returned read/write descriptor is close-on-exec. No lock is acquired:
/// callers choose shared/exclusive acquisition, wait timing, callbacks and unlock
/// lifetime. It composes with [`lock_exclusive_with_wait`]. Descriptor clones
/// share kernel lock ownership; explicit unlock and final-close policy remain
/// caller-owned.
///
/// Callers own trusted parent paths and namespace stability. Only the final
/// component is opened without following symlinks; this is not root confinement.
/// Admission does not bound filesystem creation, open or sync latency.
/// Existing entries are admitted without staging or syncing another file, so
/// their parent need not be writable. Missing entries use durable creation once;
/// a competing creator is admitted through the same checks. Removal during
/// admission can return a native not-found error rather than retrying indefinitely.
///
/// # Errors
/// Returns typed file admission, creation, sync or unsupported-host failures.
pub fn open_regular_lock_file_with_parents(path: &Path) -> Result<fs::File, RegularFileLockError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            match create_new_bytes_with_parents(path, &[]) {
                Ok(()) => {}
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => return Err(RegularFileLockError::Io(source)),
            }
            fs::symlink_metadata(path).map_err(RegularFileLockError::Io)?
        }
        Err(source) => return Err(RegularFileLockError::Io(source)),
    };
    if !metadata.file_type().is_file() {
        return Err(RegularFileLockError::NotRegular);
    }

    #[cfg(not(windows))]
    {
        use rustix::{
            fd::OwnedFd,
            fs::{FileType, Mode, OFlags, fstat, open},
        };

        // Path metadata selects creation and rejects known special entries; the
        // no-follow/nonblocking open and descriptor check still own admission
        // if the path changes between observations.
        let fd: OwnedFd = open(
            path,
            OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(errno_to_lock_error)?;
        let metadata = fstat(&fd).map_err(errno_to_lock_error)?;
        if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile {
            return Err(RegularFileLockError::NotRegular);
        }
        Ok(fs::File::from(fd))
    }

    #[cfg(windows)]
    {
        Err(RegularFileLockError::UnsupportedPlatform)
    }
}

#[cfg(not(windows))]
fn errno_to_lock_error(source: rustix::io::Errno) -> RegularFileLockError {
    RegularFileLockError::Io(io::Error::from_raw_os_error(source.raw_os_error()))
}

fn commit_bytes(path: &Path, bytes: &[u8], options: WriteOptions) -> io::Result<()> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        supported::commit_with_hook(path, bytes, options, |_, _| Ok(()))
    }

    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (path, bytes, options);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "durable atomic file publication is unsupported on platform {}",
                std::env::consts::OS
            ),
        ))
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
mod supported {
    use super::{NamedWriteError, PublicationMode, WriteOptions};

    use std::{
        ffi::{OsStr, OsString},
        fs,
        io::{self, Write},
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use rustix::{
        fd::{AsFd, BorrowedFd, OwnedFd},
        fs::{self as unix_fs, AtFlags, Mode, OFlags, RenameFlags},
    };

    const TEMP_ATTEMPTS: usize = 64;
    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum FileCommitStep {
        ParentDirectoryCreate,
        CreatedDirectorySync,
        CreatedDirectoryParentSync,
        TemporaryFileCreate,
        TemporaryFileWrite,
        TemporaryFileSync,
        Publication,
        FinalParentSync,
    }

    pub(super) fn commit_with_hook(
        path: &Path,
        bytes: &[u8],
        options: WriteOptions,
        before: impl FnMut(FileCommitStep, &Path) -> io::Result<()>,
    ) -> io::Result<()> {
        commit_with_writer_and_hook(path, options, |file| file.write_all(bytes), before)
    }

    pub(super) fn commit_with_writer_and_hook<T>(
        path: &Path,
        options: WriteOptions,
        write: impl FnOnce(&mut fs::File) -> io::Result<T>,
        before: impl FnMut(FileCommitStep, &Path) -> io::Result<()>,
    ) -> io::Result<T> {
        commit_path_with_options(
            path,
            options,
            |_, file| write(file),
            None::<fn(&Path, &T) -> io::Result<()>>,
            before,
        )
        .map_err(NamedWriteError::into_io)
    }

    pub(super) fn commit_path_with_options<T, E>(
        path: &Path,
        options: WriteOptions,
        produce: impl FnOnce(&Path, &mut fs::File) -> Result<T, E>,
        admit: Option<impl FnOnce(&Path, &T) -> Result<(), E>>,
        mut before: impl FnMut(FileCommitStep, &Path) -> io::Result<()>,
    ) -> Result<T, NamedWriteError<E>> {
        validate_permissions(options.permissions).map_err(NamedWriteError::before)?;
        let (parent, file_name) = split_target(path).map_err(NamedWriteError::before)?;
        create_parent_hierarchy(parent, &mut before).map_err(NamedWriteError::before)?;
        let parent_fd = open_directory(parent).map_err(NamedWriteError::before)?;
        commit_at_with_hook(
            parent_fd.as_fd(),
            Some(parent),
            file_name,
            options,
            produce,
            admit,
            before,
        )
    }

    pub(super) fn commit_at_with_hook<T, E>(
        parent_fd: BorrowedFd<'_>,
        parent_path: Option<&Path>,
        file_name: &OsStr,
        options: WriteOptions,
        produce: impl FnOnce(&Path, &mut fs::File) -> Result<T, E>,
        admit: Option<impl FnOnce(&Path, &T) -> Result<(), E>>,
        mut before: impl FnMut(FileCommitStep, &Path) -> io::Result<()>,
    ) -> Result<T, NamedWriteError<E>> {
        validate_permissions(options.permissions).map_err(NamedWriteError::before)?;
        validate_file_name(file_name).map_err(NamedWriteError::before)?;
        let metadata = unix_fs::fstat(parent_fd)
            .map_err(|error| NamedWriteError::before(errno_to_io(error)))?;
        if unix_fs::FileType::from_raw_mode(metadata.st_mode) != unix_fs::FileType::Directory {
            return Err(NamedWriteError::before(io::Error::from(
                io::ErrorKind::NotADirectory,
            )));
        }
        // Descriptor-only calls use relative names solely for hook diagnostics.
        // All filesystem operations below use the held directory descriptor.
        let parent = parent_path.unwrap_or_else(|| Path::new(""));
        let path = parent.join(file_name);
        let permissions =
            native_permissions(options.permissions).map_err(NamedWriteError::before)?;
        let (temp_name, temp_path, mut temp_file) =
            create_sibling_temp(&parent_fd, parent, file_name, permissions, &mut before)
                .map_err(NamedWriteError::before)?;

        let cleanup = || remove_owned_temp(&parent_fd, &temp_name, &temp_file).err();
        if let Err(source) = before(FileCommitStep::TemporaryFileWrite, &temp_path) {
            return Err(NamedWriteError::BeforePublication {
                source,
                cleanup_error: cleanup(),
            });
        }
        let value = match produce(&temp_path, &mut temp_file) {
            Ok(value) => value,
            Err(source) => {
                return Err(NamedWriteError::Producer {
                    source,
                    cleanup_error: remove_owned_temp(&parent_fd, &temp_name, &temp_file).err(),
                });
            }
        };

        let staged = (|| {
            before(FileCommitStep::TemporaryFileSync, &temp_path)?;
            verify_staging(&parent_fd, parent_path, &temp_name, &temp_file)?;
            temp_file.sync_all()
        })();
        if let Err(source) = staged {
            return Err(NamedWriteError::BeforePublication {
                source,
                cleanup_error: remove_owned_temp(&parent_fd, &temp_name, &temp_file).err(),
            });
        }
        if let Some(admit) = admit {
            match retain_read_only(&parent_fd, parent_path, &temp_name, &temp_file) {
                Ok(read_only) => drop(std::mem::replace(&mut temp_file, read_only)),
                Err(source) => {
                    return Err(NamedWriteError::BeforePublication {
                        source,
                        cleanup_error: remove_owned_temp(&parent_fd, &temp_name, &temp_file).err(),
                    });
                }
            }
            if let Err(source) = admit(&temp_path, &value) {
                return Err(NamedWriteError::Producer {
                    source,
                    cleanup_error: remove_owned_temp(&parent_fd, &temp_name, &temp_file).err(),
                });
            }
        }
        let staged = (|| {
            before(FileCommitStep::Publication, &path)?;
            // Keep the descriptor alive through rename so its identity cannot
            // be recycled after a producer unlinks or replaces the pathname.
            verify_staging(&parent_fd, parent_path, &temp_name, &temp_file)
        })();
        if let Err(source) = staged {
            return Err(NamedWriteError::BeforePublication {
                source,
                cleanup_error: remove_owned_temp(&parent_fd, &temp_name, &temp_file).err(),
            });
        }
        let published = match options.mode {
            PublicationMode::Replace => {
                unix_fs::renameat(parent_fd, &temp_name, parent_fd, file_name)
                    .map_err(|error| NamedWriteError::before(errno_to_io(error)))
            }
            PublicationMode::CreateNew => {
                publish_create_new(&parent_fd, &temp_name, file_name, &temp_file)
            }
        };
        if let Err(NamedWriteError::BeforePublication { source, .. }) = published {
            return Err(NamedWriteError::BeforePublication {
                source,
                cleanup_error: remove_owned_temp(&parent_fd, &temp_name, &temp_file).err(),
            });
        }
        published?;

        before(FileCommitStep::FinalParentSync, parent)
            .map_err(|source| NamedWriteError::AfterPublication { source })?;
        unix_fs::fsync(parent_fd).map_err(|error| NamedWriteError::AfterPublication {
            source: errno_to_io(error),
        })?;
        Ok(value)
    }

    fn native_permissions(permissions: u32) -> io::Result<Mode> {
        validate_permissions(permissions)?;
        // Darwin's mode_t is u16; retain checked narrowing after admission.
        #[cfg(target_vendor = "apple")]
        let permissions = u16::try_from(permissions).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "invalid file permission bits")
        })?;
        Ok(Mode::from_raw_mode(permissions))
    }

    fn retain_read_only(
        parent_fd: &impl AsFd,
        parent_path: Option<&Path>,
        name: &OsStr,
        original: &fs::File,
    ) -> io::Result<fs::File> {
        let read_only = fs::File::from(
            unix_fs::openat(
                parent_fd,
                name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(errno_to_io)?,
        );
        // Both descriptors remain open until identity is verified, preventing reuse.
        let held = unix_fs::fstat(original).map_err(errno_to_io)?;
        let reopened = unix_fs::fstat(&read_only).map_err(errno_to_io)?;
        if !same_identity(&held, &reopened) {
            return Err(staging_identity_error());
        }
        verify_staging(parent_fd, parent_path, name, &read_only)?;
        Ok(read_only)
    }

    fn same_identity(left: &unix_fs::Stat, right: &unix_fs::Stat) -> bool {
        left.st_dev == right.st_dev
            && left.st_ino == right.st_ino
            && unix_fs::FileType::from_raw_mode(left.st_mode)
                == unix_fs::FileType::from_raw_mode(right.st_mode)
    }

    fn staging_identity_error() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "staging or parent identity changed",
        )
    }

    fn verify_staging(
        parent_fd: &impl AsFd,
        parent: Option<&Path>,
        name: &OsStr,
        file: &fs::File,
    ) -> io::Result<()> {
        if let Some(parent) = parent {
            let held_parent = unix_fs::fstat(parent_fd).map_err(errno_to_io)?;
            let named_parent = unix_fs::statat(unix_fs::CWD, parent, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(errno_to_io)?;
            if !same_identity(&held_parent, &named_parent) {
                return Err(staging_identity_error());
            }
        }
        let held = unix_fs::fstat(file).map_err(errno_to_io)?;
        let named =
            unix_fs::statat(parent_fd, name, AtFlags::SYMLINK_NOFOLLOW).map_err(errno_to_io)?;
        if !same_identity(&held, &named)
            || unix_fs::FileType::from_raw_mode(held.st_mode) != unix_fs::FileType::RegularFile
            || held.st_nlink != 1
            || named.st_nlink != 1
        {
            return Err(staging_identity_error());
        }
        Ok(())
    }

    fn remove_owned_temp(parent_fd: &impl AsFd, name: &OsStr, file: &fs::File) -> io::Result<()> {
        let named = match unix_fs::statat(parent_fd, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(named) => named,
            Err(rustix::io::Errno::NOENT) => return Ok(()),
            Err(error) => return Err(errno_to_io(error)),
        };
        let held = unix_fs::fstat(file).map_err(errno_to_io)?;
        if !same_identity(&held, &named) {
            return Err(staging_identity_error());
        }
        unix_fs::unlinkat(parent_fd, name, AtFlags::empty()).map_err(errno_to_io)
    }

    fn publish_create_new<E>(
        parent_fd: &impl AsFd,
        temp_name: &OsStr,
        file_name: &OsStr,
        temp_file: &fs::File,
    ) -> Result<(), NamedWriteError<E>> {
        let renamed = unix_fs::renameat_with(
            parent_fd,
            temp_name,
            parent_fd,
            file_name,
            RenameFlags::NOREPLACE,
        );
        finish_create_new_publication(parent_fd, temp_name, file_name, renamed, || {
            remove_owned_temp(parent_fd, temp_name, temp_file)
        })
    }

    pub(super) fn finish_create_new_publication<E>(
        parent_fd: &impl AsFd,
        temp_name: &OsStr,
        file_name: &OsStr,
        renamed: rustix::io::Result<()>,
        cleanup: impl FnOnce() -> io::Result<()>,
    ) -> Result<(), NamedWriteError<E>> {
        match renamed {
            Ok(()) => Ok(()),
            Err(
                rustix::io::Errno::INVAL | rustix::io::Errno::NOSYS | rustix::io::Errno::OPNOTSUPP,
            ) => {
                unix_fs::linkat(parent_fd, temp_name, parent_fd, file_name, AtFlags::empty())
                    .map_err(|error| NamedWriteError::before(errno_to_io(error)))?;
                // Publication already happened. Never classify an unlink failure
                // as safe to retry, or silently retain the extra staging link.
                cleanup().map_err(|source| NamedWriteError::AfterPublication { source })
            }
            Err(error) => Err(NamedWriteError::before(errno_to_io(error))),
        }
    }

    #[cfg(test)]
    pub(super) fn publish_create_new_after_error(
        parent: &Path,
        temp_name: &OsStr,
        file_name: &OsStr,
        error: rustix::io::Errno,
    ) -> io::Result<()> {
        let parent_fd = open_directory(parent)?;
        finish_create_new_publication(&parent_fd, temp_name, file_name, Err(error), || {
            remove_temp(&parent_fd, temp_name)
        })
        .map_err(NamedWriteError::into_io)
    }

    fn split_target(path: &Path) -> io::Result<(&Path, &OsStr)> {
        let file_name = path.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("durable write target has no file name: {}", path.display()),
            )
        })?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        Ok((parent, file_name))
    }

    fn validate_permissions(permissions: u32) -> io::Result<()> {
        if permissions & !0o777 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid file permission bits",
            ));
        }
        Ok(())
    }

    fn validate_file_name(name: &OsStr) -> io::Result<()> {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name
                .as_encoded_bytes()
                .iter()
                .any(|byte| matches!(byte, b'/' | 0))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected one filename component",
            ));
        }
        Ok(())
    }

    fn create_parent_hierarchy(
        parent: &Path,
        before: &mut impl FnMut(FileCommitStep, &Path) -> io::Result<()>,
    ) -> io::Result<()> {
        let mut missing = Vec::new();
        let mut current = parent;
        loop {
            match fs::symlink_metadata(current) {
                Ok(metadata) if metadata.is_dir() => break,
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotADirectory,
                        format!("output parent is not a directory: {}", current.display()),
                    ));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    missing.push(current.to_path_buf());
                    current = current
                        .parent()
                        .filter(|ancestor| !ancestor.as_os_str().is_empty())
                        .unwrap_or_else(|| Path::new("."));
                }
                Err(error) => return Err(error),
            }
        }

        for directory in missing.into_iter().rev() {
            before(FileCommitStep::ParentDirectoryCreate, &directory)?;
            match fs::create_dir(&directory) {
                Ok(()) => sync_created_directory(&directory, before)?,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    if !fs::symlink_metadata(&directory)?.is_dir() {
                        return Err(io::Error::new(
                            io::ErrorKind::NotADirectory,
                            format!("output parent is not a directory: {}", directory.display()),
                        ));
                    }
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn sync_created_directory(
        directory: &Path,
        before: &mut impl FnMut(FileCommitStep, &Path) -> io::Result<()>,
    ) -> io::Result<()> {
        before(FileCommitStep::CreatedDirectorySync, directory)?;
        let directory_fd = open_directory(directory)?;
        unix_fs::fsync(&directory_fd).map_err(errno_to_io)?;

        let owner = directory
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        before(FileCommitStep::CreatedDirectoryParentSync, owner)?;
        let owner_fd = open_directory(owner)?;
        unix_fs::fsync(&owner_fd).map_err(errno_to_io)
    }

    fn create_sibling_temp(
        parent_fd: &impl AsFd,
        parent: &Path,
        file_name: &OsStr,
        permissions: Mode,
        before: &mut impl FnMut(FileCommitStep, &Path) -> io::Result<()>,
    ) -> io::Result<(OsString, PathBuf, fs::File)> {
        for _ in 0..TEMP_ATTEMPTS {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let name = format!(".ic-host-tmp-{}-{sequence}", std::process::id());
            // Short names support NAME_MAX destinations. Never stage directly
            // into the selected output, including on case-insensitive hosts.
            if file_name
                .as_encoded_bytes()
                .eq_ignore_ascii_case(name.as_bytes())
            {
                continue;
            }
            let temp_name = OsString::from(name);
            let temp_path = parent.join(&temp_name);
            before(FileCommitStep::TemporaryFileCreate, &temp_path)?;
            match unix_fs::openat(
                parent_fd,
                &temp_name,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC,
                permissions,
            ) {
                Ok(file) => return Ok((temp_name, temp_path, fs::File::from(file))),
                Err(error) if error == rustix::io::Errno::EXIST => {}
                Err(error) => return Err(errno_to_io(error)),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "could not allocate a unique sibling temporary file for {}",
                parent.join(file_name).display()
            ),
        ))
    }

    fn open_directory(path: &Path) -> io::Result<OwnedFd> {
        unix_fs::open(
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(errno_to_io)
    }

    #[cfg(test)]
    fn remove_temp(parent_fd: &impl AsFd, temp_name: &OsStr) -> io::Result<()> {
        unix_fs::unlinkat(parent_fd, temp_name, AtFlags::empty()).map_err(errno_to_io)
    }

    fn errno_to_io(error: rustix::io::Errno) -> io::Error {
        io::Error::from_raw_os_error(error.raw_os_error())
    }
}
