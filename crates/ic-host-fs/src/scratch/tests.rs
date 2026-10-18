use super::*;
use crate::test_support::fixture;
use std::{
    error::Error as _,
    os::unix::fs::{PermissionsExt as _, symlink},
};

#[test]
fn complete_private_files_are_absolute_unique_and_caller_retained() {
    let fixture = fixture();
    let alias = fixture.path().join("alias");
    let parent = fixture.path().join("parent");
    fs::create_dir(&parent).unwrap();
    symlink(&parent, &alias).unwrap();
    let payload = vec![0xa5; 128 * 1024];
    let path = create_private_bytes(&alias, "arguments", &payload, 4).unwrap();
    let empty = create_private_bytes(&alias, "arguments", b"", 4).unwrap();
    assert!(path.is_absolute());
    assert_eq!(path.parent().unwrap(), parent.canonicalize().unwrap());
    assert_ne!(path, empty);
    assert_eq!(fs::read(&path).unwrap(), payload);
    assert_eq!(fs::read(&empty).unwrap(), b"");
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o177, 0);
    assert!(path.exists());
    fs::remove_file(path).unwrap();
    fs::remove_file(empty).unwrap();
}

#[test]
fn collision_allowance_preserves_files_links_and_directories() {
    let fixture = fixture();
    let candidate = |sequence| {
        fixture
            .path()
            .join(format!("held-{}-{sequence}", std::process::id()))
    };
    let target = fixture.path().join("target");
    fs::write(&target, b"target").unwrap();
    fs::write(candidate(0), b"existing").unwrap();
    symlink(&target, candidate(1)).unwrap();
    fs::create_dir(candidate(2)).unwrap();
    let mut sequence = 0;
    let error = create_with_sequence(fixture.path(), "held", b"new", 3, || {
        let next = sequence;
        sequence += 1;
        next
    })
    .unwrap_err();
    assert!(
        matches!(error, PrivateBytesError::BeforeCreation(ref source) if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(sequence, 3);
    assert_eq!(fs::read(candidate(0)).unwrap(), b"existing");
    assert_eq!(fs::read_link(candidate(1)).unwrap(), target);
    assert!(candidate(2).is_dir());
    assert!(!candidate(3).exists());
    assert_eq!(fs::read(&target).unwrap(), b"target");
    let path = create_with_sequence(fixture.path(), "held", b"new", 4, {
        let mut n = 0;
        move || {
            let next = n;
            n += 1;
            next
        }
    })
    .unwrap();
    assert_eq!(path, candidate(3));
    assert_eq!(fs::read(path).unwrap(), b"new");
}

#[test]
fn invalid_requests_and_open_failures_create_nothing() {
    let fixture = fixture();
    for prefix in ["", ".", "..", "a/b", "a/", "a\n", "a\r", "a\0"] {
        assert!(
            matches!(create_private_bytes(fixture.path(), prefix, b"secret", 1), Err(PrivateBytesError::BeforeCreation(source)) if source.kind() == io::ErrorKind::InvalidInput)
        );
    }
    assert!(
        matches!(create_private_bytes(fixture.path(), "ok", b"", 0), Err(PrivateBytesError::BeforeCreation(source)) if source.kind() == io::ErrorKind::InvalidInput)
    );
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 0);
    let missing = create_private_bytes(&fixture.path().join("missing"), "ok", b"", 3).unwrap_err();
    assert_eq!(missing.to_string(), "private scratch creation failed");
    assert_eq!(
        missing
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        io::ErrorKind::NotFound
    );
    let file = fixture.path().join("file");
    fs::write(&file, b"preserved").unwrap();
    assert!(matches!(
        create_private_bytes(&file, "ok", b"", 3),
        Err(PrivateBytesError::BeforeCreation(_))
    ));
    assert_eq!(fs::read(file).unwrap(), b"preserved");
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 1);
}

#[test]
fn partial_write_failure_removes_only_the_created_candidate() {
    let fixture = fixture();
    let path = fixture.path().join("candidate");
    let retained = fixture.path().join("retained");
    fs::write(&retained, b"retained").unwrap();
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .unwrap();
    let error = finish_candidate(
        file,
        path.clone(),
        |file| {
            file.write_all(b"partial")?;
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "original write failure",
            ))
        },
        |path| fs::remove_file(path),
    )
    .unwrap_err();
    assert!(!path.exists());
    assert_eq!(error.to_string(), "private scratch write failed");
    assert_eq!(fs::read(retained).unwrap(), b"retained");
    assert!(
        matches!(&error, PrivateBytesError::Write { path: observed, source, cleanup: None } if observed == &path && source.kind() == io::ErrorKind::BrokenPipe)
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "original write failure"
    );
}

#[test]
fn failed_cleanup_retains_the_partial_file_and_both_causes() {
    let fixture = fixture();
    let path = fixture.path().join("candidate");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .unwrap();
    let error = finish_candidate(
        file,
        path.clone(),
        |file| {
            file.write_all(b"private partial bytes")?;
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "original write failure",
            ))
        },
        |_| {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "original cleanup failure",
            ))
        },
    )
    .unwrap_err();
    assert_eq!(fs::read(&path).unwrap(), b"private partial bytes");
    assert!(
        matches!(&error, PrivateBytesError::Write { path: observed, source, cleanup: Some(cleanup) } if observed == &path && source.kind() == io::ErrorKind::BrokenPipe && cleanup.kind() == io::ErrorKind::PermissionDenied && cleanup.to_string() == "original cleanup failure")
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "original write failure"
    );
    assert_eq!(
        error.to_string(),
        "private scratch write and cleanup failed"
    );
}
