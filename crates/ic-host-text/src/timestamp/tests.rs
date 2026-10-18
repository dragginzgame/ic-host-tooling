use super::{format_utc_timestamp_secs, parse_utc_timestamp_secs};

#[test]
fn timestamp_calendar_vectors_preserve_exact_text() {
    for (seconds, text) in [
        (0, "1970-01-01T00:00:00Z"),
        (86_399, "1970-01-01T23:59:59Z"),
        (951_782_400, "2000-02-29T00:00:00Z"),
        (4_107_542_400, "2100-03-01T00:00:00Z"),
        (253_402_300_799, "9999-12-31T23:59:59Z"),
        (253_402_300_800, "10000-01-01T00:00:00Z"),
    ] {
        assert_eq!(format_utc_timestamp_secs(seconds), text);
        assert_eq!(parse_utc_timestamp_secs(text), Some(seconds));
    }
}

#[test]
fn timestamp_unsigned_formatting_preserves_signed_parsing_limit() {
    let maximum = i64::MAX.unsigned_abs();
    assert_eq!(
        parse_utc_timestamp_secs(&format_utc_timestamp_secs(maximum)),
        Some(maximum)
    );
    for seconds in [maximum + 1, u64::MAX] {
        assert_eq!(
            parse_utc_timestamp_secs(&format_utc_timestamp_secs(seconds)),
            None
        );
    }
    assert_eq!(
        format_utc_timestamp_secs(u64::MAX),
        "584554051223-11-09T07:00:15Z"
    );
}

#[test]
fn timestamp_parser_accepts_canonical_formatter_output() {
    for seconds in [
        0,
        1,
        86_400,
        951_782_400,
        1_780_531_200,
        253_402_300_800,
        i64::MAX.unsigned_abs(),
    ] {
        let timestamp = format_utc_timestamp_secs(seconds);
        assert_eq!(parse_utc_timestamp_secs(&timestamp), Some(seconds));
    }
}

#[test]
fn timestamp_parser_requires_canonical_utc_text() {
    for timestamp in [
        "2026-6-04T00:00:00Z",
        "2026-06-4T00:00:00Z",
        "2026-06-04T0:00:00Z",
        "2026-06-04T00:0:00Z",
        "2026-06-04T00:00:0Z",
        "002026-06-04T00:00:00Z",
        "+2026-06-04T00:00:00Z",
        "2026-+06-04T00:00:00Z",
        "2026-06-04T00:00:000Z",
        "2026-06-04T00:00:00.000Z",
        "2026-06-04T00:00:00+00:00",
        "2026-06-04t00:00:00z",
        " 2026-06-04T00:00:00Z",
        "2026-06-04T00:00:00Z ",
    ] {
        assert_eq!(parse_utc_timestamp_secs(timestamp), None, "{timestamp:?}");
    }
}

#[test]
fn timestamp_parser_rejects_impossible_calendar_dates() {
    assert_eq!(parse_utc_timestamp_secs("2026-02-29T00:00:00Z"), None);
    assert_eq!(parse_utc_timestamp_secs("2026-04-31T00:00:00Z"), None);
    assert!(parse_utc_timestamp_secs("2028-02-29T00:00:00Z").is_some());
    assert_eq!(parse_utc_timestamp_secs("2100-02-29T00:00:00Z"), None);
    assert!(parse_utc_timestamp_secs("2400-02-29T00:00:00Z").is_some());
}

#[test]
fn timestamp_parser_rejects_invalid_fields_and_pre_epoch_dates() {
    for timestamp in [
        "",
        "1970-01-01",
        "1970-01-01T00:00Z",
        "1970-01-01T00:00:00:00Z",
        "1970-01-01-01T00:00:00Z",
        "1970-00-01T00:00:00Z",
        "1970-13-01T00:00:00Z",
        "1970-01-00T00:00:00Z",
        "1970-01-32T00:00:00Z",
        "1970-01-01T24:00:00Z",
        "1970-01-01T00:60:00Z",
        "1970-01-01T00:00:60Z",
        "1969-12-31T23:59:59Z",
        "-0001-01-01T00:00:00Z",
        "1970-01-01T00:00:00ZZ",
        "é-01-01T00:00:00Z",
    ] {
        assert_eq!(parse_utc_timestamp_secs(timestamp), None, "{timestamp:?}");
    }
}

#[test]
fn timestamp_parser_rejects_out_of_range_years_without_panicking() {
    for year in [i64::MIN, i64::MAX, 25_252_734_927_766_555] {
        assert_eq!(
            parse_utc_timestamp_secs(&format!("{year}-01-01T00:00:00Z")),
            None
        );
    }
}
