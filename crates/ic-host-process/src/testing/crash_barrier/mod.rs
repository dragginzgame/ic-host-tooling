//! Opt-in acknowledged crash barriers for direct-child persistence tests.

use std::{
    io,
    path::Path,
    process::Child,
    thread,
    time::{Duration, Instant},
};

use ic_host_fs::durable::{PublicationMode, WriteOptions, write_bytes};

// Existence is the handshake signal, so publish complete bytes atomically.
// The caller exclusively owns this fixture namespace and its retained files.
fn publish_marker(path: &Path, contents: &[u8]) -> io::Result<()> {
    write_bytes(
        path,
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions: 0o600,
        },
        contents,
    )
    .map_err(io::Error::other)
}

/// Kill and reap a direct test child after a three-marker acknowledgement.
///
/// The child must call [`hold_at_acknowledged_barrier`] with the same exclusively
/// owned fixture root. The caller owns spawning, selected crash point, descendant
/// lifetime, fixture retention and recovery assertions. A wait failure kills and
/// reaps the direct child before returning the original failure.
///
/// # Errors
/// Returns marker read/write, early child exit, timeout, kill or reap errors.
pub fn kill_child_at_acknowledged_barrier(
    child: &mut Child,
    root: &Path,
    timeout: Duration,
) -> io::Result<()> {
    let ready_path = root.join("barrier-ready");
    let acknowledge_path = root.join("barrier-acknowledged");
    let armed_path = root.join("barrier-armed");
    let handshake = (|| {
        wait_for_child_path(child, &ready_path, "child barrier", timeout)?;
        publish_marker(&acknowledge_path, b"acknowledged\n")?;
        wait_for_child_path(child, &armed_path, "armed child barrier", timeout)
    })();
    if let Err(error) = handshake {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    child.kill()?;
    child.wait()?;
    Ok(())
}

/// Signal that a test child reached its selected barrier, then wait to be killed.
///
/// Panics if the fixture cannot be written or the parent never acknowledges the
/// barrier within `timeout`. A successful handshake deliberately never returns.
///
/// # Panics
/// Panics on marker write failure or missing parent acknowledgement.
pub fn hold_at_acknowledged_barrier(root: &Path, timeout: Duration) -> ! {
    let ready_path = root.join("barrier-ready");
    let acknowledge_path = root.join("barrier-acknowledged");
    let armed_path = root.join("barrier-armed");
    publish_marker(&ready_path, b"ready\n").expect("signal child barrier");
    wait_for_path(&acknowledge_path, "parent barrier acknowledgement", timeout)
        .expect("wait for parent barrier acknowledgement");
    publish_marker(&armed_path, b"armed\n").expect("arm child crash");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

/// Wait for a marker while checking whether the direct child already exited.
///
/// # Errors
/// Returns child-observation, early-exit or timeout errors.
pub fn wait_for_child_path(
    child: &mut Child,
    path: &Path,
    description: &str,
    timeout: Duration,
) -> io::Result<()> {
    wait_for_marker(path, description, timeout, || {
        if let Some(status) = child.try_wait()? {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("crash child exited before {description}: {status}"),
            ));
        }
        Ok(())
    })
}

/// Wait for a parent-owned marker without observing a child process.
///
/// # Errors
/// Returns a timeout when the marker does not appear by `timeout`.
pub fn wait_for_path(path: &Path, description: &str, timeout: Duration) -> io::Result<()> {
    wait_for_marker(path, description, timeout, || Ok(()))
}

fn wait_for_marker(
    path: &Path,
    description: &str,
    timeout: Duration,
    mut check_child: impl FnMut() -> io::Result<()>,
) -> io::Result<()> {
    let deadline = Instant::now() + timeout;
    while !path.is_file() {
        check_child()?;
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timed out waiting for {description}"),
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
