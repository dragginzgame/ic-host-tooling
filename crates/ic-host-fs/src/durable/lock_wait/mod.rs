//! Regular descriptor locking, release and wait observations.

#[cfg(unix)]
use std::time::Instant;
use std::{fs::File, io, time::Duration};

#[cfg(all(test, unix))]
mod tests;

/// An owned shared descriptor lock released explicitly before its file closes.
///
/// Duplicated or inherited descriptors must not extend this guard's lock lifetime.
/// Independent acquisitions remain independent. Callers own any shared guard
/// lifetime, paths, namespace custody and cleanup policy; dropping it removes no file.
#[derive(Debug)]
pub struct SharedFileLock(File);

impl SharedFileLock {
    /// Acquire a blocking shared lock on an already-opened regular file.
    ///
    /// This consumes the descriptor without reopening it or imposing a deadline.
    ///
    /// # Errors
    /// Returns the original regular-descriptor admission or native lock error.
    pub fn acquire(file: File) -> io::Result<Self> {
        lock_shared(&file)?;
        Ok(Self(file))
    }

    /// Borrow the locked descriptor; clones do not extend the guard's lifetime.
    ///
    /// Callers must coordinate explicit unlock or lock-mode changes through this
    /// descriptor or its duplicates with their own guard lifetime contract.
    #[must_use]
    pub const fn file(&self) -> &File {
        &self.0
    }
}

impl Drop for SharedFileLock {
    fn drop(&mut self) {
        // Closing alone can leave a flock held by an inherited/duplicated
        // descriptor. Release at the selected ownership boundary instead.
        // On unlock failure, closing the owned file remains the fallback.
        let _ = unlock(&self.0);
    }
}

/// Acquire a blocking shared lock on a caller-owned regular descriptor.
///
/// Independent shared holders coexist. Opening, namespace custody, deadlines,
/// descriptor clones and final unlock/close policy remain caller-owned.
/// This does not reopen the file or impose an elapsed-time deadline.
///
/// # Errors
/// Returns metadata, non-regular descriptor, native lock or unsupported-host errors.
pub fn lock_shared(file: &File) -> io::Result<()> {
    apply_lock(file, DescriptorOperation::Shared)
}

/// Acquire a blocking exclusive lock on a caller-owned regular descriptor.
///
/// Ownership and deadline policy are the same as [`lock_shared`]. Use
/// [`lock_exclusive_with_wait`] when contention needs caller observations.
///
/// # Errors
/// Returns metadata, non-regular descriptor, native lock or unsupported-host errors.
pub fn lock_exclusive(file: &File) -> io::Result<()> {
    apply_lock(file, DescriptorOperation::Exclusive)
}

/// Attempt one exclusive lock acquisition on a caller-owned regular descriptor.
///
/// Contention returns [`io::ErrorKind::WouldBlock`] without waiting or retrying.
/// Success leaves the lock held by the descriptor, with the same clone, unlock
/// and final-close obligations as [`lock_exclusive`]. Opening, durability,
/// namespace custody and contention/retry policy remain caller-owned.
///
/// # Errors
/// Returns metadata, non-regular descriptor, native lock or unsupported-host errors.
pub fn try_lock_exclusive(file: &File) -> io::Result<()> {
    apply_lock(file, DescriptorOperation::TryExclusive)
}

/// Release a caller-owned regular descriptor's lock without closing it.
///
/// Unlocking releases the shared kernel ownership of its duplicated or inherited
/// descriptors too. It does not release independently opened holders. The caller
/// chooses the final-owner boundary; this does not close or remove any file.
///
/// # Errors
/// Returns metadata, non-regular descriptor, native unlock or unsupported-host errors.
pub fn unlock(file: &File) -> io::Result<()> {
    apply_lock(file, DescriptorOperation::Unlock)
}

#[derive(Clone, Copy)]
enum DescriptorOperation {
    Shared,
    Exclusive,
    TryExclusive,
    Unlock,
}

fn apply_lock(file: &File, operation: DescriptorOperation) -> io::Result<()> {
    require_regular_descriptor(file)?;
    #[cfg(unix)]
    {
        use rustix::fs::FlockOperation;
        let operation = match operation {
            DescriptorOperation::Shared => FlockOperation::LockShared,
            DescriptorOperation::Exclusive => FlockOperation::LockExclusive,
            DescriptorOperation::TryExclusive => FlockOperation::NonBlockingLockExclusive,
            DescriptorOperation::Unlock => FlockOperation::Unlock,
        };
        rustix::fs::flock(file, operation).map_err(io::Error::from)
    }
    #[cfg(not(unix))]
    {
        let _ = operation;
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

fn require_regular_descriptor(file: &File) -> io::Result<()> {
    if file.metadata()?.is_file() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "lock requires a regular file",
        ))
    }
}

/// Acquire an exclusive lock on a caller-owned regular descriptor with wait observations.
///
/// Each contention result calls `waiting` before sleeping for `poll_interval`.
/// Interrupted acquisition is retried without reporting contention. Uncontended
/// acquisition reports no wait. Returns elapsed acquisition time; the descriptor
/// retains the lock until the caller unlocks or closes it. The caller owns opening,
/// namespace, progress policy, deadlines and descriptor clones. This adds no path
/// admission or descendant ownership. Callback failure stops acquisition.
///
/// # Errors
/// Returns a zero polling interval, non-regular descriptor, metadata, callback,
/// lock acquisition or unsupported-platform error.
pub fn lock_exclusive_with_wait(
    file: &File,
    poll_interval: Duration,
    waiting: impl FnMut(Duration) -> io::Result<()>,
) -> io::Result<Duration> {
    if poll_interval.is_zero() || !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "lock requires a regular file and nonzero polling interval",
        ));
    }
    #[cfg(unix)]
    {
        wait_for_lock(poll_interval, waiting, || {
            rustix::fs::flock(file, rustix::fs::FlockOperation::NonBlockingLockExclusive)
                .map_err(|error| io::Error::from_raw_os_error(error.raw_os_error()))
        })
    }
    #[cfg(not(unix))]
    {
        let _ = waiting;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "descriptor lock waiting is unsupported",
        ))
    }
}

#[cfg(unix)]
fn wait_for_lock(
    poll_interval: Duration,
    mut waiting: impl FnMut(Duration) -> io::Result<()>,
    mut acquire: impl FnMut() -> io::Result<()>,
) -> io::Result<Duration> {
    let started = Instant::now();
    loop {
        match acquire() {
            Ok(()) => return Ok(started.elapsed()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                waiting(started.elapsed())?;
                std::thread::sleep(poll_interval);
            }
            Err(error) => return Err(error),
        }
    }
}
