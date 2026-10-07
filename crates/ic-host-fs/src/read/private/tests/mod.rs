use super::*;
use crate::test_support::Fixture;
use std::{
    error::Error as _,
    fs,
    io::Cursor,
    os::unix::fs::{PermissionsExt as _, symlink},
};

#[test]
fn missing_exact_and_empty_private_reads_do_not_mutate_sources() {
    let fixture = Fixture::new();
    let path = fixture.root.join("key");
    assert!(read_private_bytes::<4>(&path).unwrap().is_none());
    assert!(!path.exists());
    crate::durable::create_private_bytes_with_parents(&path, b"key!").unwrap();
    assert_eq!(read_private_bytes::<4>(&path).unwrap(), Some(*b"key!"));
    assert_eq!(fs::read(&path).unwrap(), b"key!");
    fs::write(&path, []).unwrap();
    assert_eq!(read_private_bytes::<0>(&path).unwrap(), Some([]));
}

#[test]
fn unsafe_private_files_remain_errors_without_regeneration() {
    let fixture = Fixture::new();
    let path = fixture.root.join("key");
    crate::durable::create_private_bytes_with_parents(&path, b"key!").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    assert!(matches!(
        read_private_bytes::<4>(&path),
        Err(PrivateFileReadError::Permissions { mode: 0o640 })
    ));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::hard_link(&path, fixture.root.join("other-name")).unwrap();
    assert!(matches!(
        read_private_bytes::<4>(&path),
        Err(PrivateFileReadError::LinkCount { actual: 2 })
    ));
    fs::remove_file(fixture.root.join("other-name")).unwrap();
    for length in [3, 5] {
        fs::write(&path, vec![7; length]).unwrap();
        assert!(matches!(
            read_private_bytes::<4>(&path),
            Err(PrivateFileReadError::Length { expected: 4 })
        ));
        assert_eq!(fs::metadata(&path).unwrap().len(), length as u64);
    }
    for (name, target) in [("link", "key"), ("dangling", "missing")] {
        let link = fixture.root.join(name);
        symlink(target, &link).unwrap();
        assert!(matches!(
            read_private_bytes::<4>(&link),
            Err(PrivateFileReadError::Read(ArtifactError::NotRegularFile))
        ));
    }
    assert!(matches!(
        read_private_bytes::<4>(&fixture.root),
        Err(PrivateFileReadError::Read(ArtifactError::NotRegularFile))
    ));
}

#[test]
fn private_io_errors_preserve_the_original_cause() {
    let error = read_private_bytes::<4>(Path::new("invalid\0path")).unwrap_err();
    assert!(
        matches!(&error, PrivateFileReadError::Read(ArtifactError::Io(source))
        if source.kind() == io::ErrorKind::InvalidInput)
    );
    let artifact = error
        .source()
        .unwrap()
        .downcast_ref::<ArtifactError>()
        .unwrap();
    assert!(
        artifact
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .is_some()
    );
}

#[test]
fn private_stream_growth_shrink_and_interruption_keep_distinct_outcomes() {
    struct Trailer {
        interrupted: bool,
        failure: Option<io::ErrorKind>,
    }
    impl Read for Trailer {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.failure.map_or(Ok(0), |kind| Err(kind.into()))
        }
    }
    assert!(matches!(
        read_exact_private::<4>(&mut Cursor::new(b"longer")),
        Err(PrivateFileReadError::Length { expected: 4 })
    ));
    assert!(matches!(read_exact_private::<4>(&mut Cursor::new(b"cut")),
        Err(PrivateFileReadError::Read(ArtifactError::Io(error))) if error.kind() == io::ErrorKind::UnexpectedEof));
    for failure in [None, Some(io::ErrorKind::PermissionDenied)] {
        let mut reader = Cursor::new(b"key!").chain(Trailer {
            interrupted: false,
            failure,
        });
        let result = read_exact_private::<4>(&mut reader);
        if failure.is_none() {
            assert_eq!(result.unwrap(), *b"key!");
        } else {
            assert!(
                matches!(result, Err(PrivateFileReadError::Read(ArtifactError::Io(error)))
                if error.kind() == io::ErrorKind::PermissionDenied)
            );
        }
    }
}
