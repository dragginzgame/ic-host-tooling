use super::*;
use ciborium::Value;
const LIMITS: DecodeLimits = DecodeLimits {
    bytes: 65_536,
    depth: 32,
};

fn encode(value: &impl serde::Serialize) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(value, &mut bytes).unwrap();
    bytes
}
fn array(values: Vec<Value>) -> Value {
    Value::Array(values)
}
fn tag(tag: i64) -> Value {
    Value::Integer(tag.into())
}
fn certificate(signature: Value) -> Value {
    Value::Map(vec![
        (Value::Text("tree".into()), array(vec![tag(0)])),
        (Value::Text("signature".into()), signature),
    ])
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
        Value::Map(vec![]),
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
        ic_certification::empty()
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
    for extra in [
        ("unknown", Value::Null),
        ("signature", Value::Bytes(vec![])),
        ("tree", array(vec![tag(0)])),
        (
            "delegation",
            Value::Map(vec![(
                Value::Text("subnet_id".into()),
                Value::Bytes(vec![]),
            )]),
        ),
        (
            "delegation",
            Value::Map(vec![
                (Value::Text("subnet_id".into()), Value::Text("bad".into())),
                (Value::Text("certificate".into()), Value::Bytes(vec![])),
            ]),
        ),
        (
            "delegation",
            Value::Map(vec![
                (Value::Text("subnet_id".into()), Value::Bytes(vec![])),
                (Value::Text("certificate".into()), Value::Bytes(vec![])),
                (Value::Text("extra".into()), Value::Null),
            ]),
        ),
    ] {
        let mut invalid = fields.clone();
        invalid.push((Value::Text(extra.0.into()), extra.1));
        assert!(matches!(
            decode_certificate(&encode(&Value::Map(invalid)), LIMITS),
            Err(DecodeError::Cbor(_))
        ));
    }
    let mut duplicate_null = fields.clone();
    duplicate_null.extend(std::iter::repeat_n(
        (Value::Text("delegation".into()), Value::Null),
        2,
    ));
    assert!(matches!(
        decode_certificate(&encode(&Value::Map(duplicate_null)), LIMITS),
        Err(DecodeError::Cbor(_))
    ));
    for required in 0..2 {
        let mut missing = fields.clone();
        missing.remove(required);
        assert!(matches!(
            decode_certificate(&encode(&Value::Map(missing)), LIMITS),
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
