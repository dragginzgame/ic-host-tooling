use super::*;
use std::io::Cursor;

#[test]
fn io_projection_preserves_native_codes_and_typed_artifact_causes() {
    let native = io::Error::from_raw_os_error(2);
    let projected = io::Error::from(ArtifactError::Io(native));
    assert_eq!(projected.raw_os_error(), Some(2));
    for error in [
        ArtifactError::NotRegularFile,
        ArtifactError::LimitExceeded { limit: 42 },
    ] {
        let projected = io::Error::from(error);
        assert!(matches!(
            projected.get_ref().unwrap().downcast_ref::<ArtifactError>(),
            Some(ArtifactError::NotRegularFile | ArtifactError::LimitExceeded { limit: 42 })
        ));
    }
}

#[test]
fn sha256_matches_known_vectors_and_round_trips_authority() {
    for (bytes, hex) in [
        (
            b"".as_slice(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".as_slice(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ] {
        let expected: Sha256Digest = hex.parse().unwrap();
        assert_eq!(Sha256Digest::compute(bytes), expected);
        assert_eq!(expected.to_string(), hex);
        assert_eq!(Sha256Digest::from_bytes(*expected.as_bytes()), expected);
        assert_eq!(
            hash_reader(bytes, bytes.len() as u64).unwrap(),
            ArtifactIdentity {
                bytes: bytes.len() as u64,
                sha256: expected
            }
        );
    }
    assert_eq!(
        "abc".parse::<Sha256Digest>(),
        Err(DigestParseError::Length { actual: 3 })
    );
    assert_eq!(
        "A".repeat(64).parse::<Sha256Digest>(),
        Err(DigestParseError::Digit { offset: 0 })
    );
    assert!(matches!(
        "é".repeat(32).parse::<Sha256Digest>(),
        Err(DigestParseError::Digit { .. })
    ));
}

#[test]
fn streaming_hashes_multiple_buffers_and_preserves_mismatch_identity() {
    let bytes = vec![42; 40_001];
    let expected = Sha256Digest::compute(&bytes);
    let actual = verify_reader(bytes.as_slice(), 40_001, expected).unwrap();
    assert_eq!(actual.bytes, 40_001);
    let wrong = Sha256Digest::compute(b"wrong");
    assert!(matches!(verify_reader(bytes.as_slice(), 40_001, wrong),
        Err(ArtifactError::DigestMismatch { expected, actual: observed }) if expected == wrong && observed == actual));
}

#[test]
fn byte_limits_are_inclusive_and_consume_only_one_overflow_byte() {
    let mut stream = Cursor::new(vec![9; 40_000]);
    assert!(matches!(
        hash_reader(&mut stream, 16_385),
        Err(ArtifactError::LimitExceeded { limit: 16_385 })
    ));
    assert_eq!(stream.position(), 16_386);
    let mut stream = Cursor::new(b"abc");
    assert!(matches!(
        hash_reader(&mut stream, 0),
        Err(ArtifactError::LimitExceeded { limit: 0 })
    ));
    assert_eq!(stream.position(), 1);
    assert_eq!(hash_reader(b"".as_slice(), 0).unwrap().bytes, 0);
    assert_eq!(hash_reader(b"abc".as_slice(), u64::MAX).unwrap().bytes, 3);
}

#[test]
fn interrupted_reads_resume_and_other_io_errors_stay_typed() {
    struct Interrupted(bool);
    impl Read for Interrupted {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if !self.0 {
                self.0 = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            b"abc".as_slice().read(buffer)
        }
    }
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    // Take bounds this synthetic reader, which emits the same bytes on every read.
    assert_eq!(
        hash_reader(Interrupted(false).take(3), 3).unwrap().sha256,
        Sha256Digest::compute(b"abc")
    );
    assert!(
        matches!(hash_reader(Broken, 10), Err(ArtifactError::Io(source)) if source.kind() == io::ErrorKind::PermissionDenied)
    );
}

#[test]
fn owned_stream_reads_are_bounded_and_preserve_io_failure_types() {
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    let mut stream = Cursor::new(vec![7; 40_000]);
    assert!(matches!(
        read_reader(&mut stream, 16_385),
        Err(ArtifactError::LimitExceeded { limit: 16_385 })
    ));
    assert_eq!(stream.position(), 16_386);
    assert_eq!(read_reader(b"abc".as_slice(), 3).unwrap(), b"abc");
    assert_eq!(read_reader(b"".as_slice(), 0).unwrap(), b"");
    assert!(
        matches!(read_reader(Broken, 10), Err(ArtifactError::Io(source))
        if source.kind() == io::ErrorKind::PermissionDenied)
    );
}

#[test]
fn impossible_reader_counts_fail_before_hashing_or_storage() {
    struct InvalidReader;
    impl Read for InvalidReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            Ok(buffer.len() + 1)
        }
    }
    for limit in [0, 3, 40_000, u64::MAX] {
        assert!(matches!(hash_reader(InvalidReader, limit),
            Err(ArtifactError::Io(source)) if source.kind() == io::ErrorKind::InvalidData));
    }
    for limit in [0, 3, 40_000] {
        assert!(matches!(read_reader(InvalidReader, limit),
            Err(ArtifactError::Io(source)) if source.kind() == io::ErrorKind::InvalidData));
    }
}
