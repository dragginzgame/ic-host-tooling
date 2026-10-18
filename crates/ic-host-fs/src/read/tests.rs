use super::*;
use ic_host_artifacts::artifact::Sha256Digest;
use std::io;

#[cfg(unix)]
#[test]
fn hashing_retains_the_selected_descriptors_metadata() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    let fixture = crate::test_support::fixture();
    let path = fixture.path().join("observed");
    for observe in [hash_file_with_metadata, hash_file_no_follow_with_metadata] {
        std::fs::write(&path, b"selected bytes").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        let selected = File::open(&path).unwrap();
        let expected = selected.metadata().unwrap();
        let (identity, metadata) = observe(&path, 14).unwrap();
        assert_eq!(identity.bytes, 14);
        assert_eq!(identity.sha256, Sha256Digest::compute(b"selected bytes"));
        assert_eq!(
            (metadata.dev(), metadata.ino()),
            (expected.dev(), expected.ino())
        );
        assert_eq!(metadata.permissions().mode() & 0o777, 0o640);
        assert_eq!(metadata.len(), identity.bytes);
        assert!(matches!(
            observe(&path, 13),
            Err(ArtifactError::LimitExceeded { limit: 13 })
        ));

        let retained = fixture.path().join("retained");
        std::fs::rename(&path, &retained).unwrap();
        std::fs::write(&path, b"different").unwrap();
        assert_ne!(std::fs::metadata(&path).unwrap().ino(), metadata.ino());
        assert_eq!(selected.metadata().unwrap().ino(), metadata.ino());
        std::fs::remove_file(retained).unwrap();
    }
}

#[test]
fn bounded_file_operations_leave_source_intact_and_reject_special_inputs() {
    // A checked-in source fixture avoids temp allocation and filesystem cleanup.
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/empty.wasm"
    ));
    let original = std::fs::read(path).unwrap();
    assert_eq!(read_file(path, 8).unwrap(), original);
    assert_eq!(
        hash_file(path, 8).unwrap().sha256,
        Sha256Digest::compute(&original)
    );
    assert!(matches!(
        read_file(path, 7),
        Err(ArtifactError::LimitExceeded { limit: 7 })
    ));
    assert!(matches!(
        hash_file(path.parent().unwrap(), 100),
        Err(ArtifactError::NotRegularFile)
    ));
    assert_eq!(std::fs::read(path).unwrap(), original);
    let missing = path.with_extension("missing");
    assert!(
        matches!(hash_file(&missing, 10), Err(ArtifactError::Io(source)) if source.kind() == io::ErrorKind::NotFound)
    );
}

#[cfg(unix)]
#[test]
fn following_reads_preserve_open_failure_before_size_admission() {
    use std::os::unix::fs::PermissionsExt as _;
    let fixture = crate::test_support::fixture();
    let path = fixture.path().join("inaccessible");
    std::fs::write(&path, b"oversized").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    // Privileged callers can bypass DAC; qualify ordering when opening refuses.
    if let Err(expected) = File::open(&path) {
        for result in [
            read_file(&path, 0).map(|_| ()),
            hash_file(&path, 0).map(|_| ()),
            read_file_prefix(&path, 0).map(|_| ()),
        ] {
            assert!(
                matches!(result, Err(ArtifactError::Io(error)) if error.kind() == expected.kind())
            );
        }
    } else {
        assert!(matches!(
            read_file(&path, 0),
            Err(ArtifactError::LimitExceeded { limit: 0 })
        ));
        assert!(matches!(
            hash_file(&path, 0),
            Err(ArtifactError::LimitExceeded { limit: 0 })
        ));
    }
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"oversized");
}

#[cfg(unix)]
#[test]
fn regular_opener_follows_links_and_keeps_the_selected_inode() {
    let fixture = crate::test_support::fixture();
    let path = fixture.path().join("source");
    let link = fixture.path().join("link");
    std::fs::write(&path, b"original").unwrap();
    std::os::unix::fs::symlink(&path, &link).unwrap();
    let file = open_regular_file(&link).unwrap();
    std::fs::rename(&path, fixture.path().join("retained")).unwrap();
    std::fs::write(&path, b"replacement").unwrap();
    assert_eq!(read_opened_file(file, 8).unwrap(), b"original");
    assert_eq!(read_file(&link, 11).unwrap(), b"replacement");
    assert_eq!(read_file_prefix(&link, 3).unwrap(), (b"rep".to_vec(), 11));
    assert!(matches!(
        open_regular_file(fixture.path()),
        Err(ArtifactError::NotRegularFile)
    ));
    assert!(matches!(
        open_regular_file(&fixture.path().join("missing")),
        Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::NotFound
    ));
    std::fs::write(&path, []).unwrap();
    assert_eq!(
        read_opened_file(open_regular_file(&link).unwrap(), 0).unwrap(),
        [] as [u8; 0]
    );
}

#[cfg(unix)]
#[test]
fn regular_opener_refuses_a_fifo_before_waiting_for_a_writer() {
    use rustix::fs::{Mode, OFlags, open};
    use std::{sync::mpsc, thread, time::Duration};
    let fixture = crate::test_support::fixture();
    let fifo = fixture.path().join("fifo");
    crate::test_support::create_fifo(&fifo, Some(0o600)).unwrap();
    let path = fifo.clone();
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        sender
            .send(
                matches!(open_regular_file(&path), Err(ArtifactError::NotRegularFile))
                    && matches!(
                        read_file_prefix(&path, 0),
                        Err(ArtifactError::NotRegularFile)
                    ),
            )
            .unwrap();
    });
    let result = receiver.recv_timeout(Duration::from_secs(2));
    // Unblock a regressed blocking open before reporting the original timeout.
    let _writer = result.is_err().then(|| {
        File::from(
            open(
                &fifo,
                OFlags::RDWR | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .unwrap(),
        )
    });
    reader.join().unwrap();
    assert!(result.unwrap());
}

#[test]
fn file_prefix_preserves_source_and_reports_observed_length() {
    let fixture = crate::test_support::fixture();
    let path = fixture.path().join("source");
    std::fs::write(&path, b"original").unwrap();
    for limit in [0, 3, 8, 32] {
        let (bytes, length) = read_file_prefix(&path, limit).unwrap();
        assert_eq!(bytes, b"original"[..limit.min(8)]);
        assert_eq!(length, 8);
    }
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
    assert!(matches!(
        read_file_prefix(fixture.path(), 0),
        Err(ArtifactError::NotRegularFile)
    ));
    assert!(
        matches!(read_file_prefix(&path.with_extension("missing"), 0), Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::NotFound)
    );
    File::create(&path)
        .unwrap()
        .set_len(4 * 1024 * 1024 * 1024)
        .unwrap();
    let (bytes, length) = read_file_prefix(&path, 16).unwrap();
    assert_eq!(bytes, [0; 16]);
    assert_eq!(length, 4 * 1024 * 1024 * 1024);
}
