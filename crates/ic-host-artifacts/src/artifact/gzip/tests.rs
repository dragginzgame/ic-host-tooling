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
        encode_gzip(input, &mut output, 9, reference.len() as u64).unwrap();
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
    encode_gzip(b"", &mut complete, 1, 100).unwrap();
    let mut partial = Vec::new();
    let error = encode_gzip(b"", &mut partial, 1, complete.len() as u64 - 1).unwrap_err();
    assert!(matches!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(WriterError::LimitExceeded { .. })
    ));
    assert!(partial.len() < complete.len());
    let error = encode_gzip(b"payload", FailedSink, 0, 100).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn compression_levels_preserve_backend_bytes_and_reject_invalid_input_before_writing() {
    for level in 0..=9 {
        let input = b"same bytes same bytes same bytes";
        let mut expected = GzBuilder::new()
            .mtime(0)
            .write(Vec::new(), Compression::new(level));
        expected.write_all(input).unwrap();
        let mut output = Vec::new();
        encode_gzip(input, &mut output, level, 4096).unwrap();
        assert_eq!(output, expected.finish().unwrap());
    }
    let mut output = b"untouched".to_vec();
    for level in [10, u32::MAX] {
        assert_eq!(
            encode_gzip(b"input", &mut output, level, 4096)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(output, b"untouched");
    }
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

#[test]
fn streamed_hashes_identify_payloads_and_enforce_both_representation_budgets() {
    for raw in [vec![], b"\0asm\x01\0\0\0".to_vec(), vec![42; 100_003]] {
        let compressed = gzip(&raw);
        let expected = super::super::ArtifactIdentity {
            bytes: raw.len() as u64,
            sha256: super::super::Sha256Digest::compute(&raw),
        };
        assert_eq!(
            hash_gzip(&compressed, compressed.len(), expected.bytes).unwrap(),
            expected
        );
        for input in [&raw, &compressed] {
            assert_eq!(
                hash_gzip_or_raw(input, input.len(), expected.bytes).unwrap(),
                expected
            );
            if !input.is_empty() {
                assert!(matches!(
                    hash_gzip_or_raw(input, input.len() - 1, expected.bytes),
                    Err(GzipError::InputLimit { .. })
                ));
            }
            if expected.bytes != 0 {
                assert!(matches!(
                    hash_gzip_or_raw(input, input.len(), expected.bytes - 1),
                    Err(GzipError::Decode(ArtifactError::LimitExceeded { .. }))
                ));
            }
        }
    }
    // Only a complete magic prefix selects gzip; no Wasm interpretation is made.
    for raw in [b"\x1f".as_slice(), b"\x1f\x00", b"ordinary data"] {
        assert_eq!(
            hash_gzip_or_raw(raw, raw.len(), raw.len() as u64)
                .unwrap()
                .sha256,
            super::super::Sha256Digest::compute(raw)
        );
        assert!(hash_gzip(raw, raw.len(), raw.len() as u64).is_err());
    }
    assert!(hash_gzip_or_raw(b"\x1f\x8b", 2, 2).is_err());
}

#[test]
fn gzip_comparison_is_exact_bounded_and_checks_completion_after_mismatch() {
    let raw = vec![42; 100_003];
    let compressed = gzip(&raw);
    assert!(gzip_matches(&compressed, &raw, compressed.len()).unwrap());
    let mut different = raw.clone();
    different[0] ^= 1;
    assert!(!gzip_matches(&compressed, &different, compressed.len()).unwrap());
    different = raw.clone();
    different.push(0);
    assert!(!gzip_matches(&compressed, &different, compressed.len()).unwrap());
    assert!(matches!(
        gzip_matches(&compressed, &raw[..raw.len() - 1], compressed.len()),
        Err(GzipError::Decode(ArtifactError::LimitExceeded { .. }))
    ));
    assert!(matches!(
        gzip_matches(&compressed, &raw, compressed.len() - 1),
        Err(GzipError::InputLimit { .. })
    ));
    let empty = gzip(b"");
    assert!(gzip_matches(&empty, b"", empty.len()).unwrap());
    assert!(!gzip_matches(&empty, b"a", empty.len()).unwrap());
}

#[test]
fn streamed_operations_refuse_truncation_integrity_failure_and_trailing_data() {
    let raw = b"a complete artifact";
    let compressed = gzip(raw);
    let check = |input: &[u8]| {
        // A content mismatch must not conceal malformed framing or integrity.
        let different = vec![0; raw.len()];
        assert!(gzip_matches(input, &different, input.len()).is_err());
        assert!(hash_gzip(input, input.len(), raw.len() as u64).is_err());
        if input.starts_with(&[0x1f, 0x8b]) {
            assert!(hash_gzip_or_raw(input, input.len(), raw.len() as u64).is_err());
        }
    };
    for end in 0..compressed.len() {
        check(&compressed[..end]);
    }
    for index in [2, compressed.len() - 8, compressed.len() - 4] {
        let mut corrupt = compressed.clone();
        corrupt[index] ^= 1;
        check(&corrupt);
    }
    for tail in [vec![0], gzip(b""), gzip(b"more")] {
        let mut joined = compressed.clone();
        joined.extend(tail);
        assert!(matches!(
            hash_gzip(&joined, joined.len(), raw.len() as u64),
            Err(GzipError::TrailingData)
        ));
        assert!(matches!(
            gzip_matches(&joined, raw, joined.len()),
            Err(GzipError::TrailingData)
        ));
        check(&joined);
    }
}
