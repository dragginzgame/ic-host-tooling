//! Descriptor acquisition shared by path admission and consumer-owned lock files.

use std::{
    fs::File,
    io,
    time::{Duration, Instant},
};

#[cfg(all(test, unix))]
mod tests;

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
