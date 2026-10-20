//! Private fixture paths shared by unit and integration targets.

use std::path::PathBuf;

pub fn unique_temp_directory(label: &str) -> PathBuf {
    ic_host_fs::testing::create_fixture_directory(
        &std::env::temp_dir(),
        &format!("ic-testkit-{label}"),
    )
    .expect("create test directory")
}
