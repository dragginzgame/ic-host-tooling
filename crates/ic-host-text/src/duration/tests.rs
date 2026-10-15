use super::compact_duration;

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
