//! Non-Unix API admission; the Wasm gate compiles this contract without execution.
#![cfg(not(unix))]

use ic_host_fs::durable::{
    RegularFileLockError, open_existing_regular_lock_file, open_private_regular_lock_file,
    open_regular_lock_file_with_parents,
};
use std::path::Path;

#[test]
fn non_unix_lock_opening_refuses_before_path_access() {
    // An unsupported platform must refuse even a path invalid for filesystem IO.
    let path = Path::new("invalid\0lock");
    assert!(matches!(
        open_regular_lock_file_with_parents(path),
        Err(RegularFileLockError::UnsupportedPlatform)
    ));
    assert!(matches!(
        open_private_regular_lock_file(path),
        Err(RegularFileLockError::UnsupportedPlatform)
    ));
    assert!(matches!(
        open_existing_regular_lock_file(path),
        Err(RegularFileLockError::UnsupportedPlatform)
    ));
}
