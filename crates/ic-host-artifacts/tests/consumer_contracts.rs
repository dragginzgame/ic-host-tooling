//! Public consumer projections: policy remains with the artifact owner.

use ic_host_artifacts::{
    artifact::{ArtifactError, BoundedWriter, Sha256Digest, copy_reader, hash_reader},
    wasm::{ExportKind, InspectionError, InspectionLimits, ParseError, WasmFacts, inspect},
};
use std::{collections::BTreeMap, io};

#[cfg(feature = "wasm-validation")]
#[test]
fn wasm_validation_exposes_host_errors_without_a_parser_dependency() {
    use ic_host_artifacts::wasm::{ValidationError, validate};
    use std::error::Error as _;

    let invalid_export = b"\0asm\x01\0\0\0\x07\x05\x01\x01x\0\x63";
    let Err(ValidationError::Invalid(diagnostic)) = validate(invalid_export, invalid_export.len())
    else {
        panic!("expected nonexistent function refusal");
    };
    let diagnostic: ParseError = diagnostic;
    assert_eq!(diagnostic.offset(), 11);
    assert_ne!(diagnostic.message(), "");
    assert!(diagnostic.source().is_none());
    validate(b"\0asm\x01\0\0\0", 8).unwrap();
}

#[test]
fn wasm_parse_errors_expose_owned_offsets_and_diagnostics() {
    use std::error::Error as _;

    let limits = InspectionLimits {
        module_bytes: 64,
        sections: 1,
        exports: 1,
        custom_sections: 0,
    };
    let invalid_export = b"\0asm\x01\0\0\0\x07\x05\x01\x01x\x09\0";
    for (bytes, expected_offset) in [(b"not wasm".as_slice(), 0), (invalid_export.as_slice(), 13)] {
        let error = inspect(bytes, limits).unwrap_err();
        let InspectionError::Parse(parse) = &error else {
            panic!("expected a structural parse failure");
        };
        let parse: &ParseError = parse;
        let offset: u64 = parse.offset();
        assert_eq!(offset, expected_offset);
        assert_ne!(parse.message(), "");
        assert_ne!(parse.to_string(), "");
        assert!(error.source().unwrap().is::<ParseError>());
    }
}

#[test]
fn pretty_json_count_hash_and_staging_preserve_the_consumers_exact_encoding() {
    use sha2::{Digest as _, Sha256};

    let value = serde_json::json!({"rows": ["a", "b"], "schema_version": 1});
    let expected = br#"{
  "rows": [
    "a",
    "b"
  ],
  "schema_version": 1
}"#;
    let limit = expected.len() as u64;
    let mut count = BoundedWriter::new(io::sink(), limit);
    serde_json::to_writer_pretty(&mut count, &value).unwrap();
    assert_eq!(count.bytes_written(), limit);
    assert!(!count.limit_exceeded());

    let mut hash = BoundedWriter::new(Sha256::new(), limit);
    serde_json::to_writer_pretty(&mut hash, &value).unwrap();
    assert_eq!(hash.bytes_written(), limit);
    assert_eq!(
        Sha256Digest::from_bytes(hash.into_inner().finalize().into()),
        Sha256Digest::compute(expected)
    );

    let mut staged = BoundedWriter::new(Vec::new(), limit);
    serde_json::to_writer_pretty(&mut staged, &value).unwrap();
    assert_eq!(staged.bytes_written(), count.bytes_written());
    assert_eq!(staged.into_inner(), expected);

    let mut count = BoundedWriter::new(io::sink(), limit - 1);
    let error = serde_json::to_writer_pretty(&mut count, &value).unwrap_err();
    assert_eq!(error.io_error_kind(), Some(io::ErrorKind::Other));
    assert!(count.limit_exceeded());
}

#[test]
fn a_successful_serialization_preflight_does_not_unbound_the_staging_pass() {
    use serde::{Serialize, Serializer};
    use std::cell::Cell;

    struct ChangingSerialization(Cell<bool>);
    impl Serialize for ChangingSerialization {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(if self.0.replace(true) {
                "a larger second serialization that exceeds the write limit"
            } else {
                "small"
            })
        }
    }

    let value = ChangingSerialization(Cell::new(false));
    let limit = 16;
    let mut preflight = BoundedWriter::new(io::sink(), limit);
    serde_json::to_writer_pretty(&mut preflight, &value).unwrap();
    assert!(preflight.bytes_written() <= limit);
    assert!(!preflight.limit_exceeded());

    let old = b"existing published JSON".to_vec();
    let mut published = old.clone();
    let mut staging = BoundedWriter::new(Vec::new(), limit);
    let result = serde_json::to_writer_pretty(&mut staging, &value);
    assert_eq!(
        result.as_ref().unwrap_err().io_error_kind(),
        Some(io::ErrorKind::Other)
    );
    assert!(staging.limit_exceeded());
    let accepted = staging.bytes_written();
    let retained = staging.into_inner();
    // The consumer's existing publisher is reached only for a complete encoding.
    if result.is_ok() {
        published.clone_from(&retained);
    }
    assert_eq!(published, old);
    assert_ne!(retained, [] as [u8; 0]);
    assert_eq!(retained.len() as u64, accepted);
    assert!(accepted <= limit);
}

#[test]
fn staged_stream_identity_is_checked_before_consumer_publication() {
    let old = b"existing published artifact".to_vec();
    let candidate = b"candidate artifact";
    let expected = Sha256Digest::compute(candidate);
    let mut published = old.clone();
    let mut staged = Vec::new();
    let observed = copy_reader(b"wrong".as_slice(), &mut staged, 128).unwrap();
    assert_ne!(observed.sha256, expected);
    assert_eq!(published, old);
    assert_eq!(staged, b"wrong");

    let mut staged = BoundedWriter::new(Vec::new(), 128);
    let observed = copy_reader(candidate.as_slice(), &mut staged, 128).unwrap();
    assert_eq!(observed.sha256, expected);
    assert_eq!(observed.bytes, staged.bytes_written());
    // The consumer's publisher owns this step; shared copying never performs it.
    published = staged.into_inner();
    assert_eq!(published, candidate);

    let mut count = BoundedWriter::new(io::sink(), 128);
    let observed = copy_reader(candidate.as_slice(), &mut count, 128).unwrap();
    assert_eq!(observed.bytes, count.bytes_written());
    assert_eq!(observed.sha256, expected);
}

const LIMITS: InspectionLimits = InspectionLimits {
    module_bytes: 4096,
    sections: 20,
    exports: 20,
    custom_sections: 10,
};

fn artifact(export_index: u8, candid: &[u8]) -> Vec<u8> {
    let mut bytes = b"\0asm\x01\0\0\0".to_vec();
    bytes.extend([1, 4, 1, 0x60, 0, 0]); // Type () -> ().
    bytes.extend([3, 3, 2, 0, 0]); // Two defined functions of that type.
    let name = b"canister_query status";
    let mut export = vec![1, u8::try_from(name.len()).unwrap()];
    export.extend(name);
    export.extend([0, export_index]);
    bytes.extend([7, u8::try_from(export.len()).unwrap()]);
    bytes.extend(export);
    bytes.extend([10, 7, 2, 2, 0, 0x0b, 2, 0, 0x0b]);
    let metadata_name = b"icp:public candid:service";
    let mut metadata = vec![u8::try_from(metadata_name.len()).unwrap()];
    metadata.extend(metadata_name);
    metadata.extend(candid);
    bytes.extend([0, u8::try_from(metadata.len()).unwrap()]);
    bytes.extend(metadata);
    bytes
}

fn export_kinds<'a>(facts: &WasmFacts<'a>) -> BTreeMap<&'a str, ExportKind> {
    facts
        .exports
        .iter()
        .map(|(name, export)| (*name, export.kind))
        .collect()
}

fn public_candid<'a>(facts: &WasmFacts<'a>) -> Vec<&'a [u8]> {
    facts
        .custom_sections
        .iter()
        .filter(|section| section.name == "icp:public candid:service")
        .map(|section| section.data)
        .collect()
}

#[test]
fn transform_contract_preserves_export_kinds_and_selected_metadata_after_reindexing() {
    let compiler = artifact(0, b"service : { status : () -> () query; }\n");
    let final_bytes = artifact(1, b"service : { status : () -> () query; }\n");
    let changed = artifact(1, b"service : {}\n");
    let before = inspect(&compiler, LIMITS).unwrap();
    let after = inspect(&final_bytes, LIMITS).unwrap();
    assert_eq!(export_kinds(&before), export_kinds(&after));
    assert_eq!(public_candid(&before), public_candid(&after));
    assert_ne!(before.exports, after.exports);
    assert_ne!(
        public_candid(&before),
        public_candid(&inspect(&changed, LIMITS).unwrap())
    );
}

#[test]
fn report_facts_support_consumer_budgets_after_exact_digest_verification() {
    let bytes = artifact(0, b"service : {}\n");
    let expected = Sha256Digest::compute(&bytes);
    let identity = hash_reader(bytes.as_slice(), LIMITS.module_bytes as u64)
        .unwrap()
        .verify(expected)
        .unwrap();
    let report = inspect(&bytes, LIMITS).unwrap();
    assert_eq!(identity.bytes, report.raw_bytes as u64);
    assert_eq!(
        (report.defined_functions, report.code_section_bytes),
        (2, 7)
    );
    // Different owners may accept or reject identical facts without a library default.
    let admits = |budget| report.code_section_bytes <= budget;
    assert!(admits(7));
    assert!(!admits(6));
    let application_functions: Vec<_> = report
        .exports
        .iter()
        .filter(|(name, export)| {
            export.kind == ExportKind::Function && name.starts_with("canister_query ")
        })
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(application_functions, ["canister_query status"]);
    assert!(matches!(
        hash_reader(bytes.as_slice(), LIMITS.module_bytes as u64)
            .unwrap()
            .verify(Sha256Digest::compute(b"wrong")),
        Err(ArtifactError::DigestMismatch { .. })
    ));
}
