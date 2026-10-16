use super::*;

#[cfg(unix)]
#[test]
fn no_follow_directory_refuses_links_and_non_directories() {
    use std::{fs, os::unix::fs::symlink};

    let fixture = crate::test_support::Fixture::new();
    let root = fixture.root.join("directory");
    fs::create_dir(&root).unwrap();
    let held = open_no_follow(&root).unwrap();
    assert!(crate::durable::admit_directory(&root, &held).is_ok());

    let linked = fixture.root.join("linked");
    symlink(&root, &linked).unwrap();
    assert!(open_no_follow(&linked).is_err());
    assert!(crate::durable::admit_directory(&linked, &held).is_err());

    let old = fixture.root.join("old");
    fs::rename(&root, &old).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(crate::durable::admit_directory(&root, &held).is_err());
    assert!(crate::durable::admit_directory(&old, &held).is_ok());
    assert!(open_no_follow(&root).is_ok());
    assert!(open_no_follow(&fixture.root.join("missing")).is_err());
    assert!(open_no_follow(&linked).is_err());
    let regular = fixture.root.join("file");
    fs::write(&regular, b"not a directory").unwrap();
    assert!(open_no_follow(&regular).is_err());
}
