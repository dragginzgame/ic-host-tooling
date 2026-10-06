use super::*;
use std::io::Cursor;

#[test]
fn copying_hashing_and_reading_share_the_same_complete_stream_identity() {
    let bytes = vec![37; 40_001];
    let mut output = Vec::new();
    let copied = copy_reader(bytes.as_slice(), &mut output, 40_001).unwrap();
    assert_eq!(output, bytes);
    assert_eq!(
        copied,
        super::super::hash_reader(bytes.as_slice(), 40_001).unwrap()
    );
    assert_eq!(
        super::super::read_reader(bytes.as_slice(), 40_001).unwrap(),
        output
    );
    assert_eq!(
        copy_reader(b"".as_slice(), &mut Vec::new(), 0)
            .unwrap()
            .bytes,
        0
    );
    assert_eq!(
        copy_reader(b"abc".as_slice(), &mut Vec::new(), u64::MAX)
            .unwrap()
            .bytes,
        3
    );
}

#[test]
fn overflow_observes_one_extra_byte_and_preserves_only_admitted_output_chunks() {
    let mut input = Cursor::new(vec![7; 40_000]);
    let mut output = Vec::new();
    assert!(matches!(
        copy_reader(&mut input, &mut output, 16_385),
        Err(CopyError::Input(ArtifactError::LimitExceeded {
            limit: 16_385
        }))
    ));
    assert_eq!(input.position(), 16_386);
    assert_eq!(output, vec![7; 16_384]);
    let mut input = Cursor::new(b"abc");
    let mut output = Vec::new();
    assert!(matches!(
        copy_reader(&mut input, &mut output, 0),
        Err(CopyError::Input(ArtifactError::LimitExceeded { limit: 0 }))
    ));
    assert_eq!(input.position(), 1);
    assert_eq!(output, [] as [u8; 0]);
}

#[test]
fn read_errors_stay_distinct_from_sink_errors_and_preserve_partial_effects() {
    struct BrokenReader;
    impl Read for BrokenReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    struct BrokenWriter(Vec<u8>);
    impl Write for BrokenWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if !self.0.is_empty() {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            self.0.extend_from_slice(&bytes[..2]);
            Ok(2)
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("copy must not flush its staging sink")
        }
    }
    let mut output = b"existing".to_vec();
    assert!(matches!(copy_reader(BrokenReader, &mut output, 3),
        Err(CopyError::Input(ArtifactError::Io(source))) if source.kind() == io::ErrorKind::PermissionDenied));
    assert_eq!(output, b"existing");
    let mut output = BrokenWriter(Vec::new());
    assert!(matches!(copy_reader(b"abcdef".as_slice(), &mut output, 6),
        Err(CopyError::Output(source)) if source.kind() == io::ErrorKind::BrokenPipe));
    assert_eq!(output.0, b"ab");
}

#[test]
fn interrupted_reads_and_short_writes_complete_without_changing_the_digest() {
    struct InterruptedReader {
        interrupted: bool,
        input: Cursor<Vec<u8>>,
    }
    impl Read for InterruptedReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.input.read(buffer)
        }
    }
    struct ShortWriter(Vec<u8>, bool);
    impl Write for ShortWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if !self.1 {
                self.1 = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            let count = bytes.len().min(2);
            self.0.extend_from_slice(&bytes[..count]);
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("copy must not flush its staging sink")
        }
    }
    let input = InterruptedReader {
        interrupted: false,
        input: Cursor::new(b"abcdef".to_vec()),
    };
    let mut output = ShortWriter(Vec::new(), false);
    let identity = copy_reader(input, &mut output, 6).unwrap();
    assert_eq!(output.0, b"abcdef");
    assert_eq!(identity.bytes, 6);
    assert_eq!(identity.sha256, Sha256Digest::compute(b"abcdef"));
}

#[test]
fn invalid_read_counts_do_not_reach_the_staging_sink() {
    struct InvalidReader;
    impl Read for InvalidReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            Ok(buffer.len() + 1)
        }
    }
    for limit in [0, 3, 40_000, u64::MAX] {
        let mut output = b"existing".to_vec();
        assert!(matches!(copy_reader(InvalidReader, &mut output, limit),
            Err(CopyError::Input(ArtifactError::Io(source)))
            if source.kind() == io::ErrorKind::InvalidData));
        assert_eq!(output, b"existing");
    }
}

#[test]
fn invalid_write_counts_preserve_the_sink_prefix_and_return_the_shared_error() {
    use crate::artifact::WriterError;

    struct InvalidWriter(Vec<u8>);
    impl Write for InvalidWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                self.0.extend_from_slice(&bytes[..2]);
                Ok(2)
            } else {
                Ok(bytes.len() + 1)
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("copy must not flush its staging sink")
        }
    }
    let mut output = InvalidWriter(Vec::new());
    let CopyError::Output(error) = copy_reader(b"abcdef".as_slice(), &mut output, 6).unwrap_err()
    else {
        panic!("an invalid sink count must be an output failure")
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(&WriterError::InvalidWriteCount {
            offered: 4,
            written: 5,
        })
    );
    assert_eq!(output.0, b"ab");
}
