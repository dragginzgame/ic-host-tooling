use super::HashingWriter;
use crate::artifact::{Sha256Digest, WriterError};
use std::io::{self, IoSlice, Write};

#[test]
fn empty_and_chunked_output_return_the_sink_and_exact_identity() {
    let writer = HashingWriter::new(Vec::<u8>::new(), 0);
    let (bytes, identity) = writer.into_parts();
    assert_eq!(bytes, [] as [u8; 0]);
    assert_eq!(identity.bytes, 0);
    assert_eq!(identity.sha256, Sha256Digest::compute(b""));
    let mut writer = HashingWriter::new(Vec::new(), 6);
    writer.write_all(b"ab").unwrap();
    writer.write_all(b"cdef").unwrap();
    assert_eq!(writer.bytes_written(), 6);
    let (bytes, identity) = writer.into_parts();
    assert_eq!(bytes, b"abcdef");
    assert_eq!(identity.bytes, 6);
    assert_eq!(identity.sha256, Sha256Digest::compute(&bytes));
}

#[derive(Default)]
struct ShortSink {
    bytes: Vec<u8>,
    interrupt: bool,
    fail_after: Option<usize>,
    flushes: usize,
}

impl Write for ShortSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if std::mem::take(&mut self.interrupt) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self
            .fail_after
            .is_some_and(|limit| self.bytes.len() >= limit)
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let count = bytes.len().min(2);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        Err(io::ErrorKind::BrokenPipe.into())
    }
}

#[test]
fn interrupted_short_and_vectored_writes_hash_only_accepted_bytes() {
    let sink = ShortSink {
        interrupt: true,
        ..ShortSink::default()
    };
    let mut writer = HashingWriter::new(sink, 6);
    writer.write_all(b"abcd").unwrap();
    let accepted = writer
        .write_vectored(&[IoSlice::new(b"ef"), IoSlice::new(b"g")])
        .unwrap();
    assert_eq!(accepted, 2);
    let (sink, identity) = writer.into_parts();
    assert_eq!(sink.bytes, b"abcdef");
    assert_eq!(identity.bytes, 6);
    assert_eq!(identity.sha256, Sha256Digest::compute(&sink.bytes));
    assert_eq!(sink.flushes, 0);
}

#[test]
fn failed_write_and_flush_keep_the_observed_prefix_without_finalization_effects() {
    let sink = ShortSink {
        fail_after: Some(2),
        ..ShortSink::default()
    };
    let mut writer = HashingWriter::new(sink, 9);
    assert_eq!(
        writer.write_all(b"abcdef").unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        writer.flush().unwrap_err().kind(),
        io::ErrorKind::BrokenPipe
    );
    let (sink, identity) = writer.into_parts();
    assert_eq!(sink.bytes, b"ab");
    assert_eq!(sink.flushes, 1);
    assert_eq!(identity.bytes, 2);
    assert_eq!(identity.sha256, Sha256Digest::compute(b"ab"));
}

#[test]
fn allowance_rejection_keeps_sink_and_hash_unchanged() {
    let mut writer = HashingWriter::new(Vec::new(), 3);
    writer.write_all(b"abc").unwrap();
    let error = writer.write(b"d").unwrap_err();
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(&WriterError::LimitExceeded { limit: 3 })
    );
    assert!(writer.limit_exceeded());
    assert_eq!(writer.write(b"").unwrap(), 0);
    let (bytes, identity) = writer.into_parts();
    assert_eq!(bytes, b"abc");
    assert_eq!(identity.bytes, 3);
    assert_eq!(identity.sha256, Sha256Digest::compute(b"abc"));
}

struct InvalidSink;
impl Write for InvalidSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(bytes.len() + 1)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn impossible_sink_counts_are_typed_errors_without_advancing_the_identity() {
    let mut writer = HashingWriter::new(InvalidSink, 2);
    let error = writer.write(b"ab").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(&WriterError::InvalidWriteCount {
            offered: 2,
            written: 3
        })
    );
    let (_, identity) = writer.into_parts();
    assert_eq!(identity.bytes, 0);
    assert_eq!(identity.sha256, Sha256Digest::compute(b""));
}

struct FailedProducer;
impl serde::Serialize for FailedProducer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq as _;
        let mut output = serializer.serialize_seq(Some(2))?;
        output.serialize_element("partial")?;
        Err(serde::ser::Error::custom("producer failed"))
    }
}

#[test]
fn serialization_success_is_required_before_accepting_a_complete_artifact() {
    let mut published = b"previous artifact".to_vec();
    let mut staging = HashingWriter::new(Vec::new(), 100);
    let result = serde_json::to_writer(&mut staging, &FailedProducer);
    let (bytes, prefix) = staging.into_parts();
    if result.is_ok() {
        published.clone_from(&bytes);
    }
    assert!(result.is_err());
    assert_eq!(published, b"previous artifact");
    assert_ne!(bytes, [] as [u8; 0]);
    assert_eq!(prefix.bytes, bytes.len() as u64);
    assert_eq!(prefix.sha256, Sha256Digest::compute(&bytes));
}
