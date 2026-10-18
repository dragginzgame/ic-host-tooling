use super::{UnsignedLeb128Error, decode_canonical_unsigned_u64};

#[test]
fn independent_encoding_admits_every_width_boundary() {
    let mut values = vec![0, 1, u64::MAX];
    for shift in (7..64).step_by(7) {
        let boundary = 1_u64 << shift;
        values.extend([boundary - 1, boundary, boundary + 1]);
    }
    for value in values {
        let mut bytes = Vec::new();
        leb128::write::unsigned(&mut bytes, value).expect("reference encoder");
        assert_eq!(decode_canonical_unsigned_u64(&bytes), Ok(value));
    }
}

#[test]
fn malformed_values_retain_typed_failure_categories() {
    use UnsignedLeb128Error::{NonCanonical, Overflow, TrailingData, Truncated};

    for (bytes, expected) in [
        (vec![], Truncated),
        (vec![0x80], Truncated),
        (vec![0x80; 9], Truncated),
        (vec![0x80, 0], NonCanonical),
        (vec![0xff, 0], NonCanonical),
        (vec![0, 0], TrailingData),
        (vec![0x80, 0, 0], TrailingData),
        (vec![0x80; 10], Overflow),
        (vec![0xff; 11], Overflow),
    ] {
        assert_eq!(decode_canonical_unsigned_u64(&bytes), Err(expected));
    }
    for final_byte in [2, 0x7f, 0x81, 0xff] {
        let mut bytes = [0xff; 10];
        bytes[9] = final_byte;
        assert_eq!(decode_canonical_unsigned_u64(&bytes), Err(Overflow));
    }
    assert_eq!(
        decode_canonical_unsigned_u64(&vec![0x80; 65_536]),
        Err(Overflow)
    );
}
