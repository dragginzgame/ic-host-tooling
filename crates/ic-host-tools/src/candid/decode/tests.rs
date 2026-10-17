use super::*;

fn limits(bytes: usize) -> DecodeLimits {
    DecodeLimits {
        reply_bytes: bytes,
        decoding_quota: 4096,
        skipping_quota: 4096,
        type_entries: 64,
        header_bytes: 1024,
    }
}

#[test]
fn exact_budget_and_repeated_replies_are_charged() {
    let bytes = candid_parser::encode_one(42_u64).unwrap();
    let mut remaining = bytes.len() * 2;
    for _ in 0..2 {
        assert_eq!(
            decode_one::<u64>(&bytes, limits(bytes.len()), &mut remaining).unwrap(),
            42
        );
    }
    assert_eq!(remaining, 0);
    assert!(matches!(
        decode_one::<u64>(&bytes, limits(bytes.len()), &mut remaining),
        Err(DecodeError::AggregateBytes)
    ));
}

#[test]
fn byte_admission_precedes_decode_and_never_underflows() {
    let bytes = b"not candid";
    let mut remaining = bytes.len();
    assert!(matches!(
        decode_one::<u64>(bytes, limits(bytes.len() - 1), &mut remaining),
        Err(DecodeError::ReplyBytes)
    ));
    assert_eq!(remaining, bytes.len());
    assert!(matches!(
        decode_one::<u64>(bytes, limits(bytes.len()), &mut remaining),
        Err(DecodeError::Candid(_))
    ));
    assert_eq!(remaining, 0);
}

#[test]
fn caller_decoder_quotas_are_enforced() {
    let bytes = candid_parser::encode_one(vec![42_u64; 128]).unwrap();
    for constrained in [
        DecodeLimits {
            decoding_quota: 0,
            ..limits(bytes.len())
        },
        DecodeLimits {
            type_entries: 0,
            ..limits(bytes.len())
        },
        DecodeLimits {
            header_bytes: 0,
            ..limits(bytes.len())
        },
    ] {
        let mut remaining = bytes.len();
        assert!(matches!(
            decode_one::<Vec<u64>>(&bytes, constrained, &mut remaining),
            Err(DecodeError::Candid(_))
        ));
        assert_eq!(remaining, 0);
    }
}

#[test]
fn skipped_fields_consume_the_selected_work_quota() {
    let bytes = candid_parser::encode_one(vec![42_u64; 128]).unwrap();
    let mut remaining = bytes.len();
    let constrained = DecodeLimits {
        skipping_quota: 0,
        ..limits(bytes.len())
    };
    assert!(matches!(
        decode_one::<candid_parser::Reserved>(&bytes, constrained, &mut remaining),
        Err(DecodeError::Candid(_))
    ));
    assert_eq!(remaining, 0);
}
