use super::*;

#[test]
fn complete_match_is_independent_of_write_boundaries() {
    let mut writer = MatchingWriter::new(b"complete bytes");
    writer.write_all(b"com").unwrap();
    assert!(!writer.is_complete_match());
    writer.write_all(b"plete ").unwrap();
    writer.write_all(b"bytes").unwrap();
    writer.flush().unwrap();
    assert!(writer.is_complete_match());
    writer.write_all(b"extra").unwrap();
    assert!(!writer.is_complete_match());
    assert!(MatchingWriter::new(b"").is_complete_match());
}

#[test]
fn mismatch_does_not_stop_the_producer_or_become_a_match() {
    let mut writer = MatchingWriter::new(b"abc");
    assert_eq!(writer.write(b"x").unwrap(), 1);
    writer.write_all(b"bc").unwrap();
    assert!(!writer.is_complete_match());
    writer.write_all(b"more").unwrap();
    assert!(!writer.is_complete_match());
}

#[test]
fn serialized_json_is_compared_as_exact_bytes_without_selecting_ordering() {
    let value = serde_json::json!({"text": "escape\nλ", "rows": [1, true, null]});
    let expected = serde_json::to_vec(&value).unwrap();
    for bytes in [
        expected.as_slice(),
        &expected[..expected.len() - 1],
        b"wrong",
    ] {
        let mut writer = MatchingWriter::new(bytes);
        serde_json::to_writer(&mut writer, &value).unwrap();
        assert_eq!(writer.is_complete_match(), bytes == expected);
    }
}
