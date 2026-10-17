use super::canonicalize_allow_missing;
use crate::test_support::fixture;
use std::{fs, io, path::Path};

#[test]
fn missing_parent_traversal_resumes_symlink_resolution() {
    let fixture = fixture();
    let target = fixture.path().join("real");
    fs::create_dir(&target).unwrap();
    std::os::unix::fs::symlink(&target, fixture.path().join("alias")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(
            Path::new("missing/../alias/generated/nested/../output"),
            fixture.path(),
            usize::MAX
        )
        .unwrap(),
        target.join("generated/output")
    );
    assert_eq!(
        canonicalize_allow_missing(
            &fixture.path().join("alias/../other/output"),
            Path::new("ignored"),
            usize::MAX
        )
        .unwrap(),
        fixture.path().join("other/output")
    );
}

#[test]
fn explicit_base_errors_and_native_names_are_preserved() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt as _};
    let fixture = fixture();
    assert_eq!(
        canonicalize_allow_missing(Path::new("output"), Path::new("relative"), usize::MAX)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    let native = OsString::from_vec(vec![b'n', 0xff]);
    assert_eq!(
        canonicalize_allow_missing(Path::new(&native), fixture.path(), usize::MAX).unwrap(),
        fixture.path().join(&native)
    );
    std::os::unix::fs::symlink(&native, fixture.path().join("native-alias")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("native-alias"), fixture.path(), usize::MAX).unwrap(),
        fixture.path().join(native)
    );
    fs::write(fixture.path().join("file"), b"contents").unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("file/child"), fixture.path(), usize::MAX)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotADirectory
    );
    assert_eq!(
        canonicalize_allow_missing(Path::new("."), fixture.path(), usize::MAX).unwrap(),
        fixture.path()
    );
}

#[test]
fn existing_file_traversal_errors_survive_missing_suffix_normalization() {
    let fixture = fixture();
    fs::write(fixture.path().join("file"), b"contents").unwrap();
    // realpath/canonicalize may normalize file/.. on macOS where Linux rejects
    // it. The contract retains the host's result, including after a missing
    // prefix is normalized; it does not impose Linux traversal on every host.
    for (path, existing, suffix) in [
        ("file/..", "file/..", ""),
        ("file/.", "file/.", ""),
        ("file/", "file/", ""),
        ("missing/../file/../output", "file/..", "output"),
        ("missing/../file/.", "file/.", ""),
        ("missing/../file/", "file/", ""),
    ] {
        let native = fixture
            .path()
            .join(existing)
            .canonicalize()
            .map(|resolved| {
                if suffix.is_empty() {
                    resolved
                } else {
                    resolved.join(suffix)
                }
            });
        let actual = canonicalize_allow_missing(Path::new(path), fixture.path(), usize::MAX);
        assert_eq!(
            actual.map_err(|error| error.kind()),
            native.map_err(|error| error.kind()),
            "native directory traversal: {path}"
        );
    }
}

#[test]
fn dangling_symlink_targets_chains_and_parent_traversal_share_one_identity() {
    use std::os::unix::fs::symlink;
    let fixture = fixture();
    fs::create_dir(fixture.path().join("real")).unwrap();
    symlink("real/generated", fixture.path().join("relative")).unwrap();
    symlink(
        fixture.path().join("real/generated"),
        fixture.path().join("absolute"),
    )
    .unwrap();
    symlink("relative", fixture.path().join("chain")).unwrap();
    symlink("missing/../chain", fixture.path().join("rewind")).unwrap();
    for path in ["relative", "absolute", "chain", "rewind", "chain/nested/.."] {
        assert_eq!(
            canonicalize_allow_missing(Path::new(path), fixture.path(), usize::MAX).unwrap(),
            fixture.path().join("real/generated")
        );
    }
    assert_eq!(
        canonicalize_allow_missing(
            Path::new("chain/../../chain/output"),
            fixture.path(),
            usize::MAX
        )
        .unwrap(),
        fixture.path().join("real/generated/output")
    );
}

#[test]
fn symlink_cycles_reject_without_confusing_repeated_completed_expansions() {
    use std::os::unix::fs::symlink;
    let fixture = fixture();
    symlink("missing/../cycle", fixture.path().join("cycle")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("cycle"), fixture.path(), usize::MAX)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    symlink("second", fixture.path().join("first")).unwrap();
    symlink("first", fixture.path().join("second")).unwrap();
    let native = fixture.path().join("first").canonicalize().unwrap_err();
    assert_eq!(
        canonicalize_allow_missing(Path::new("first"), fixture.path(), usize::MAX)
            .unwrap_err()
            .kind(),
        native.kind()
    );
    symlink("missing", fixture.path().join("alias")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(
            Path::new("alias/../alias/../output"),
            fixture.path(),
            usize::MAX
        )
        .unwrap(),
        fixture.path().join("output")
    );
}

#[test]
fn long_acyclic_missing_target_chains_use_iterative_resolution() {
    use std::os::unix::fs::symlink;
    let fixture = fixture();
    for index in 0..100 {
        symlink(
            format!("missing/../chain-{}", index + 1),
            fixture.path().join(format!("chain-{index}")),
        )
        .unwrap();
    }
    symlink("generated/output", fixture.path().join("chain-100")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("chain-0"), fixture.path(), usize::MAX).unwrap(),
        fixture.path().join("generated/output")
    );
}

#[test]
fn missing_target_depth_is_caller_selected_and_completed_links_do_not_accumulate() {
    use std::os::unix::fs::symlink;
    let fixture = fixture();
    symlink("missing", fixture.path().join("alias")).unwrap();
    symlink("alias", fixture.path().join("chain")).unwrap();
    for (path, depth) in [("alias", 0), ("chain", 1)] {
        assert_eq!(
            canonicalize_allow_missing(Path::new(path), fixture.path(), depth)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
    for (path, depth, expected) in [
        ("plain/output", 0, "plain/output"),
        ("chain", 2, "missing"),
        ("alias/../alias/../output", 1, "output"),
    ] {
        assert_eq!(
            canonicalize_allow_missing(Path::new(path), fixture.path(), depth).unwrap(),
            fixture.path().join(expected)
        );
    }
}
