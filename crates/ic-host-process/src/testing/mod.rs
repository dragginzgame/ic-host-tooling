//! Opt-in Unix executable fixtures for consumer tests.
//!
//! This module owns only safe writer-process lifetime. Callers own fixture
//! content, paths, namespace custody and assertions; no tool is admitted here.

use std::{
    fs,
    io::{self, Write as _},
    os::unix::fs::PermissionsExt as _,
    path::Path,
    process::{Command, Stdio},
};

use crate::child::OwnedChild;

pub mod crash_barrier;

/// Create or replace a caller-owned executable script with mode 0755.
///
/// A parallel fork can inherit a parent's writable executable descriptor until
/// exec, causing ETXTBSY after the parent closes it. This parent holds only a
/// pipe: a private writer opens the target, receives all bytes, and is reaped
/// before permissions are set or the caller can execute the script.
/// No parent directories are created and no path/symlink admission is supplied.
/// Use a trusted, exclusively owned fixture namespace. Failed writes may leave
/// partial bytes for the caller to inspect. There is no deadline or retry.
///
/// # Errors
/// Returns spawn, pipe, write, wait, failed writer exit or filesystem errors.
/// The writer is waited after a write failure; that original failure remains
/// primary. The owned process group is cleaned up if waiting fails or unwinds.
pub fn write_executable_script(path: &Path, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let mut writer = OwnedChild::spawn(
        Command::new("/bin/sh")
            .args(["-c", "cat > \"$1\"", "write-test-script"])
            .arg(path)
            .stdin(Stdio::piped()),
    )?;
    let written = writer
        .take_stdin()
        .ok_or_else(|| io::Error::other("script writer stdin missing"))?
        .write_all(contents.as_ref());
    let status = writer.wait();
    written?;
    let status = status?;
    if !status.success() {
        return Err(io::Error::other(format!("script writer failed: {status}")));
    }
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

#[cfg(test)]
mod tests;
