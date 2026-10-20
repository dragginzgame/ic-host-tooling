use std::collections::BTreeMap;

use serde::Deserialize;

use super::*;

const MAX_BYTES: usize = 4096;

#[derive(Debug, Deserialize, PartialEq)]
struct Selected {
    name: String,
}

fn selected(source: &[u8], maximum_input_bytes: usize) -> Result<Selected, DecodeError> {
    from_slice(source, maximum_input_bytes, DecodePolicy::default())
}

#[test]
fn typed_projection_preserves_yaml_strings() {
    assert_eq!(
        selected(b"name: on\nignored: [1, true, null]", MAX_BYTES).unwrap(),
        Selected { name: "on".into() }
    );
    let value: BTreeMap<String, String> =
        from_slice(b"name: example", MAX_BYTES, DecodePolicy::default()).unwrap();
    assert_eq!(value["name"], "example");
    let absent: Option<Selected> = from_slice(b"", 0, DecodePolicy::default()).unwrap();
    assert_eq!(absent, None);
    // Value-tree projection preserves scalar types instead of coercing numbers
    // into strings. Configuration schemas retain their own normalization rules.
    assert!(selected(b"name: 123", MAX_BYTES).is_err());
}

#[test]
fn byte_admission_precedes_utf8_and_syntax_and_accepts_exact_boundary() {
    let source = b"name: example";
    assert!(selected(source, source.len()).is_ok());
    assert!(matches!(
        selected(source, source.len() - 1),
        Err(DecodeError::InputTooLarge { .. })
    ));
    assert!(matches!(
        selected(&[0xff], 0),
        Err(DecodeError::InputTooLarge { .. })
    ));
    assert!(matches!(
        selected(&[0xff], MAX_BYTES),
        Err(DecodeError::Utf8(_))
    ));
}

#[test]
fn entire_document_policy_rejects_ambiguity_even_in_ignored_fields() {
    for source in [
        "name: example\nname: other",
        "name: example\nignored: {duplicate: 1, duplicate: 2}",
        "name: example\nignored: {0xb: first, 11: second}",
        "name: example\nignored: !Unknown 1",
        "name: example\nignored: {<<: {key: value}}",
        "name: example\nignored: {'<<': {key: value}}",
        "name: example\n---\nname: other",
        "name: example\n---\n",
        "name: example\n...\ntrailing: invalid",
        "name: example\n[",
        "",
    ] {
        assert!(
            matches!(
                selected(source.as_bytes(), MAX_BYTES),
                Err(DecodeError::InvalidDocument { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn projections_can_ignore_tags_and_preserve_unexpanded_merge_keys_explicitly() {
    let source = b"name: example\nignored: !Fixed 8002\nother: {<<: {name: wrong}}";
    let policy = DecodePolicy {
        ignore_unknown_tags: true,
        preserve_merge_keys: true,
    };
    assert_eq!(
        from_slice::<Selected>(source, MAX_BYTES, policy)
            .unwrap()
            .name,
        "example"
    );
    assert!(from_slice::<Selected>(b"<<: {name: example}", MAX_BYTES, policy).is_err());
}

#[test]
fn aliases_resolve_without_expanding_merge_keys() {
    let source = b"one: &value [1, 2]\ntwo: *value";
    let values: BTreeMap<String, Vec<u32>> =
        from_slice(source, MAX_BYTES, DecodePolicy::default()).unwrap();
    assert_eq!(values["two"], [1, 2]);
    let nested = format!(
        "name: example\nignored: {}0{}",
        "[".repeat(256),
        "]".repeat(256)
    );
    assert!(selected(nested.as_bytes(), MAX_BYTES).is_err());
}

#[test]
fn no_implicit_include_or_property_resolution() {
    assert_eq!(
        selected(b"name: ${HOST_YAML_UNSET}", MAX_BYTES)
            .unwrap()
            .name,
        "${HOST_YAML_UNSET}"
    );
    assert!(selected(b"name: !include /file-that-must-not-be-read", MAX_BYTES).is_err());
    assert!(from_slice::<bool>(b"on", MAX_BYTES, DecodePolicy::default()).is_err());
}
