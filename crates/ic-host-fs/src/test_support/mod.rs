//! Private filesystem fixtures.
pub fn fixture() -> crate::testing::FixtureDirectory {
    crate::testing::FixtureDirectory::new(&std::env::temp_dir(), "ic-host-fs-test")
        .expect("create test fixture")
}

/// Create a real FIFO without opening either endpoint; callers select its mode.
#[cfg(unix)]
pub fn create_fifo(path: &std::path::Path, permissions: Option<u32>) -> std::io::Result<()> {
    // rustix's mkfifoat is unavailable on Apple hosts. Use the native POSIX
    // utility consistently so every special-file fixture exercises a real FIFO.
    let mut command = std::process::Command::new("/usr/bin/mkfifo");
    command.env_clear();
    if let Some(permissions) = permissions {
        command.args(["-m", &format!("{permissions:o}")]);
    }
    let status = command.arg(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "FIFO fixture creation failed: {status}"
        )))
    }
}
