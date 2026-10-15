use super::{format_utc_timestamp_secs, parse_utc_timestamp_secs};

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
