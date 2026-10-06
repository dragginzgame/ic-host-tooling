use super::{JsonErrorKind, ResponseError, ResponseFormat, ResponseLimits, decode};

fn bounded(input: &[u8], format: ResponseFormat) -> Result<Vec<u8>, ResponseError> {
    decode(
        input,
        format,
        ResponseLimits {
            input_bytes: input.len(),
            decoded_bytes: 32,
        },
    )
}

#[test]
fn canic_envelope_returns_opaque_bytes_and_ignores_metadata() {
    // The bytes are candid::Encode!(&42_u64), from Canic's response tests.
    let input = br#"{"response_text":null,"response_candid":"scripted","extra":{"items":[null,true,3]},"response_bytes":"4449444c0001782a00000000000000"}"#;
    assert_eq!(
        bounded(input, ResponseFormat::Json).unwrap(),
        b"DIDL\0\x01\x78\x2a\0\0\0\0\0\0\0"
    );
    // Byte decoding must not impose Candid validity or decode a Result envelope.
    assert_eq!(
        bounded(br#"{"response_bytes":"00"}"#, ResponseFormat::Json).unwrap(),
        [0]
    );
    assert_eq!(
        bounded(br#"{"response_bytes":""}"#, ResponseFormat::Json).unwrap(),
        [] as [u8; 0]
    );
}

#[test]
fn icydb_plain_and_labeled_hex_match_existing_call_fixtures() {
    assert_eq!(
        bounded(b"4449444c00017f", ResponseFormat::Hex).unwrap(),
        b"DIDL\0\x01\x7f"
    );
    assert_eq!(
        bounded(b"response (hex): 44 49 44 4c", ResponseFormat::LabeledHex).unwrap(),
        b"DIDL"
    );
    assert_eq!(
        bounded(
            b" \tresponse (hex): 4 4\r\n4 9\t4 4\x0c4 C ",
            ResponseFormat::LabeledHex
        )
        .unwrap(),
        b"DIDL"
    );
}

#[test]
fn json_escapes_are_decoded_by_the_json_owner() {
    assert_eq!(
        bounded(
            br#"{"response_\u0062ytes":"\u0034\u0034\u0034\u0063"}"#,
            ResponseFormat::Json
        )
        .unwrap(),
        b"DL"
    );
}

#[test]
fn top_level_string_is_required_and_duplicate_authority_is_rejected() {
    for input in [
        b"{}".as_slice(),
        br#"{"response_bytes":null}"#,
        br#"{"other":{"response_bytes":"00"}}"#,
    ] {
        assert!(matches!(
            bounded(input, ResponseFormat::Json),
            Err(ResponseError::MissingResponseBytes)
        ));
    }
    for input in [
        b"[]".as_slice(),
        b"null",
        br#"{"response_bytes":12}"#,
        br#"{"response_bytes":[]}"#,
        br#"{"response_bytes":"00","response_bytes":"01"}"#,
        br#"{"response_bytes":null,"response_bytes":"01"}"#,
        br#"{"response_bytes":"00","response_\u0062ytes":"01"}"#,
    ] {
        assert!(matches!(
            bounded(input, ResponseFormat::Json),
            Err(ResponseError::Json {
                kind: JsonErrorKind::Data,
                ..
            })
        ));
    }
}

#[test]
fn truncated_malformed_and_trailing_json_are_rejected() {
    for input in [
        b"{".as_slice(),
        br#"{"response_bytes":"00""#,
        br#"{"response_bytes":"\u00"#,
    ] {
        assert!(matches!(
            bounded(input, ResponseFormat::Json),
            Err(ResponseError::Json {
                kind: JsonErrorKind::EndOfInput,
                ..
            })
        ));
    }
    for input in [
        br#"{"response_bytes":"00",}"#.as_slice(),
        br#"{"response_bytes":"00"} {"response_bytes":"01"}"#,
        br#"{"response_bytes":"00","extra":[1,]}"#,
        br#"{"response_bytes":"\q"}"#,
    ] {
        assert!(matches!(
            bounded(input, ResponseFormat::Json),
            Err(ResponseError::Json {
                kind: JsonErrorKind::Syntax,
                ..
            })
        ));
    }
}

#[test]
fn invalid_utf8_is_rejected_even_in_ignored_json_metadata() {
    let input = b"{\"response_bytes\":\"00\",\"other\":\"\xff\"}";
    assert!(matches!(
        bounded(input, ResponseFormat::Json),
        Err(ResponseError::InvalidUtf8 { offset: 32 })
    ));
}

#[test]
fn invalid_hex_and_odd_digits_return_typed_failures() {
    for (input, format, offset) in [
        (b"00g0".as_slice(), ResponseFormat::Hex, 2),
        (b"00\xff", ResponseFormat::Hex, 2),
        (b"00\0", ResponseFormat::Hex, 2),
        (b"00\x0b", ResponseFormat::Hex, 2),
        ("00\u{a0}01".as_bytes(), ResponseFormat::Hex, 2),
        (br#"{"response_bytes":"00 01"}"#, ResponseFormat::Json, 2),
        (br#"{"response_bytes":" 00"}"#, ResponseFormat::Json, 0),
        (br#"{"response_bytes":"\u2603"}"#, ResponseFormat::Json, 0),
    ] {
        assert!(
            matches!(bounded(input, format), Err(ResponseError::InvalidHex { offset: actual }) if actual == offset)
        );
    }
    for input in [b"0".as_slice(), b"12 3"] {
        assert!(matches!(
            bounded(input, ResponseFormat::Hex),
            Err(ResponseError::OddHexLength)
        ));
    }
    assert!(matches!(
        bounded(br#"{"response_bytes":"123"}"#, ResponseFormat::Json),
        Err(ResponseError::OddHexLength)
    ));
}

#[test]
fn formats_are_explicit_and_labels_cannot_hide_other_output() {
    for input in [
        b"response (hex): 00".as_slice(),
        br#"{"response_bytes":"00"}"#,
    ] {
        assert!(matches!(
            bounded(input, ResponseFormat::Hex),
            Err(ResponseError::InvalidHex { .. })
        ));
    }
    for input in [
        b"00".as_slice(),
        b"warning\nresponse (hex): 00",
        b"Response (hex): 00",
    ] {
        assert!(matches!(
            bounded(input, ResponseFormat::LabeledHex),
            Err(ResponseError::MissingHexLabel)
        ));
    }
    for input in [
        b"response (hex): 00 response (hex): 01".as_slice(),
        b"response (hex): 00\nwarning",
    ] {
        assert!(matches!(
            bounded(input, ResponseFormat::LabeledHex),
            Err(ResponseError::InvalidHex { .. })
        ));
    }
    for input in [b"".as_slice(), b" \t\r\n"] {
        assert!(matches!(
            bounded(input, ResponseFormat::Hex),
            Err(ResponseError::EmptyHex)
        ));
    }
    assert!(matches!(
        bounded(b"response (hex): \n", ResponseFormat::LabeledHex),
        Err(ResponseError::EmptyHex)
    ));
}

#[test]
fn limits_cover_complete_metadata_and_exact_decoded_boundaries() {
    for (input, format) in [
        (
            br#"{"response_bytes":"00ff","metadata":[1,2,3]}"#.as_slice(),
            ResponseFormat::Json,
        ),
        (b"00 ff \n", ResponseFormat::Hex),
        (b"response (hex): 00 ff", ResponseFormat::LabeledHex),
    ] {
        assert_eq!(
            decode(
                input,
                format,
                ResponseLimits {
                    input_bytes: input.len(),
                    decoded_bytes: 2
                }
            )
            .unwrap(),
            [0, 255]
        );
        assert!(matches!(
            decode(
                input,
                format,
                ResponseLimits {
                    input_bytes: input.len() - 1,
                    decoded_bytes: 2
                }
            ),
            Err(ResponseError::InputLimit { .. })
        ));
        assert!(matches!(
            decode(
                input,
                format,
                ResponseLimits {
                    input_bytes: input.len(),
                    decoded_bytes: 1
                }
            ),
            Err(ResponseError::DecodedLimit { limit: 1 })
        ));
    }
    assert!(matches!(
        decode(
            b"not JSON",
            ResponseFormat::Json,
            ResponseLimits {
                input_bytes: 0,
                decoded_bytes: 0
            }
        ),
        Err(ResponseError::InputLimit { limit: 0 })
    ));
    assert_eq!(
        decode(
            br#"{"response_bytes":""}"#,
            ResponseFormat::Json,
            ResponseLimits {
                input_bytes: 21,
                decoded_bytes: 0
            }
        )
        .unwrap(),
        [] as [u8; 0]
    );
    assert!(matches!(
        decode(
            b"00",
            ResponseFormat::Hex,
            ResponseLimits {
                input_bytes: 2,
                decoded_bytes: 0
            }
        ),
        Err(ResponseError::DecodedLimit { limit: 0 })
    ));
}

#[test]
fn diagnostics_do_not_format_response_values() {
    for (input, format) in [
        (
            br#"{"response_bytes":"credential-secret"}"#.as_slice(),
            ResponseFormat::Json,
        ),
        (
            br#"{"response_bytes":{"credential-secret":"private-value"}}"#,
            ResponseFormat::Json,
        ),
        (b"credential-secret", ResponseFormat::Hex),
    ] {
        let error = bounded(input, format).unwrap_err();
        for text in [format!("{error}"), format!("{error:?}")] {
            assert!(!text.contains("credential-secret"));
            assert!(!text.contains("private-value"));
        }
    }
}
