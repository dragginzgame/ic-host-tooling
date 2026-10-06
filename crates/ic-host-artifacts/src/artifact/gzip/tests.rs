use super::*;
use flate2::{Compression, GzBuilder};
use std::io::Write as _;

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

#[test]
fn encoded_member_is_repeatable_and_preserves_the_existing_zero_timestamp_format() {
    let input = b"artifact input with repeated bytes repeated bytes repeated bytes";
    let mut reference = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::best());
    reference.write_all(input).unwrap();
    let reference = reference.finish().unwrap();
    for _ in 0..2 {
        let mut output = Vec::new();
        encode_gzip(
            input,
            &mut output,
            Compression::best(),
            reference.len() as u64,
        )
        .unwrap();
        assert_eq!(output, reference);
        assert_eq!(&output[4..8], &[0, 0, 0, 0]);
        assert_eq!(
            decode_gzip(&output, output.len(), input.len()).unwrap(),
            input
        );
    }
}

#[test]
fn compressed_budget_includes_header_and_trailer_and_preserves_sink_errors() {
    use super::super::WriterError;
    struct FailedSink;
    impl std::io::Write for FailedSink {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::PermissionDenied.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut complete = Vec::new();
    encode_gzip(b"", &mut complete, Compression::fast(), 100).unwrap();
    let mut partial = Vec::new();
    let error = encode_gzip(
        b"",
        &mut partial,
        Compression::fast(),
        complete.len() as u64 - 1,
    )
    .unwrap_err();
    assert!(matches!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(WriterError::LimitExceeded { .. })
    ));
    assert!(partial.len() < complete.len());
    let error = encode_gzip(b"payload", FailedSink, Compression::none(), 100).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn inclusive_compressed_and_decoded_limits_preserve_input() {
    let payload = vec![42; 40_001];
    let compressed = gzip(&payload);
    let original = compressed.clone();
    assert_eq!(
        decode_gzip(&compressed, compressed.len(), payload.len()).unwrap(),
        payload
    );
    assert!(
        matches!(decode_gzip(&compressed, compressed.len() - 1, payload.len()),
        Err(GzipError::InputLimit { actual, limit })
            if actual == compressed.len() && limit == compressed.len() - 1)
    );
    assert!(matches!(
        decode_gzip(&compressed, compressed.len(), payload.len() - 1),
        Err(GzipError::Decode(ArtifactError::LimitExceeded {
            limit: 40_000
        }))
    ));
    assert_eq!(compressed, original);
}

#[test]
fn empty_payloads_and_predecode_input_limits_are_distinct() {
    let compressed = gzip(b"");
    assert_eq!(
        decode_gzip(&compressed, compressed.len(), 0).unwrap(),
        [] as [u8; 0]
    );
    assert!(matches!(
        decode_gzip(&[], 0, 0),
        Err(GzipError::Decode(ArtifactError::Io(_)))
    ));
    assert!(matches!(
        decode_gzip(b"invalid gzip", 0, 0),
        Err(GzipError::InputLimit {
            actual: 12,
            limit: 0
        })
    ));
    let nonempty = gzip(b"a");
    assert!(matches!(
        decode_gzip(&nonempty, nonempty.len(), 0),
        Err(GzipError::Decode(ArtifactError::LimitExceeded { limit: 0 }))
    ));
}

#[test]
fn truncated_headers_payloads_and_footers_never_return_partial_bytes() {
    let compressed = gzip(b"retained artifact payload");
    for end in 0..compressed.len() {
        assert!(matches!(
            decode_gzip(&compressed[..end], compressed.len(), 1024),
            Err(GzipError::Decode(ArtifactError::Io(_)))
        ));
    }
}

#[test]
fn malformed_headers_crc_and_length_fail_with_structured_errors() {
    let compressed = gzip(b"retained artifact payload");
    for index in [0, 2, compressed.len() - 8, compressed.len() - 4] {
        let mut corrupt = compressed.clone();
        corrupt[index] ^= 1;
        assert!(matches!(
            decode_gzip(&corrupt, corrupt.len(), 1024),
            Err(GzipError::Decode(ArtifactError::Io(_)))
        ));
    }
}

#[test]
fn trailing_bytes_and_concatenated_members_are_rejected() {
    let compressed = gzip(b"payload");
    for tail in [vec![0], b"unrelated".to_vec(), gzip(b""), gzip(b"second")] {
        let mut joined = compressed.clone();
        joined.extend_from_slice(&tail);
        assert!(matches!(
            decode_gzip(&joined, joined.len(), 1024),
            Err(GzipError::TrailingData)
        ));
    }
    let mut corrupt = compressed;
    let crc = corrupt.len() - 8;
    corrupt[crc] ^= 1;
    corrupt.push(0);
    assert!(matches!(
        decode_gzip(&corrupt, corrupt.len(), 1024),
        Err(GzipError::Decode(ArtifactError::Io(_)))
    ));
}

#[test]
fn inflation_is_bounded_independently_of_small_compressed_input() {
    let compressed = gzip(&vec![0; 1_000_000]);
    assert!(matches!(
        decode_gzip(&compressed, compressed.len(), 16),
        Err(GzipError::Decode(ArtifactError::LimitExceeded {
            limit: 16
        }))
    ));
}

#[test]
fn optional_header_fields_count_against_the_complete_input_allowance() {
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .filename("fixture.wasm")
        .comment("saved artifact")
        .extra(vec![1, 2, 3])
        .write(Vec::new(), Compression::fast());
    encoder.write_all(b"payload").unwrap();
    let compressed = encoder.finish().unwrap();
    assert_eq!(
        decode_gzip(&compressed, compressed.len(), 7).unwrap(),
        b"payload"
    );
    assert!(matches!(
        decode_gzip(&compressed, compressed.len() - 1, 7),
        Err(GzipError::InputLimit { .. })
    ));
}
