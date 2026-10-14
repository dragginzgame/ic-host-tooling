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
