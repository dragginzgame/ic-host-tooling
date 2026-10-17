use super::{OwnedDirectory, remove_path_if_present};
use crate::test_support::Fixture;
use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
};

#[test]
fn unwind_removes_only_owned_child_and_preserve_transfers_custody() {
    let fixture = Fixture::new();
    let owned = fixture.root.join("owned");
    let retained = fixture.root.join("retained");
    fs::create_dir(&owned).unwrap();
    fs::write(owned.join("bytes"), b"owned").unwrap();
    fs::create_dir(&retained).unwrap();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _owned = OwnedDirectory::new(owned.clone());
            OwnedDirectory::new(retained.clone()).preserve();
            panic!("producer unwind");
        }))
        .is_err()
    );
    assert!(!owned.exists());
    assert!(retained.is_dir());
}

#[test]
fn absent_files_and_directories_have_explicit_cleanup() {
    let fixture = Fixture::new();
    let path = fixture.root.join("child");
    let mut guard = OwnedDirectory::new(path.clone());
    assert!(guard.is_armed());
    assert_eq!(guard.path(), path);
    guard.try_cleanup().unwrap();
    assert!(!guard.is_armed());
    fs::write(&path, b"replacement").unwrap();
    guard.disarm();
    drop(guard);
    assert!(path.is_file());
    OwnedDirectory::new(path.clone()).cleanup().unwrap();
    assert!(!path.exists());
    remove_path_if_present(&path).unwrap();
}

#[cfg(unix)]
#[test]
fn final_symlinks_are_unlinked_without_touching_their_targets() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let target = fixture.root.join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("bytes"), b"retained").unwrap();
    let alias = fixture.root.join("alias");
    symlink(&target, &alias).unwrap();
    OwnedDirectory::new(alias.clone()).cleanup().unwrap();
    assert!(fs::symlink_metadata(alias).is_err());
    assert_eq!(fs::read(target.join("bytes")).unwrap(), b"retained");
}

#[cfg(unix)]
#[test]
fn cleanup_failure_stays_armed_for_drop_retry() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    let fixture = Fixture::new();
    if fs::metadata(&fixture.root).unwrap().uid() == 0 {
        return;
    }
    let path = fixture.root.join("owned");
    fs::create_dir(&path).unwrap();
    let permissions = fs::metadata(&fixture.root).unwrap().permissions();
    let mut guard = OwnedDirectory::new(path.clone());
    fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o500)).unwrap();
    let error = guard.try_cleanup().unwrap_err();
    fs::set_permissions(&fixture.root, permissions).unwrap();
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(guard.is_armed());
    drop(guard);
    assert!(!path.exists());
}
