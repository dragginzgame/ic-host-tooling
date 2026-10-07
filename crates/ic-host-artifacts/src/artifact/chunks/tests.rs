use super::*;
use std::io::{self, Cursor};

struct Fragmented {
    input: Cursor<Vec<u8>>,
    fragment: usize,
    interrupt: bool,
}

impl Read for Fragmented {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if std::mem::replace(&mut self.interrupt, false) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        self.interrupt = true;
        let count = buffer.len().min(self.fragment);
        self.input.read(&mut buffer[..count])
    }
}

#[test]
fn chunk_boundaries_and_whole_identity_ignore_read_fragmentation() {
    let input = (0_u8..251).cycle().take(40_001).collect::<Vec<_>>();
    for chunk_size in [1, 3, 16_384, 16_385, input.len(), usize::MAX] {
        let expected = input
            .chunks(chunk_size)
            .map(Sha256Digest::compute)
            .collect::<Vec<_>>();
        for fragment in [1, 17, 16_384] {
            let reader = Fragmented {
                input: Cursor::new(input.clone()),
                fragment,
                interrupt: true,
            };
            let (chunks, whole) = chunk_digests(
                reader,
                NonZeroUsize::new(chunk_size).unwrap(),
                input.len() as u64,
                expected.len(),
            )
            .unwrap();
            assert_eq!(chunks, expected);
            assert_eq!(
                whole,
                ArtifactIdentity {
                    bytes: input.len() as u64,
                    sha256: Sha256Digest::compute(&input)
                }
            );
        }
    }
}

#[test]
fn empty_input_and_exact_multiples_have_no_empty_final_chunk() {
    let size = NonZeroUsize::new(3).unwrap();
    let (chunks, whole) = chunk_digests(b"".as_slice(), size, 0, 0).unwrap();
    assert_eq!(chunks, [] as [Sha256Digest; 0]);
    assert_eq!(
        whole,
        ArtifactIdentity {
            bytes: 0,
            sha256: Sha256Digest::compute(b"")
        }
    );
    for input in [b"abc".as_slice(), b"abcdef", b"abcdefg"] {
        let expected = input
            .chunks(3)
            .map(Sha256Digest::compute)
            .collect::<Vec<_>>();
        let (chunks, whole) =
            chunk_digests(input, size, input.len() as u64, expected.len()).unwrap();
        assert_eq!(chunks, expected);
        assert_eq!(whole.sha256, Sha256Digest::compute(input));
    }
}

#[test]
fn byte_and_chunk_budgets_reject_without_partial_identities() {
    let size = NonZeroUsize::new(3).unwrap();
    let mut reader = Cursor::new(b"abcdefg");
    assert!(matches!(
        chunk_digests(&mut reader, size, 6, 3),
        Err(ChunkDigestError::Input(ArtifactError::LimitExceeded {
            limit: 6
        }))
    ));
    assert_eq!(reader.position(), 7);
    for (input, count) in [
        (b"a".as_slice(), 0),
        (b"abcd".as_slice(), 1),
        (b"abcdefg".as_slice(), 2),
    ] {
        assert!(matches!(chunk_digests(input, size, 100, count),
            Err(ChunkDigestError::ChunkLimit { limit }) if limit == count));
    }
}

#[test]
fn source_errors_after_complete_chunks_remain_typed() {
    struct InvalidReader;
    impl Read for InvalidReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            Ok(buffer.len() + 1)
        }
    }
    struct FailingReader(bool);
    impl Read for FailingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if std::mem::replace(&mut self.0, false) {
                buffer[..3].copy_from_slice(b"abc");
                Ok(3)
            } else {
                Err(io::Error::from_raw_os_error(13))
            }
        }
    }
    let error =
        chunk_digests(FailingReader(true), NonZeroUsize::new(3).unwrap(), 100, 4).unwrap_err();
    assert!(matches!(error,
        ChunkDigestError::Input(ArtifactError::Io(ref source)) if source.raw_os_error() == Some(13)));
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<ArtifactError>()
    );

    assert!(
        matches!(chunk_digests(InvalidReader, NonZeroUsize::new(3).unwrap(), 100, 4),
        Err(ChunkDigestError::Input(ArtifactError::Io(source))) if source.kind() == io::ErrorKind::InvalidData)
    );
}
