use super::{BoundedWriter, WriterError};
use std::io::{self, IoSlice, Write};

#[test]
fn limits_are_inclusive_and_rejected_buffers_do_not_reach_the_sink() {
    let mut writer = BoundedWriter::new(Vec::new(), 3);
    let error = writer.write(b"four").unwrap_err();
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(&WriterError::LimitExceeded { limit: 3 })
    );
    assert_eq!(writer.bytes_written(), 0);
    writer.write_all(b"abc").unwrap();
    assert_eq!(writer.bytes_written(), 3);
    assert_eq!(writer.write(b"").unwrap(), 0);
    assert!(writer.write(b"d").is_err());
    assert!(writer.limit_exceeded());
    assert_eq!(writer.into_inner(), b"abc");

    let mut zero = BoundedWriter::new(Vec::new(), 0);
    zero.write_all(b"").unwrap();
    assert!(zero.write(b"a").is_err());
    assert_eq!(zero.into_inner(), [] as [u8; 0]);
}

struct ShortWriter {
    bytes: Vec<u8>,
    fail: bool,
    zero: bool,
    flushed: bool,
}

impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        if self.zero {
            return Ok(0);
        }
        let count = bytes.len().min(2);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushed = true;
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
}

#[test]
fn short_writes_and_failures_preserve_the_accepted_prefix() {
    let sink = ShortWriter {
        bytes: Vec::new(),
        fail: false,
        zero: false,
        flushed: false,
    };
    let mut writer = BoundedWriter::new(sink, 6);
    writer.write_all(b"abcdef").unwrap();
    assert_eq!(writer.bytes_written(), 6);
    assert!(!writer.limit_exceeded());
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::BrokenPipe
    );
    let mut sink = writer.into_inner();
    assert!(sink.flushed);
    assert_eq!(sink.bytes, b"abcdef");
    sink.fail = true;
    let mut writer = BoundedWriter::new(sink, 9);
    assert_eq!(
        writer.write(b"xyz").unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(writer.bytes_written(), 0);
    assert!(!writer.limit_exceeded());
    let mut sink = writer.into_inner();
    sink.fail = false;
    sink.zero = true;
    let mut writer = BoundedWriter::new(sink, 9);
    assert_eq!(
        writer.write_all(b"xyz").unwrap_err().kind(),
        io::ErrorKind::WriteZero
    );
    assert_eq!(writer.bytes_written(), 0);
    assert_eq!(writer.into_inner().bytes, b"abcdef");
}

#[test]
fn recovering_the_sink_does_not_flush_or_discard_it() {
    let sink = ShortWriter {
        bytes: b"prefix".to_vec(),
        fail: false,
        zero: false,
        flushed: false,
    };
    let writer = BoundedWriter::new(sink, 10);
    assert_eq!(writer.bytes_written(), 0);
    let sink = writer.into_inner();
    assert!(!sink.flushed);
    assert_eq!(sink.bytes, b"prefix");
}

#[test]
fn invalid_sink_write_counts_are_typed_failures_without_count_overflow() {
    struct InvalidWriter;
    impl Write for InvalidWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len() + 1)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = BoundedWriter::new(InvalidWriter, 1);
    let error = writer.write(b"a").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(&WriterError::InvalidWriteCount {
            offered: 1,
            written: 2
        })
    );
    assert_eq!(writer.bytes_written(), 0);
    assert!(!writer.limit_exceeded());
}

#[test]
fn vectored_writes_remain_bounded_by_the_standard_write_contract() {
    let mut writer = BoundedWriter::new(Vec::new(), 3);
    let slices = [
        IoSlice::new(b""),
        IoSlice::new(b"abc"),
        IoSlice::new(b"def"),
    ];
    assert_eq!(writer.write_vectored(&slices).unwrap(), 3);
    assert!(writer.write_vectored(&[IoSlice::new(b"d")]).is_err());
    assert_eq!(writer.bytes_written(), 3);
    assert_eq!(writer.into_inner(), b"abc");
}

#[test]
fn serialized_json_can_be_counted_or_hashed_without_an_encoded_copy() {
    use crate::artifact::Sha256Digest;
    use sha2::{Digest, Sha256};

    let value = serde_json::json!({"rows": ["a", "b"], "schema_version": 1});
    let expected = br#"{"rows":["a","b"],"schema_version":1}"#;
    let limit = u64::try_from(expected.len()).unwrap();
    let mut counter = BoundedWriter::new(io::sink(), limit);
    serde_json::to_writer(&mut counter, &value).unwrap();
    assert_eq!(counter.bytes_written(), limit);

    let mut hasher = BoundedWriter::new(Sha256::new(), limit);
    serde_json::to_writer(&mut hasher, &value).unwrap();
    assert_eq!(hasher.bytes_written(), limit);
    assert_eq!(
        Sha256Digest::from_bytes(hasher.into_inner().finalize().into()),
        Sha256Digest::compute(expected)
    );

    let mut rejected = BoundedWriter::new(Vec::new(), limit - 1);
    let error = serde_json::to_writer(&mut rejected, &value).unwrap_err();
    assert_eq!(error.io_error_kind(), Some(io::ErrorKind::Other));
    assert!(rejected.limit_exceeded());
    let count = rejected.bytes_written();
    let partial = rejected.into_inner();
    assert_eq!(u64::try_from(partial.len()).unwrap(), count);
    assert!(expected.starts_with(&partial));
}
