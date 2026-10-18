use super::{InvalidDecimal, compact_decimal_text, fixed_decimal_text};

#[test]
fn exact_scaling_rounding_and_carries_preserve_arbitrary_precision() {
    for (input, exponent, fixed, compact) in [
        ("0", 8, "0.00", "0"),
        ("000", 0, "0.00", "0"),
        ("0.005", 0, "0.01", "0.01"),
        ("0.004999", 0, "0.00", "0"),
        ("12.3", 0, "12.30", "12.3"),
        ("1.", 0, "1.00", "1"),
        ("123", 0, "123.00", "123"),
        ("123", 1, "12.30", "12.3"),
        ("123", 2, "1.23", "1.23"),
        ("999", 3, "1.00", "1"),
        ("500000", 8, "0.01", "0.01"),
        ("999.995", 0, "1000.00", "1000"),
        ("999999.995", 3, "1000.00", "1000"),
        ("000001000.0", 3, "1.00", "1"),
        ("40067084771.847176", 9, "40.07", "40.07"),
        ("123456789", usize::MAX, "0.00", "0"),
        (
            "340282366920938463463374607431768211456.789",
            18,
            "340282366920938463463.37",
            "340282366920938463463.37",
        ),
    ] {
        assert_eq!(fixed_decimal_text(input, exponent), Ok(fixed.to_string()));
        assert_eq!(
            compact_decimal_text(input, exponent),
            Ok(compact.to_string())
        );
    }
    let long = format!("1.234{}", "9".repeat(4096));
    assert_eq!(compact_decimal_text(&long, 0), Ok("1.23".to_string()));
}

#[test]
fn malformed_text_refuses_without_returning_a_partial_number() {
    for input in ["", ".", ".1", "-1", "+1", "1_000", " 1", "1 ", "1.2.3", "١"] {
        assert_eq!(fixed_decimal_text(input, 0), Err(InvalidDecimal));
        assert_eq!(compact_decimal_text(input, 0), Err(InvalidDecimal));
    }
    let trailing_invalid = format!("1.234{}x", "9".repeat(4096));
    assert_eq!(
        compact_decimal_text(&trailing_invalid, 0),
        Err(InvalidDecimal)
    );
}

#[test]
fn text_rounding_agrees_with_integer_arithmetic() {
    // Independent integer oracle: input is an exact number of thousandths.
    for base in [0, 999_000, 123_456_789_012_345_678_000_000_u128] {
        for offset in (0..10_000).step_by(37) {
            let thousandths = base + offset;
            let input = format!("{}.{:03}", thousandths / 1_000, thousandths % 1_000);
            for exponent in 0..=8 {
                let divisor = 10_u128.pow(exponent + 1);
                let cents = (thousandths + divisor / 2) / divisor;
                let fixed = format!("{}.{:02}", cents / 100, cents % 100);
                let compact = fixed.trim_end_matches('0').trim_end_matches('.');
                assert_eq!(
                    fixed_decimal_text(&input, exponent as usize),
                    Ok(fixed.clone())
                );
                assert_eq!(
                    compact_decimal_text(&input, exponent as usize).as_deref(),
                    Ok(compact)
                );
            }
        }
    }
}
