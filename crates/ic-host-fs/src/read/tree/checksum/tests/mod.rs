use super::*;
use crate::test_support::Fixture;
use std::{fs, os::unix::fs::symlink};

#[test]
fn filesystem_and_declarations_keep_original_framing() {
    let fixture = Fixture::new();
    let root = &fixture.root;
    assert_eq!(
        checksum_directory(root, 0).unwrap(),
        Sha256Digest::compute(b"")
    );
    fs::create_dir(root.join("nested")).unwrap();
    fs::write(root.join("a.txt"), b"a").unwrap();
    fs::write(root.join("nested/b.txt"), b"b").unwrap();
    let expected = checksum_relative_files(vec![
        ("nested/b.txt".into(), Sha256Digest::compute(b"b")),
        ("a.txt".into(), Sha256Digest::compute(b"a")),
    ])
    .unwrap();
    let actual = checksum_directory(root, 1).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(
        actual.to_string(),
        "e4d330f138b8f1b3044e84b5dcbe4fd1cb7e043d0c20810c083d791b6de01266"
    );
    let directory = open_directory_no_follow(root).unwrap();
    assert_eq!(
        checksum_directory_at(&directory.into(), root, 1).unwrap(),
        expected
    );
}

#[test]
fn per_file_limit_rejects_without_changing_input() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("bytes"), b"abc").unwrap();
    assert!(matches!(
        checksum_directory(&fixture.root, 2),
        Err(TreeChecksumError::Input(
            ArtifactError::LimitExceeded { .. }
        ))
    ));
    assert_eq!(fs::read(fixture.root.join("bytes")).unwrap(), b"abc");
    checksum_directory(&fixture.root, 3).unwrap();
}

#[test]
fn symlinks_and_special_entries_reject_without_following_or_waiting() {
    let fixture = Fixture::new();
    let outside = fixture.root.join("outside");
    let root = fixture.root.join("tree");
    fs::create_dir(&root).unwrap();
    fs::write(&outside, b"outside").unwrap();
    for target in [&outside, &fixture.root.join("missing")] {
        symlink(target, root.join("link")).unwrap();
        assert!(matches!(
            checksum_directory(&root, u64::MAX),
            Err(TreeChecksumError::UnsupportedEntry {
                kind: FileType::Symlink,
                ..
            })
        ));
        fs::remove_file(root.join("link")).unwrap();
    }
    crate::test_support::create_fifo(&root.join("pipe"), None).unwrap();
    assert!(matches!(
        checksum_directory(&root, u64::MAX),
        Err(TreeChecksumError::UnsupportedEntry {
            kind: FileType::Fifo,
            ..
        })
    ));
    assert_eq!(fs::read(outside).unwrap(), b"outside");
}

#[test]
fn root_path_admission_rejects_aliases_files_and_missing_paths() {
    let fixture = Fixture::new();
    let root = fixture.root.join("tree");
    fs::create_dir(&root).unwrap();
    symlink(&root, fixture.root.join("alias")).unwrap();
    assert!(checksum_directory(&fixture.root.join("alias"), 0).is_err());
    fs::write(fixture.root.join("file"), b"x").unwrap();
    assert!(matches!(
        checksum_directory(&fixture.root.join("file"), 1),
        Err(TreeChecksumError::UnsupportedEntry {
            kind: FileType::RegularFile,
            ..
        })
    ));
    assert!(
        matches!(checksum_directory(&fixture.root.join("missing"), 0), Err(TreeChecksumError::Io(error)) if error.kind() == io::ErrorKind::NotFound)
    );
    for path in [
        Path::new(""),
        Path::new("/"),
        Path::new("."),
        Path::new("../tree"),
    ] {
        assert!(matches!(
            checksum_directory(path, 0),
            Err(TreeChecksumError::InvalidPath { .. })
        ));
    }
}

#[test]
fn held_root_keeps_its_inode_after_path_replacement() {
    let fixture = Fixture::new();
    let root = fixture.root.join("tree");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a"), b"original").unwrap();
    let expected = checksum_directory(&root, 8).unwrap();
    let directory = open_directory_no_follow(&root).unwrap();
    fs::rename(&root, fixture.root.join("retained")).unwrap();
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a"), b"changed").unwrap();
    assert_eq!(
        checksum_directory_at(&directory.into(), &root, 8).unwrap(),
        expected
    );
    assert_ne!(checksum_directory(&root, 8).unwrap(), expected);
}

#[test]
fn non_utf8_child_is_rejected_without_lossy_substitution() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let fixture = Fixture::new();
    let path = fixture.root.join(OsStr::from_bytes(b"\xff"));
    if let Err(error) = fs::write(&path, b"bytes") {
        assert!(
            cfg!(target_os = "macos")
                && error.raw_os_error() == Some(rustix::io::Errno::ILSEQ.raw_os_error()),
            "{error}"
        );
        return;
    }
    assert!(
        matches!(checksum_directory(&fixture.root, 5), Err(TreeChecksumError::NonUtf8Path { path: actual }) if actual == path)
    );
    assert_eq!(fs::read(path).unwrap(), b"bytes");
}
