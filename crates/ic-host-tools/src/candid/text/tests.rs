use super::*;

#[test]
fn normalization_preserves_the_canic_line_contract_including_blank_lines() {
    for (input, expected) in [
        ("", ""),
        ("service : {}", "service : {}\n"),
        (
            "//  \r\nservice : {  \r\n  query : () -> ();\t\r\n}",
            "//\nservice : {\n  query : () -> ();\n}\n",
        ),
        ("service : {}\n\n", "service : {}\n\n"),
        ("  indented\u{2003}\n\t\n", "  indented\n\n"),
        ("standalone\rcarriage", "standalone\rcarriage\n"),
    ] {
        assert_eq!(normalize(input.as_bytes(), 1024).unwrap(), expected);
    }
}

#[test]
fn normalization_bounds_input_and_added_newlines_without_lossy_utf8() {
    assert!(matches!(
        normalize(&[0xff], 10),
        Err(NormalizationError::Utf8(_))
    ));
    assert!(matches!(
        normalize(b"abc", 2),
        Err(NormalizationError::InputLimit {
            actual: 3,
            limit: 2
        })
    ));
    assert!(matches!(
        normalize(b"abc", 3),
        Err(NormalizationError::OutputLimit { limit: 3 })
    ));
    assert_eq!(normalize(b"abc", 4).unwrap(), "abc\n");
    assert_eq!(normalize(b"abc\n", 4).unwrap(), "abc\n");
    assert_eq!(normalize(b"", 0).unwrap(), "");
}
