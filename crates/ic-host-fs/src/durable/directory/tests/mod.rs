use super::*;
use crate::test_support::Fixture;
use std::os::unix::fs::PermissionsExt as _;

#[test]
fn claim_is_private_exclusive_and_retained_after_drop() {
    let fixture = Fixture::new();
    let path = fixture.root.join("claim");
    let directory = create_private_directory(&path).unwrap();
    assert_eq!(
        directory.metadata().unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert!(
        matches!(create_private_directory(&path), Err(DirectoryCreationError::BeforeCreation(error)) if error.kind() == io::ErrorKind::AlreadyExists)
    );
    admit_directory(&path, &directory).unwrap();
    drop(directory);
    assert!(path.is_dir());
}

#[test]
fn replaced_removed_and_symlink_names_do_not_admit_original_descriptor() {
    let fixture = Fixture::new();
    let path = fixture.root.join("claim");
    let moved = fixture.root.join("moved");
    let directory = create_private_directory(&path).unwrap();
    std::fs::rename(&path, &moved).unwrap();
    assert!(admit_directory(&path, &directory).is_err());
    std::fs::create_dir(&path).unwrap();
    assert!(admit_directory(&path, &directory).is_err());
    std::fs::remove_dir(&path).unwrap();
    std::os::unix::fs::symlink(&moved, &path).unwrap();
    assert!(admit_directory(&path, &directory).is_err());
    assert!(create_private_directory(&path).is_err());
    admit_directory(&moved, &directory).unwrap();
    let file = File::create(fixture.root.join("regular")).unwrap();
    assert!(admit_directory(&moved, &file).is_err());
}

#[test]
fn missing_parent_and_forbidden_namespace_leave_no_claim() {
    let fixture = Fixture::new();
    for path in [
        fixture.root.join("missing/claim"),
        fixture.root.join("bad\nclaim"),
    ] {
        assert!(matches!(
            create_private_directory(&path),
            Err(DirectoryCreationError::BeforeCreation(_))
        ));
        assert!(!path.exists());
    }
}
