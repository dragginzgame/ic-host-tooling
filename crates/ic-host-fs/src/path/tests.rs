use super::{canonicalize_allow_missing, canonicalize_allow_missing_with_symlink_limit};
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
        fixture.root.join(&native)
    );
    std::os::unix::fs::symlink(&native, fixture.root.join("native-alias")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("native-alias"), &fixture.root).unwrap(),
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

#[test]
fn existing_file_traversal_errors_survive_missing_suffix_normalization() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("file"), b"contents").unwrap();
    for path in [
        "file/..",
        "file/.",
        "file/",
        "missing/../file/../output",
        "missing/../file/.",
        "missing/../file/",
    ] {
        assert_eq!(
            canonicalize_allow_missing(Path::new(path), &fixture.root)
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotADirectory,
            "invalid directory traversal: {path}"
        );
    }
}

#[test]
fn dangling_symlink_targets_chains_and_parent_traversal_share_one_identity() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    fs::create_dir(fixture.root.join("real")).unwrap();
    symlink("real/generated", fixture.root.join("relative")).unwrap();
    symlink(
        fixture.root.join("real/generated"),
        fixture.root.join("absolute"),
    )
    .unwrap();
    symlink("relative", fixture.root.join("chain")).unwrap();
    symlink("missing/../chain", fixture.root.join("rewind")).unwrap();
    for path in ["relative", "absolute", "chain", "rewind", "chain/nested/.."] {
        assert_eq!(
            canonicalize_allow_missing(Path::new(path), &fixture.root).unwrap(),
            fixture.root.join("real/generated")
        );
    }
    assert_eq!(
        canonicalize_allow_missing(Path::new("chain/../../chain/output"), &fixture.root).unwrap(),
        fixture.root.join("real/generated/output")
    );
}

#[test]
fn symlink_cycles_reject_without_confusing_repeated_completed_expansions() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    symlink("missing/../cycle", fixture.root.join("cycle")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("cycle"), &fixture.root)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    symlink("second", fixture.root.join("first")).unwrap();
    symlink("first", fixture.root.join("second")).unwrap();
    let native = fixture.root.join("first").canonicalize().unwrap_err();
    assert_eq!(
        canonicalize_allow_missing(Path::new("first"), &fixture.root)
            .unwrap_err()
            .kind(),
        native.kind()
    );
    symlink("missing", fixture.root.join("alias")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("alias/../alias/../output"), &fixture.root).unwrap(),
        fixture.root.join("output")
    );
}

#[test]
fn long_acyclic_missing_target_chains_use_iterative_resolution() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    for index in 0..100 {
        symlink(
            format!("missing/../chain-{}", index + 1),
            fixture.root.join(format!("chain-{index}")),
        )
        .unwrap();
    }
    symlink("generated/output", fixture.root.join("chain-100")).unwrap();
    assert_eq!(
        canonicalize_allow_missing(Path::new("chain-0"), &fixture.root).unwrap(),
        fixture.root.join("generated/output")
    );
}

#[test]
fn missing_target_depth_is_caller_selected_and_completed_links_do_not_accumulate() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    symlink("missing", fixture.root.join("alias")).unwrap();
    symlink("alias", fixture.root.join("chain")).unwrap();
    for (path, depth) in [("alias", 0), ("chain", 1)] {
        assert_eq!(
            canonicalize_allow_missing_with_symlink_limit(Path::new(path), &fixture.root, depth)
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
            canonicalize_allow_missing_with_symlink_limit(Path::new(path), &fixture.root, depth)
                .unwrap(),
            fixture.root.join(expected)
        );
    }
}
