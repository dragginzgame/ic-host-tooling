//! Private filesystem fixtures.
use std::{fs, path::PathBuf};
pub struct Fixture {
    pub root: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        let root =
            crate::testing::create_fixture_directory(&std::env::temp_dir(), "ic-host-fs-test")
                .expect("create filesystem fixture");
        Self { root }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "Failed filesystem fixture retained: {}",
                self.root.display()
            );
            return;
        }
        let _ = fs::remove_dir_all(&self.root);
    }
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
