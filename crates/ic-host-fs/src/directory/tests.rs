use super::*;

#[test]
fn collision_budget_preserves_existing_files_and_directories() {
    let root = crate::test_support::fixture();
    let prefix = "owned";
    let name = |sequence| {
        root.path()
            .join(format!("{prefix}-{}-{sequence}", std::process::id()))
    };
    fs::write(name(0), b"retained").unwrap();
    fs::create_dir(name(1)).unwrap();
    let sequence = AtomicU64::new(0);
    let error = create_unique_directory_with_sequence(
        root.path(),
        prefix,
        0o777,
        NonZeroUsize::new(2),
        &sequence,
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(name(0)).unwrap(), b"retained");
    assert!(name(1).is_dir());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    let created =
        create_unique_directory_with_sequence(root.path(), prefix, 0o777, None, &sequence).unwrap();
    assert_eq!(created, name(2));
    assert!(created.is_dir());
}

#[test]
fn invalid_policy_and_missing_parent_create_nothing() {
    let root = crate::test_support::fixture();
    for prefix in ["", ".", "..", "a/b", "a/", "a\nb", "a\rb"] {
        assert_eq!(
            create_unique_directory(root.path(), prefix, 0o700, None)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
    assert_eq!(
        create_unique_directory(root.path(), "owned", 0o1700, None)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(
        create_unique_directory(&root.path().join("missing"), "owned", 0o700, None)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn alias_paths_and_selected_permissions_remain_caller_owned() {
    use std::os::unix::fs::{PermissionsExt as _, symlink};
    let root = crate::test_support::fixture();
    let physical = root.path().join("physical");
    let alias = root.path().join("alias");
    fs::create_dir(&physical).unwrap();
    symlink(&physical, &alias).unwrap();
    let created = create_unique_directory(&alias, "owned", 0o700, NonZeroUsize::new(1)).unwrap();
    assert_eq!(created.parent(), Some(alias.as_path()));
    assert!(created.canonicalize().unwrap().starts_with(&physical));
    assert_eq!(
        fs::metadata(created).unwrap().permissions().mode() & 0o077,
        0
    );
}
