use super::{
    hash_file_no_follow, open_no_follow, open_optional_no_follow, open_regular_file_no_follow,
    read_file_no_follow, read_optional_file_no_follow,
};
use crate::{
    read::{admit_file, hash_file, read_file, read_opened_file},
    test_support::fixture,
};
use ic_host_artifacts::artifact::{ArtifactError, ArtifactIdentity, Sha256Digest, hash_reader};
use rustix::fs::{Mode, OFlags, open};
use std::{
    fs::{self, File},
    io::{self, Seek as _, SeekFrom},
    os::unix::fs::symlink,
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
};

#[test]
fn exact_limits_and_empty_files_preserve_the_selected_bytes() {
    let fixture = fixture();
    let path = fixture.path().join("artifact");
    let bytes = vec![42; 40_001];
    fs::write(&path, &bytes).unwrap();
    assert_eq!(read_file_no_follow(&path, bytes.len()).unwrap(), bytes);
    assert_eq!(
        hash_file_no_follow(&path, bytes.len() as u64).unwrap(),
        ArtifactIdentity {
            bytes: bytes.len() as u64,
            sha256: Sha256Digest::compute(&bytes)
        }
    );
    assert_eq!(
        read_opened_file(File::open(&path).unwrap(), bytes.len()).unwrap(),
        bytes
    );
    assert!(matches!(
        read_file_no_follow(&path, bytes.len() - 1),
        Err(ArtifactError::LimitExceeded { limit: 40_000 })
    ));
    assert!(matches!(
        hash_file_no_follow(&path, bytes.len() as u64 - 1),
        Err(ArtifactError::LimitExceeded { limit: 40_000 })
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::write(&path, []).unwrap();
    assert_eq!(read_file_no_follow(&path, 0).unwrap(), [] as [u8; 0]);
    assert_eq!(
        hash_file_no_follow(&path, 0).unwrap(),
        ArtifactIdentity {
            bytes: 0,
            sha256: Sha256Digest::compute(&[])
        }
    );
    fs::write(&path, [1]).unwrap();
    assert!(matches!(
        read_file_no_follow(&path, 0),
        Err(ArtifactError::LimitExceeded { limit: 0 })
    ));
    assert!(matches!(
        hash_file_no_follow(&path, 0),
        Err(ArtifactError::LimitExceeded { limit: 0 })
    ));
}

#[test]
fn no_follow_descriptor_remains_bound_after_replacement_and_is_close_on_exec() {
    use std::os::unix::fs::MetadataExt as _;
    let fixture = fixture();
    let path = fixture.path().join("selected");
    fs::write(&path, b"original").unwrap();
    let file = open_regular_file_no_follow(&path).unwrap();
    let selected = file.metadata().unwrap();
    assert!(
        rustix::io::fcntl_getfd(&file)
            .unwrap()
            .contains(rustix::io::FdFlags::CLOEXEC)
    );
    fs::rename(&path, fixture.path().join("retained")).unwrap();
    fs::write(&path, b"replacement").unwrap();
    assert_ne!(selected.ino(), fs::metadata(&path).unwrap().ino());
    file.sync_all().unwrap();
    assert_eq!(read_opened_file(file, 8).unwrap(), b"original");
    assert_eq!(fs::read(&path).unwrap(), b"replacement");
}

#[test]
fn final_symlinks_are_rejected_without_changing_existing_following_reads() {
    let fixture = fixture();
    let target = fixture.path().join("target");
    let link = fixture.path().join("link");
    fs::write(&target, b"retained").unwrap();
    symlink("target", &link).unwrap();
    assert!(
        matches!(open_regular_file_no_follow(&link), Err(ArtifactError::Io(error))
        if error.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()))
    );
    assert!(
        matches!(read_file_no_follow(&link, 8), Err(ArtifactError::Io(error))
        if error.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()))
    );
    assert_eq!(read_file(&link, 8).unwrap(), b"retained");
    assert_eq!(
        hash_file(&link, 8).unwrap().sha256,
        Sha256Digest::compute(b"retained")
    );
    assert!(
        matches!(hash_file_no_follow(&link, 8), Err(ArtifactError::Io(error))
        if error.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()))
    );
    // Descriptor validation does not claim the opener rejected links.
    assert_eq!(
        read_opened_file(File::open(&link).unwrap(), 8).unwrap(),
        b"retained"
    );
    assert_eq!(fs::read(&target).unwrap(), b"retained");
    let dangling = fixture.path().join("dangling");
    symlink("missing", &dangling).unwrap();
    assert!(
        matches!(read_file_no_follow(&dangling, 8), Err(ArtifactError::Io(error))
        if error.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()))
    );
    assert!(
        matches!(hash_file_no_follow(&dangling, 8), Err(ArtifactError::Io(error))
        if error.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()))
    );
}

#[test]
fn ancestor_symlinks_remain_an_explicit_consumer_confinement_obligation() {
    let fixture = fixture();
    let actual = fixture.path().join("actual");
    fs::create_dir(&actual).unwrap();
    fs::write(actual.join("artifact"), b"data").unwrap();
    symlink("actual", fixture.path().join("alias")).unwrap();
    assert_eq!(
        read_file_no_follow(&fixture.path().join("alias/artifact"), 4).unwrap(),
        b"data"
    );
    assert_eq!(
        hash_file_no_follow(&fixture.path().join("alias/artifact"), 4)
            .unwrap()
            .sha256,
        Sha256Digest::compute(b"data")
    );
}

#[test]
fn fifo_without_a_writer_is_rejected_without_waiting_for_one() {
    let fixture = fixture();
    let path = fixture.path().join("fifo");
    crate::test_support::create_fifo(&path, Some(0o600)).unwrap();
    let (tx, rx) = mpsc::channel();
    let path_for_reader = path.clone();
    let reader = thread::spawn(move || {
        tx.send(read_file_no_follow(&path_for_reader, 1024))
            .unwrap();
        tx.send(hash_file_no_follow(&path_for_reader, 1024).map(|_| Vec::new()))
            .unwrap();
        tx.send(open_regular_file_no_follow(&path_for_reader).map(|_| Vec::new()))
            .unwrap();
    });
    // Bound a regression to a blocking FIFO open; this is a fixture deadline,
    // not a filesystem timing guarantee or performance assertion.
    for _ in 0..3 {
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(5))
                .expect("FIFO read/hash must not wait for a writer"),
            Err(ArtifactError::NotRegularFile)
        ));
    }
    reader.join().unwrap();
    let fd = open(
        &path,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    assert!(matches!(
        read_opened_file(File::from(fd), 1024),
        Err(ArtifactError::NotRegularFile)
    ));
}

#[test]
fn opened_directories_and_devices_are_rejected_before_reading() {
    let fixture = fixture();
    for path in [fixture.path(), Path::new("/dev/null")] {
        assert!(matches!(
            open_regular_file_no_follow(path),
            Err(ArtifactError::NotRegularFile)
        ));
        assert!(matches!(
            hash_file_no_follow(path, 1024),
            Err(ArtifactError::NotRegularFile)
        ));
    }
    assert!(matches!(
        read_file_no_follow(fixture.path(), 1024),
        Err(ArtifactError::NotRegularFile)
    ));
    assert!(matches!(
        read_opened_file(File::open(fixture.path()).unwrap(), 1024),
        Err(ArtifactError::NotRegularFile)
    ));
    assert!(matches!(
        read_file_no_follow(Path::new("/dev/null"), 1024),
        Err(ArtifactError::NotRegularFile)
    ));
}

#[test]
fn descriptor_identity_survives_path_replacement_and_reads_from_its_current_position() {
    let fixture = fixture();
    let path = fixture.path().join("artifact");
    let replacement = fixture.path().join("replacement");
    fs::write(&path, b"original").unwrap();
    let mut file = File::open(&path).unwrap();
    file.seek(SeekFrom::Start(3)).unwrap();
    fs::write(&replacement, b"different").unwrap();
    fs::rename(&replacement, &path).unwrap();
    assert_eq!(read_opened_file(file, 8).unwrap(), b"ginal");
    assert_eq!(fs::read(&path).unwrap(), b"different");
    let mut file = File::open(&path).unwrap();
    file.seek(SeekFrom::End(-1)).unwrap();
    // The descriptor's complete metadata length is the early bound, even when
    // the cursor has fewer bytes remaining.
    assert!(matches!(
        read_opened_file(file, 1),
        Err(ArtifactError::LimitExceeded { limit: 1 })
    ));
}

#[test]
fn missing_and_invalid_paths_preserve_structured_open_failures() {
    let fixture = fixture();
    assert!(
        matches!(hash_file_no_follow(&fixture.path().join("missing"), 1024),
        Err(ArtifactError::Io(error)) if error.raw_os_error() == Some(rustix::io::Errno::NOENT.raw_os_error()))
    );
    assert!(
        matches!(hash_file_no_follow(Path::new("invalid\0path"), 1024),
        Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput)
    );
    assert!(
        matches!(read_file_no_follow(&fixture.path().join("missing"), 1024),
        Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::NotFound)
    );
    assert!(
        matches!(read_file_no_follow(Path::new("invalid\0path"), 1024),
        Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput)
    );
}

#[test]
fn streaming_hash_rejects_growth_after_descriptor_metadata_admission() {
    let fixture = fixture();
    let path = fixture.path().join("growing");
    fs::write(&path, b"1234").unwrap();
    // Coordinate growth between the production opener/metadata boundary and
    // shared stream traversal, without a timing race or a production test hook.
    let file = open_no_follow(&path).unwrap();
    admit_file(&file, 4).unwrap();
    fs::write(&path, b"12345").unwrap();
    assert!(matches!(
        hash_reader(file, 4),
        Err(ArtifactError::LimitExceeded { limit: 4 })
    ));
    assert_eq!(fs::read(&path).unwrap(), b"12345");
}

#[test]
fn bounded_regular_read_rejects_existing_and_growing_oversize_content() {
    let fixture = fixture();
    let path = fixture.path().join("object");
    fs::write(&path, b"12345").unwrap();
    assert!(matches!(
        read_optional_file_no_follow(&path, 4),
        Err(ArtifactError::LimitExceeded { limit: 4 })
    ));
    fs::write(&path, b"1234").unwrap();
    let file = open_optional_no_follow(&path).unwrap().unwrap();
    fs::write(&path, b"12345").unwrap();
    assert!(matches!(
        read_opened_file(file, 4),
        Err(ArtifactError::LimitExceeded { limit: 4 })
    ));
}

#[test]
fn bounded_regular_read_preserves_exact_bytes_and_rejects_links() {
    let fixture = fixture();
    let path = fixture.path().join("object");
    assert!(read_optional_file_no_follow(&path, 4).unwrap().is_none());
    fs::write(&path, b"1234").unwrap();
    assert_eq!(
        read_optional_file_no_follow(&path, 4).unwrap(),
        Some(b"1234".to_vec())
    );
    for (name, target) in [("linked", "object"), ("dangling", "missing")] {
        let link = fixture.path().join(name);
        symlink(target, &link).unwrap();
        assert!(matches!(
            read_optional_file_no_follow(&link, 4),
            Err(ArtifactError::NotRegularFile)
        ));
    }
    assert!(matches!(
        read_optional_file_no_follow(fixture.path(), 4),
        Err(ArtifactError::NotRegularFile)
    ));
    assert!(
        matches!(read_optional_file_no_follow(Path::new("invalid\0path"), 4),
        Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput)
    );
    fs::write(&path, []).unwrap();
    assert_eq!(
        read_optional_file_no_follow(&path, 0).unwrap(),
        Some(Vec::new())
    );
}
