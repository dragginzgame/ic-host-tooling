use super::*;
use ic_host_artifacts::artifact::Sha256Digest;
use std::io;

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
fn regular_opener_follows_links_and_keeps_the_selected_inode() {
    let fixture = crate::test_support::Fixture::new();
    let path = fixture.root.join("source");
    let link = fixture.root.join("link");
    std::fs::write(&path, b"original").unwrap();
    std::os::unix::fs::symlink(&path, &link).unwrap();
    let file = open_regular_file(&link).unwrap();
    std::fs::rename(&path, fixture.root.join("retained")).unwrap();
    std::fs::write(&path, b"replacement").unwrap();
    assert_eq!(read_opened_file(file, 8).unwrap(), b"original");
    assert_eq!(read_file(&link, 11).unwrap(), b"replacement");
    assert!(matches!(
        open_regular_file(&fixture.root),
        Err(ArtifactError::NotRegularFile)
    ));
    assert!(matches!(
        open_regular_file(&fixture.root.join("missing")),
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
    use rustix::fs::{CWD, Mode, OFlags, mkfifoat, open};
    use std::{sync::mpsc, thread, time::Duration};
    let fixture = crate::test_support::Fixture::new();
    let fifo = fixture.root.join("fifo");
    mkfifoat(CWD, &fifo, Mode::RUSR | Mode::WUSR).unwrap();
    let path = fifo.clone();
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        sender
            .send(matches!(
                open_regular_file(&path),
                Err(ArtifactError::NotRegularFile)
            ))
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
