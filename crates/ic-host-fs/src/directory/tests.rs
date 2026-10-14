use super::*;

#[cfg(unix)]
#[test]
fn no_follow_directory_retains_original_identity_across_replacement() {
    use std::{fs, os::unix::fs::symlink};

    let fixture = crate::test_support::Fixture::new();
    let root = fixture.root.join("directory");
    fs::create_dir(&root).unwrap();
    let held = open_no_follow(&root).unwrap();
    assert!(same_held_inode(&root, &held).unwrap());

    let linked = fixture.root.join("linked");
    symlink(&root, &linked).unwrap();
    assert!(open_no_follow(&linked).is_err());
    assert!(!same_held_inode(&linked, &held).unwrap());

    let old = fixture.root.join("old");
    fs::rename(&root, &old).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(!same_held_inode(&root, &held).unwrap());
    assert!(same_held_inode(&old, &held).unwrap());
    assert!(open_no_follow(&root).is_ok());
    assert!(open_no_follow(&fixture.root.join("missing")).is_err());
    assert!(open_no_follow(&linked).is_err());
}
