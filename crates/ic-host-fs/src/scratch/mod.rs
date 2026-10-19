//! Private retained and scoped invocation files without durable publication.

#[cfg(all(unix, feature = "temporary-file"))]
mod temporary;
#[cfg(all(unix, feature = "temporary-file"))]
pub use temporary::TemporaryFile;

#[cfg(unix)]
use std::{
    fs::{self, File, OpenOptions},
    io::Write as _,
    os::unix::fs::OpenOptionsExt as _,
    sync::atomic::{AtomicU64, Ordering},
};
use std::{
    io,
    path::{Path, PathBuf},
};

#[cfg(all(test, unix))]
mod tests;

#[cfg(unix)]
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Private scratch creation failed, retaining the original IO and cleanup causes.
#[derive(Debug, thiserror::Error)]
pub enum PrivateBytesError {
    /// Validation, parent resolution or opening failed before creating a file.
    #[error("private scratch creation failed")]
    BeforeCreation(#[source] io::Error),
    /// A newly created file could not be written completely.
    #[error("private scratch write{} failed", if .cleanup.is_some() { " and cleanup" } else { "" })]
    Write {
        /// Absolute candidate path for caller-owned recovery if cleanup failed.
        path: PathBuf,
        /// Original write failure, which is also this error's source.
        source: io::Error,
        /// Failed removal after closing the file; `None` means removal succeeded.
        cleanup: Option<io::Error>,
    },
}

/// Create a complete, closed private invocation file under an existing parent.
///
/// The caller selects the bytes, their budget, filename prefix and nonzero maximum
/// creation attempts. The prefix is one normal filename component without NUL,
/// CR or LF. Parent aliases are resolved before allocation, making the result
/// absolute. Each attempt uses create-new and Unix mode 0600 subject to umask;
/// occupied files, symlinks and directories are preserved and consume one attempt.
/// Only an occupied, uncreated candidate is retried. Writes are never retried by
/// allocating another file. The file is closed before returning or removing a
/// failed partial write; both write and cleanup failures remain inspectable.
///
/// This performs no disk synchronization. Successful files have no automatic
/// cleanup: callers own their namespace, retention and remote-effect recovery.
/// They must exclude concurrent replacement of the parent and candidate names;
/// canonicalizing a parent is not hostile-directory confinement. Platforms
/// without the Unix privacy contract fail before creation.
///
/// # Errors
/// Returns [`PrivateBytesError::BeforeCreation`] for invalid requests, unsupported
/// platforms, parent/open failures and exhausted collisions. A partial write
/// returns [`PrivateBytesError::Write`], including any failed removal.
pub fn create_private_bytes(
    parent: &Path,
    prefix: &str,
    bytes: &[u8],
    attempts: usize,
) -> Result<PathBuf, PrivateBytesError> {
    #[cfg(unix)]
    {
        create_with_sequence(parent, prefix, bytes, attempts, || {
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        })
    }
    #[cfg(not(unix))]
    {
        let _ = (parent, prefix, bytes, attempts);
        Err(PrivateBytesError::BeforeCreation(io::Error::new(
            io::ErrorKind::Unsupported,
            "private scratch files require Unix permissions",
        )))
    }
}

#[cfg(unix)]
fn create_with_sequence(
    parent: &Path,
    prefix: &str,
    bytes: &[u8],
    attempts: usize,
    mut sequence: impl FnMut() -> u64,
) -> Result<PathBuf, PrivateBytesError> {
    if attempts == 0 {
        return Err(PrivateBytesError::BeforeCreation(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid private scratch request",
        )));
    }
    validate_prefix(prefix).map_err(PrivateBytesError::BeforeCreation)?;
    let parent = parent
        .canonicalize()
        .map_err(PrivateBytesError::BeforeCreation)?;
    for _ in 0..attempts {
        let path = parent.join(format!("{prefix}-{}-{}", std::process::id(), sequence()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => {
                return finish_candidate(
                    file,
                    path,
                    |file| file.write_all(bytes),
                    |path| fs::remove_file(path),
                );
            }
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(PrivateBytesError::BeforeCreation(source)),
        }
    }
    Err(PrivateBytesError::BeforeCreation(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "private scratch creation attempts exhausted",
    )))
}

#[cfg(unix)]
fn validate_prefix(prefix: &str) -> io::Result<()> {
    use std::path::Component;
    let mut components = Path::new(prefix).components();
    if prefix.contains(['\0', '\r', '\n'])
        || !matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(name)), None) if name == std::ffi::OsStr::new(prefix)
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid private scratch request",
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn finish_candidate(
    mut file: File,
    path: PathBuf,
    write: impl FnOnce(&mut File) -> io::Result<()>,
    remove: impl FnOnce(&Path) -> io::Result<()>,
) -> Result<PathBuf, PrivateBytesError> {
    let result = write(&mut file);
    drop(file);
    match result {
        Ok(()) => Ok(path),
        Err(source) => {
            let cleanup = remove(&path).err();
            Err(PrivateBytesError::Write {
                path,
                source,
                cleanup,
            })
        }
    }
}
