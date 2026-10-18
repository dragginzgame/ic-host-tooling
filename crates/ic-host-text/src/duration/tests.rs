use super::{
    DurationParseError, compact_duration, display_duration_seconds, parse_duration_seconds,
};

// Keep human duration labels compact for CLI tables.
#[test]
fn formats_compact_durations() {
    assert_eq!(compact_duration(0), "0s");
    assert_eq!(compact_duration(45), "45s");
    assert_eq!(compact_duration(90), "1m 30s");
    assert_eq!(compact_duration(7_230), "2h");
    assert_eq!(compact_duration(9_000), "2h 30m");
    assert_eq!(compact_duration(97_200), "1d 3h");
    assert_eq!(compact_duration(1_555_200), "2w 4d");
}

#[test]
fn boundaries_and_maximum_do_not_round_or_overflow() {
    for (seconds, expected) in [
        (59, "59s"),
        (60, "1m"),
        (3599, "59m 59s"),
        (3600, "1h"),
        (86399, "23h 59m"),
        (86400, "1d"),
        (604_799, "6d 23h"),
        (604_800, "1w"),
        (u64::MAX, "30500568904943w"),
    ] {
        assert_eq!(compact_duration(seconds), expected);
    }
}

#[test]
fn parses_supported_whole_seconds_and_boundaries() {
    for (input, expected) in [
        ("45", 45),
        ("1s", 1),
        ("30m", 1_800),
        ("2h", 7_200),
        ("1d", 86_400),
        ("+1m", 60),
        ("0001h", 3_600),
        ("18446744073709551615", u64::MAX),
        ("307445734561825860m", 18_446_744_073_709_551_600),
    ] {
        assert_eq!(parse_duration_seconds(input).unwrap(), expected, "{input}");
    }
}

#[test]
fn rejects_invalid_zero_and_overflow_without_normalizing_evidence() {
    for input in [
        "",
        "0",
        "0s",
        "-1",
        "1.5h",
        "1H",
        "1w",
        " 1h",
        "1h ",
        "1 h",
        "１h",
        "18446744073709551616",
        "307445734561825861m",
        "5124095576030432h",
        "213503982334602d",
    ] {
        let error = parse_duration_seconds(input).unwrap_err();
        assert!(matches!(&error, DurationParseError::Invalid { value } if value == input));
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn renders_exact_units_and_half_up_hundredths_without_float_arithmetic() {
    for (seconds, expected) in [
        (0, "0s"),
        (45, "45s"),
        (60, "1m"),
        (61, "1.02m"),
        (90, "1.50m"),
        (3_599, "59.98m"),
        (3_600, "1h"),
        (3_618, "1.01h"),
        (5_400, "1.50h"),
        (86_400, "1d"),
        (2_629_800, "30.44d"),
        (u64::MAX, "213503982334601.29d"),
    ] {
        assert_eq!(display_duration_seconds(seconds), expected, "{seconds}");
    }
}

#[test]
fn errors_escape_original_input_in_the_diagnostic() {
    let error = parse_duration_seconds("1h\n").unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid duration \"1h\\n\"; use positive seconds or a value ending in s, m, h, or d"
    );
}
