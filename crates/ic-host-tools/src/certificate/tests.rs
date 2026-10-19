use super::*;
use serde_cbor::Value;
const LIMITS: DecodeLimits = DecodeLimits {
    bytes: 65_536,
    depth: 32,
};

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
fn byte_and_nesting_bounds_and_complete_consumption_are_independent() {
    let tree = labeled(b"one".to_vec(), labeled(b"two".to_vec(), leaf(vec![1])));
    let bytes = encode(&tree);
    assert_eq!(
        decode_hash_tree(
            &bytes,
            DecodeLimits {
                bytes: bytes.len(),
                depth: 32
            }
        )
        .unwrap(),
        tree
    );
    assert!(matches!(
        decode_hash_tree(
            &bytes,
            DecodeLimits {
                bytes: bytes.len() - 1,
                depth: 0
            }
        ),
        Err(DecodeError::ByteLimit { .. })
    ));
    assert!(matches!(
        decode_hash_tree(&bytes, DecodeLimits { depth: 1, ..LIMITS }),
        Err(DecodeError::Cbor(_))
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
        decode_certificate(&bytes, DecodeLimits { bytes: 0, depth: 0 }),
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
    assert!(wrapped.source().unwrap().is::<minicbor::decode::Error>());
    assert!(!error.to_string().contains(secret));
    assert!(!wrapped.to_string().contains(secret));
    assert!(DecodeError::TrailingData.source().is_none());
}

#[test]
fn full_caller_depth_range_and_exact_container_boundaries_are_supported() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            for depth in [1_u8, 2, 127, 128, 129, 255] {
                let mut bytes = [0x83, 2, 0x40].repeat(usize::from(depth - 1));
                bytes.extend([0x81, 0]);
                let limits = DecodeLimits {
                    bytes: bytes.len(),
                    depth,
                };
                assert!(decode_hash_tree(&bytes, limits).is_ok(), "depth {depth}");
                assert!(matches!(
                    decode_hash_tree(
                        &bytes,
                        DecodeLimits {
                            depth: depth - 1,
                            ..limits
                        }
                    ),
                    Err(DecodeError::Cbor(_))
                ));
            }
        })
        .unwrap()
        .join()
        .unwrap();
    let bytes = encode(&certificate(Value::Bytes(vec![])));
    assert!(decode_certificate(&bytes, DecodeLimits { depth: 2, ..LIMITS }).is_ok());
    assert!(matches!(
        decode_certificate(&bytes, DecodeLimits { depth: 1, ..LIMITS }),
        Err(DecodeError::Cbor(_))
    ));
}

#[test]
fn tagged_maps_and_keys_preserve_the_existing_strict_wire_admission() {
    let certificate = b"\xa2\x64tree\x81\0\x69signature\x40";
    let mut tagged = [0xd9, 0xd9, 0xf7].repeat(150);
    tagged.extend(certificate);
    assert!(decode_certificate(&tagged, DecodeLimits { depth: 2, ..LIMITS }).is_ok());
    let byte_keys = b"\xa2\x44tree\x81\0\x49signature\x40";
    assert!(decode_certificate(byte_keys, LIMITS).is_ok());
    let tagged_key = b"\xa2\xd9\xd9\xf7\x64tree\x81\0\x69signature\x40";
    assert!(decode_certificate(tagged_key, LIMITS).is_ok());
    for invalid in [
        b"\xd9\xd9\xf7\x81\0".as_slice(),
        b"\x81\xd9\xd9\xf7\0",
        b"\x82\x03\xd9\xd9\xf7\x40",
    ] {
        assert!(matches!(
            decode_hash_tree(invalid, LIMITS),
            Err(DecodeError::Cbor(_))
        ));
    }
    // Map keys must remain definite text/byte strings.
    let indefinite_key = b"\xa2\x7f\x64tree\xff\x81\0\x69signature\x40";
    assert!(matches!(
        decode_certificate(indefinite_key, LIMITS),
        Err(DecodeError::Cbor(_))
    ));
}

#[test]
fn indefinite_containers_and_byte_chunks_remain_supported() {
    let tree = b"\x9f\x03\x5f\x42ab\x41c\xff\xff";
    assert_eq!(
        decode_hash_tree(tree, DecodeLimits { depth: 1, ..LIMITS }).unwrap(),
        leaf(b"abc".to_vec())
    );
    let certificate = b"\xbf\x64tree\x9f\0\xff\x69signature\x5f\x41x\x41y\xff\xff";
    let decoded = decode_certificate(certificate, DecodeLimits { depth: 2, ..LIMITS }).unwrap();
    assert_eq!(decoded.signature, b"xy");
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

#[test]
fn delegation_nulls_and_transparent_maps_preserve_their_depth_contract() {
    let base = encode(&certificate(Value::Bytes(vec![])));
    let delegation = Delegation {
        subnet_id: vec![1],
        certificate: vec![0xff, 0],
    };
    let mut tagged_map = vec![0xd9, 0xd9, 0xf7];
    tagged_map.extend(encode(&delegation));
    for value in [vec![0xf6], vec![0xf7], tagged_map] {
        let mut bytes = base.clone();
        bytes[0] = 0xa3;
        bytes.extend(encode(&"delegation"));
        bytes.extend(&value);
        let certificate = decode_certificate(&bytes, DecodeLimits { depth: 2, ..LIMITS }).unwrap();
        if value.len() == 1 {
            assert!(certificate.delegation.is_none());
        } else {
            assert_eq!(certificate.delegation.unwrap(), delegation);
        }
    }
    let mut invalid = base;
    invalid[0] = 0xa3;
    invalid.extend(encode(&"delegation"));
    invalid.extend([0xd9, 0xd9, 0xf7, 0xf6]);
    assert!(matches!(
        decode_certificate(&invalid, LIMITS),
        Err(DecodeError::Cbor(_))
    ));
}
