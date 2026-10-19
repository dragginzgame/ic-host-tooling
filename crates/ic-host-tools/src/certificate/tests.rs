use super::*;
use serde_cbor::Value;
const LIMITS: DecodeLimits = DecodeLimits { bytes: 65_536 };

fn encode(value: &impl serde::Serialize) -> Vec<u8> {
    serde_cbor::to_vec(value).unwrap()
}
fn array(values: Vec<Value>) -> Value {
    Value::Array(values)
}
fn tag(tag: i64) -> Value {
    Value::Integer(tag.into())
}
fn certificate(signature: Value) -> Value {
    map(vec![
        (Value::Text("tree".into()), array(vec![tag(0)])),
        (Value::Text("signature".into()), signature),
    ])
}

fn map(fields: Vec<(Value, Value)>) -> Value {
    Value::Map(fields.into_iter().collect())
}

fn encode_map(fields: &[(Value, Value)]) -> Vec<u8> {
    use serde::ser::{SerializeMap as _, Serializer as _};
    let mut serializer = serde_cbor::Serializer::new(Vec::new());
    let mut map = serializer.serialize_map(Some(fields.len())).unwrap();
    for (key, value) in fields {
        map.serialize_entry(key, value).unwrap();
    }
    map.end().unwrap();
    serializer.into_inner()
}

#[test]
fn every_tree_kind_decodes_to_owned_upstream_values() {
    for tree in [
        empty(),
        leaf(vec![1]),
        pruned([2; 32]),
        fork(leaf(vec![3]), pruned([4; 32])),
        labeled(b"label".to_vec(), leaf(vec![5; 5000])),
    ] {
        let bytes = encode(&tree);
        let decoded = decode_hash_tree(&bytes, LIMITS).unwrap();
        drop(bytes);
        assert_eq!(decoded, tree);
        assert_eq!(decoded.digest(), tree.digest());
    }
}

#[test]
fn strict_tags_arities_and_byte_kinds_refuse_malformed_trees() {
    let empty = || array(vec![tag(0)]);
    for value in [
        array(vec![]),
        array(vec![tag(-1)]),
        array(vec![tag(5)]),
        array(vec![Value::Float(0.0)]),
        array(vec![tag(0), tag(0)]),
        array(vec![tag(1), empty()]),
        array(vec![tag(1), empty(), empty(), empty()]),
        array(vec![tag(2), Value::Text("label".into()), empty()]),
        array(vec![tag(3), array(vec![tag(1)])]),
        array(vec![tag(3), Value::Null]),
        array(vec![tag(4), Value::Bytes(vec![1; 31])]),
        array(vec![tag(4), Value::Bytes(vec![1; 33])]),
        Value::Bytes(vec![0]),
        map(vec![]),
    ] {
        assert!(
            matches!(
                decode_hash_tree(&encode(&value), LIMITS),
                Err(DecodeError::Cbor(_))
            ),
            "{value:?}"
        );
    }
    assert_eq!(
        decode_hash_tree(&[0x9f, 0, 0xff], LIMITS).unwrap(),
        ic_platform::certification::empty()
    );
    assert!(matches!(
        decode_hash_tree(&[0x9f, 0, 0, 0xff], LIMITS),
        Err(DecodeError::Cbor(_))
    ));
}

#[test]
fn byte_admission_and_complete_consumption_are_independent() {
    let tree = labeled(b"one".to_vec(), labeled(b"two".to_vec(), leaf(vec![1])));
    let bytes = encode(&tree);
    assert_eq!(
        decode_hash_tree(&bytes, DecodeLimits { bytes: bytes.len() }).unwrap(),
        tree
    );
    assert!(matches!(
        decode_hash_tree(
            &bytes,
            DecodeLimits {
                bytes: bytes.len() - 1,
            }
        ),
        Err(DecodeError::ByteLimit { .. })
    ));
    for suffix in [vec![0], encode(&empty()), vec![0xff]] {
        let mut trailing = bytes.clone();
        trailing.extend(suffix);
        assert!(matches!(
            decode_hash_tree(&trailing, LIMITS),
            Err(DecodeError::TrailingData)
        ));
    }
    assert!(matches!(
        decode_hash_tree(&[], LIMITS),
        Err(DecodeError::Cbor(_))
    ));
    assert!(matches!(
        decode_hash_tree(&[0x81], LIMITS),
        Err(DecodeError::Cbor(_))
    ));
}

#[test]
fn certificates_own_signature_and_delegation_without_authenticating_them() {
    let expected = Certificate {
        tree: fork(leaf(vec![8]), pruned([9; 32])),
        signature: vec![0xff; 5000],
        delegation: Some(Delegation {
            subnet_id: vec![1, 2],
            certificate: vec![3, 4],
        }),
    };
    let bytes = encode(&expected);
    let decoded = decode_certificate(&bytes, LIMITS).unwrap();
    drop(bytes);
    assert_eq!(decoded, expected);
    let no_delegation =
        decode_certificate(&encode(&certificate(Value::Bytes(vec![]))), LIMITS).unwrap();
    assert!(no_delegation.delegation.is_none());
    // Self-describing certificate CBOR retains upstream map decoding semantics.
    let mut tagged = vec![0xd9, 0xd9, 0xf7];
    tagged.extend(encode(&expected));
    assert_eq!(decode_certificate(&tagged, LIMITS).unwrap(), expected);
}

#[test]
fn certificate_and_delegation_fields_are_strict_and_trailing_data_refuses() {
    for signature in [
        Value::Text("signature".into()),
        array(vec![tag(1)]),
        Value::Null,
    ] {
        assert!(matches!(
            decode_certificate(&encode(&certificate(signature)), LIMITS),
            Err(DecodeError::Cbor(_))
        ));
    }
    let Value::Map(fields) = certificate(Value::Bytes(vec![])) else {
        unreachable!()
    };
    let fields: Vec<_> = fields.into_iter().collect();
    for extra in [
        ("unknown", Value::Null),
        ("signature", Value::Bytes(vec![])),
        ("tree", array(vec![tag(0)])),
        (
            "delegation",
            map(vec![(
                Value::Text("subnet_id".into()),
                Value::Bytes(vec![]),
            )]),
        ),
        (
            "delegation",
            map(vec![
                (Value::Text("subnet_id".into()), Value::Text("bad".into())),
                (Value::Text("certificate".into()), Value::Bytes(vec![])),
            ]),
        ),
        (
            "delegation",
            map(vec![
                (Value::Text("subnet_id".into()), Value::Bytes(vec![])),
                (Value::Text("certificate".into()), Value::Bytes(vec![])),
                (Value::Text("extra".into()), Value::Null),
            ]),
        ),
    ] {
        let mut invalid = fields.clone();
        invalid.push((Value::Text(extra.0.into()), extra.1));
        assert!(matches!(
            decode_certificate(&encode_map(&invalid), LIMITS),
            Err(DecodeError::Cbor(_))
        ));
    }
    let mut duplicate_null = fields.clone();
    duplicate_null.extend(std::iter::repeat_n(
        (Value::Text("delegation".into()), Value::Null),
        2,
    ));
    assert!(matches!(
        decode_certificate(&encode_map(&duplicate_null), LIMITS),
        Err(DecodeError::Cbor(_))
    ));
    for required in 0..2 {
        let mut missing = fields.clone();
        missing.remove(required);
        assert!(matches!(
            decode_certificate(&encode_map(&missing), LIMITS),
            Err(DecodeError::Cbor(_))
        ));
    }
    let mut bytes = encode(&certificate(Value::Bytes(vec![])));
    bytes.push(0);
    assert!(matches!(
        decode_certificate(&bytes, LIMITS),
        Err(DecodeError::TrailingData)
    ));
    assert!(matches!(
        decode_certificate(&bytes, DecodeLimits { bytes: 0 }),
        Err(DecodeError::ByteLimit { .. })
    ));
}

#[test]
fn parser_failure_retains_wrapped_source_and_sanitized_display() {
    use std::error::Error as _;

    let secret = "private-certificate-value";
    let error = decode_certificate(&encode(&secret), LIMITS).unwrap_err();
    let wrapped = error.source().unwrap();
    assert!(wrapped.is::<ParseError>());
    assert!(wrapped.source().unwrap().is::<serde_cbor::Error>());
    assert!(!error.to_string().contains(secret));
    assert!(!wrapped.to_string().contains(secret));
    assert!(DecodeError::TrailingData.source().is_none());
}

#[test]
fn codec_recursion_matches_the_ic_agent_decoder() {
    for depth in [1, 2, 32, 126, 127, 128, 129, 255] {
        let mut bytes = Vec::new();
        for _ in 1..depth {
            bytes.extend([0x83, 2, 0x40]);
        }
        bytes.extend([0x81, 0]);
        let upstream = serde_cbor::from_slice::<HashTree>(&bytes);
        let decoded = decode_hash_tree(&bytes, LIMITS);
        assert_eq!(decoded.is_ok(), upstream.is_ok(), "depth {depth}");
        if depth <= 32 {
            assert_eq!(decoded.unwrap(), upstream.unwrap());
        } else if depth >= 128 {
            assert!(matches!(decoded, Err(DecodeError::Cbor(_))));
            assert!(upstream.is_err());
        }
    }
}

#[test]
fn semantic_tags_follow_the_ic_agent_decoder() {
    let expected = labeled(b"name".to_vec(), leaf(b"value".to_vec()));
    let mut tagged = vec![0xd9, 0xd9, 0xf7];
    tagged.extend(encode(&expected));
    assert_eq!(
        decode_hash_tree(&tagged, LIMITS).unwrap(),
        serde_cbor::from_slice::<HashTree>(&tagged).unwrap()
    );
    assert_eq!(decode_hash_tree(&tagged, LIMITS).unwrap(), expected);
    // The codec charges semantic tags to its own recursion budget.
    let mut excessive = [0xd9, 0xd9, 0xf7].repeat(128);
    excessive.extend([0x81, 0]);
    assert!(serde_cbor::from_slice::<HashTree>(&excessive).is_err());
    assert!(matches!(
        decode_hash_tree(&excessive, LIMITS),
        Err(DecodeError::Cbor(_))
    ));
}

#[test]
fn indefinite_containers_and_byte_chunks_remain_supported() {
    let tree = b"\x9f\x03\x5f\x42ab\x41c\xff\xff";
    assert_eq!(
        decode_hash_tree(tree, LIMITS).unwrap(),
        leaf(b"abc".to_vec())
    );
    assert_eq!(
        decode_hash_tree(tree, LIMITS).unwrap(),
        serde_cbor::from_slice::<HashTree>(tree).unwrap()
    );
    let certificate = b"\xbf\x64tree\x9f\0\xff\x69signature\x5f\x41x\x41y\xff\xff";
    let decoded = decode_certificate(certificate, LIMITS).unwrap();
    assert_eq!(decoded.signature, b"xy");
    assert_eq!(
        decoded,
        serde_cbor::from_slice::<Certificate>(certificate).unwrap()
    );
    for bytes in [
        b"\x82\x03\x5f\x61x\xff".as_slice(),
        b"\x82\x03\x5f\x5f\xff\xff",
    ] {
        assert!(matches!(
            decode_hash_tree(bytes, LIMITS),
            Err(DecodeError::Cbor(_))
        ));
    }
}
