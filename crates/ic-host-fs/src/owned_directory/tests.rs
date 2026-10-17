use super::{OwnedDirectory, remove_path_if_present};
use crate::test_support::fixture;
use std::{
    fs, io,
    panic::{AssertUnwindSafe, catch_unwind},
};

#[test]
fn consuming_cleanup_failure_leaves_recovery_to_the_caller() {
    let fixture = fixture();
    let path = fixture.path().join("owned");
    fs::create_dir(&path).unwrap();
    fs::write(path.join("removed"), b"partial cleanup").unwrap();
    fs::write(path.join("retained"), b"recovery evidence").unwrap();
    let error = OwnedDirectory::new(path.clone())
        .cleanup_with(|selected| {
            assert_eq!(selected, path);
            fs::remove_file(selected.join("removed"))?;
            Err(io::Error::from_raw_os_error(13))
        })
        .unwrap_err();
    assert_eq!(error.raw_os_error(), Some(13));
    assert!(!path.join("removed").exists());
    assert_eq!(
        fs::read(path.join("retained")).unwrap(),
        b"recovery evidence"
    );
    // A second best-effort Drop removal would have erased the recovery evidence.
    remove_path_if_present(&path).unwrap();
    assert!(!path.exists());
}

#[test]
fn unwind_removes_only_owned_child_and_preserve_transfers_custody() {
    let fixture = fixture();
    let owned = fixture.path().join("owned");
    let retained = fixture.path().join("retained");
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
    let fixture = fixture();
    let path = fixture.path().join("child");
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
    let fixture = fixture();
    let target = fixture.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("bytes"), b"retained").unwrap();
    let alias = fixture.path().join("alias");
    symlink(&target, &alias).unwrap();
    OwnedDirectory::new(alias.clone()).cleanup().unwrap();
    assert!(fs::symlink_metadata(alias).is_err());
    assert_eq!(fs::read(target.join("bytes")).unwrap(), b"retained");
}

#[cfg(unix)]
#[test]
fn cleanup_failure_preserves_the_selected_drop_policy() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    let fixture = fixture();
    if fs::metadata(fixture.path()).unwrap().uid() == 0 {
        return;
    }
    let path = fixture.path().join("owned");
    fs::create_dir(&path).unwrap();
    let permissions = fs::metadata(fixture.path()).unwrap().permissions();
    let mut guard = OwnedDirectory::new(path.clone());
    fs::set_permissions(fixture.path(), fs::Permissions::from_mode(0o500)).unwrap();
    let error = guard.try_cleanup().unwrap_err();
    fs::set_permissions(fixture.path(), permissions).unwrap();
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(guard.is_armed());
    drop(guard);
    assert!(!path.exists());

    fs::create_dir(&path).unwrap();
    let mut guard = OwnedDirectory::new(path.clone());
    // Borrowed cleanup needs no extra entrypoint: transfer recovery explicitly.
    guard.disarm();
    let permissions = fs::metadata(fixture.path()).unwrap().permissions();
    fs::set_permissions(fixture.path(), fs::Permissions::from_mode(0o500)).unwrap();
    let error = guard.try_cleanup().unwrap_err();
    fs::set_permissions(fixture.path(), permissions).unwrap();
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(!guard.is_armed());
    drop(guard);
    assert!(path.is_dir());
}
