//! Fresh directory allocation without retention, cleanup or durability policy.

use std::{
    fs, io,
    num::NonZeroUsize,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Create a distinct child beneath an existing caller-owned parent.
///
/// `prefix` must be one normal filename component without CR/LF. Names append
/// the process ID and a sequence. Existing entries are skipped, never reused or
/// removed. Parent aliases remain in the returned path; no parent is created.
/// Unix permissions are reduced by umask; other hosts use native permissions.
/// `attempts` bounds collision attempts; `None` explicitly permits unlimited
/// collisions. Other errors return immediately. This is not an elapsed deadline.
/// Callers retain namespace custody, retention and cleanup. Allocation does not
/// establish hostile-directory confinement or synchronize directory contents.
///
/// # Errors
/// Returns invalid prefix/permissions, directory creation errors, or
/// `AlreadyExists` after exhausting the caller-selected collision allowance.
pub fn create_unique_directory(
    parent: &Path,
    prefix: &str,
    permissions: u32,
    attempts: Option<NonZeroUsize>,
) -> io::Result<PathBuf> {
    create_unique_directory_with_sequence(parent, prefix, permissions, attempts, &SEQUENCE)
}

fn create_unique_directory_with_sequence(
    parent: &Path,
    prefix: &str,
    permissions: u32,
    attempts: Option<NonZeroUsize>,
    sequence: &AtomicU64,
) -> io::Result<PathBuf> {
    let mut components = Path::new(prefix).components();
    if prefix.contains(['\r', '\n'])
        || !matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(name)), None) if name == std::ffi::OsStr::new(prefix)
        )
        || permissions & !0o777 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid directory prefix or permissions",
        ));
    }
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(permissions);
    }
    let mut remaining = attempts.map(NonZeroUsize::get);
    while remaining.is_none_or(|count| count > 0) {
        if let Some(count) = &mut remaining {
            *count -= 1;
        }
        let candidate = parent.join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            sequence.fetch_add(1, Ordering::Relaxed),
        ));
        match builder.create(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("failed to allocate directory beneath {}", parent.display()),
    ))
}

#[cfg(test)]
mod tests;
