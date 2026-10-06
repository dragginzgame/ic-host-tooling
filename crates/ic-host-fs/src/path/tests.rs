use super::canonicalize_allow_missing;
use crate::test_support::Fixture;
use std::{fs, io, path::Path};

#[test]
fn missing_parent_traversal_resumes_symlink_resolution() {
    let fixture = Fixture::new();
    let target = fixture.root.join("real");
    fs::create_dir(&target).unwrap();
    std::os::unix::fs::symlink(&target, fixture.root.join("alias")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(
            Path::new("missing/../alias/generated/nested/../output"),
            &fixture.root
        )
        .unwrap(),
        target.join("generated/output")
    );
    assert_eq!(
        canonicalize_allow_missing(
            &fixture.root.join("alias/../other/output"),
            Path::new("ignored")
        )
        .unwrap(),
        fixture.root.join("other/output")
    );
}

#[test]
fn explicit_base_errors_and_native_names_are_preserved() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt as _};
    let fixture = Fixture::new();
    assert_eq!(
        canonicalize_allow_missing(Path::new("output"), Path::new("relative"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    let native = OsString::from_vec(vec![b'n', 0xff]);
    assert_eq!(
        canonicalize_allow_missing(Path::new(&native), &fixture.root).unwrap(),
        fixture.root.join(native)
    );
    fs::write(fixture.root.join("file"), b"contents").unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("file/child"), &fixture.root)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotADirectory
    );
    assert_eq!(
        canonicalize_allow_missing(Path::new("."), &fixture.root).unwrap(),
        fixture.root
    );
}
