use super::*;

const HEADER: &[u8] = b"\0asm\x01\0\0\0";
const LIMITS: InspectionLimits = InspectionLimits {
    module_bytes: 4096,
    sections: 20,
    exports: 20,
    custom_sections: 10,
};

fn leb(mut number: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let byte = u8::try_from(number & 0x7f).unwrap();
        number >>= 7;
        bytes.push(byte | if number == 0 { 0 } else { 0x80 });
        if number == 0 {
            return bytes;
        }
    }
}

fn section(module: &mut Vec<u8>, id: u8, payload: &[u8]) {
    module.push(id);
    module.extend(leb(u32::try_from(payload.len()).unwrap()));
    module.extend_from_slice(payload);
}

fn custom(module: &mut Vec<u8>, name: &str, data: &[u8]) {
    let mut payload = leb(u32::try_from(name.len()).unwrap());
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(data);
    section(module, 0, &payload);
}

fn fixture() -> Vec<u8> {
    let mut module = HEADER.to_vec();
    section(&mut module, 1, &[1, 0x60, 0, 0]); // One () -> () type.
    section(&mut module, 3, &[1, 0]); // One defined function.
    section(&mut module, 5, &[1, 0, 1]); // One memory.
    let mut exports = vec![2];
    for (name, kind) in [("canister_query status", 0), ("memory", 2)] {
        exports.extend(leb(u32::try_from(name.len()).unwrap()));
        exports.extend_from_slice(name.as_bytes());
        exports.extend([kind, 0]);
    }
    section(&mut module, 7, &exports);
    section(&mut module, 10, &[1, 2, 0, 0x0b]); // Vector count, body size, locals, end.
    section(&mut module, 11, &[1, 1, 3, b'a', b'b', b'c']); // One passive data segment.
    custom(&mut module, "icp:public candid:service", b"service : {}\n");
    custom(
        &mut module,
        "icp:public candid:service",
        b"second declaration",
    );
    module
}

#[test]
fn encodes_complete_custom_sections_with_exact_names_data_and_append_order() {
    let mut module = HEADER.to_vec();
    for (section, golden) in [
        (
            CustomSection {
                name: "",
                data: &[],
            },
            &[0, 1, 0][..],
        ),
        (
            CustomSection {
                name: "é",
                data: &[0xff, 0],
            },
            &[0, 5, 2, 0xc3, 0xa9, 0xff, 0][..],
        ),
        (
            CustomSection {
                name: "é",
                data: b"second",
            },
            &[0, 9, 2, 0xc3, 0xa9, b's', b'e', b'c', b'o', b'n', b'd'][..],
        ),
    ] {
        let encoded = encode_custom_section(section, golden.len()).unwrap();
        assert_eq!(encoded, golden);
        assert!(matches!(
            encode_custom_section(section, golden.len() - 1),
            Err(CustomSectionEncodingError::LimitExceeded { actual, limit })
                if actual == golden.len() && limit == golden.len() - 1
        ));
        module.extend(encoded);
    }
    let facts = inspect(&module, LIMITS).unwrap();
    assert_eq!(
        facts.custom_sections,
        [
            CustomSection {
                name: "",
                data: &[]
            },
            CustomSection {
                name: "é",
                data: &[0xff, 0],
            },
            CustomSection {
                name: "é",
                data: b"second",
            },
        ]
    );
    assert!(matches!(
        encode_custom_section(
            CustomSection {
                name: "",
                data: &[]
            },
            0
        ),
        Err(CustomSectionEncodingError::LimitExceeded {
            actual: 3,
            limit: 0
        })
    ));
}

#[test]
fn custom_section_lengths_match_independent_framing_at_leb_boundaries() {
    for (name_bytes, data_bytes) in [
        (0, 126),
        (0, 127),
        (127, 0),
        (128, 0),
        (0, 16_382),
        (0, 16_383),
        (16_383, 0),
        (16_384, 0),
    ] {
        let name = "n".repeat(name_bytes);
        let data = vec![0xff; data_bytes];
        let mut expected = HEADER.to_vec();
        // Keep the parser fixture's independent framing owner as a byte oracle.
        custom(&mut expected, &name, &data);
        let encoded_bytes = expected.len() - HEADER.len();
        let section = CustomSection {
            name: &name,
            data: &data,
        };
        let encoded = encode_custom_section(section, encoded_bytes).unwrap();
        assert_eq!(encoded, expected[HEADER.len()..]);
        assert!(matches!(
            encode_custom_section(section, encoded_bytes - 1),
            Err(CustomSectionEncodingError::LimitExceeded { .. })
        ));
        let facts = inspect(
            &expected,
            InspectionLimits {
                module_bytes: expected.len(),
                ..LIMITS
            },
        )
        .unwrap();
        assert_eq!(facts.custom_sections, [section]);
    }
}

#[test]
fn custom_section_length_admission_rejects_overflow_before_allocation() {
    let maximum = usize::try_from(u32::MAX).unwrap();
    for (name_bytes, data_bytes) in [
        (usize::MAX, 0),
        (0, usize::MAX),
        (maximum, 0),
        (0, maximum),
        (128, maximum - 129),
    ] {
        assert!(matches!(
            custom_section_layout(name_bytes, data_bytes, usize::MAX),
            Err(CustomSectionEncodingError::LengthOverflow)
        ));
    }
    // Exercise the u32 maximum arithmetically without retaining gigabyte inputs.
    let encoded_bytes = usize::try_from(u64::from(u32::MAX) + 6);
    if let Ok(encoded_bytes) = encoded_bytes {
        assert_eq!(
            custom_section_layout(128, maximum - 130, encoded_bytes).unwrap(),
            (128, u32::MAX, encoded_bytes)
        );
        assert!(matches!(
            custom_section_layout(128, maximum - 130, encoded_bytes - 1),
            Err(CustomSectionEncodingError::LimitExceeded { actual, limit })
                if actual == encoded_bytes && limit == encoded_bytes - 1
        ));
    } else {
        assert!(matches!(
            custom_section_layout(128, maximum - 130, usize::MAX),
            Err(CustomSectionEncodingError::LengthOverflow)
        ));
    }
}

#[test]
fn reports_exact_metrics_exports_and_borrowed_metadata_in_encounter_order() {
    let bytes = fixture();
    let original = bytes.clone();
    let facts = inspect(&bytes, LIMITS).unwrap();
    assert_eq!(facts.raw_bytes, bytes.len());
    assert_eq!(facts.code_section_bytes, 4);
    assert_eq!(facts.code_body_bytes, 3);
    assert_eq!(facts.data_section_bytes, 6);
    assert_eq!(facts.defined_functions, 1);
    assert_eq!(facts.data_segments, 1);
    assert_eq!(
        facts.exports["canister_query status"],
        Export {
            kind: ExportKind::Function,
            index: 0
        }
    );
    assert_eq!(facts.exports["memory"].kind, ExportKind::Memory);
    assert_eq!(facts.custom_sections.len(), 2);
    assert_eq!(facts.custom_sections[0].data, b"service : {}\n");
    assert_eq!(facts.custom_sections[1].data, b"second declaration");
    let source_range = bytes.as_ptr_range();
    assert!(source_range.contains(&facts.custom_sections[0].data.as_ptr()));
    assert_eq!(bytes, original);
}

#[test]
fn code_body_measurement_excludes_the_actual_encoded_count_width() {
    let mut module = HEADER.to_vec();
    section(&mut module, 1, &[1, 0x60, 0, 0]);
    section(&mut module, 3, &[1, 0]);
    // A valid padded LEB count occupies two bytes, not its minimal one byte.
    section(&mut module, 10, &[0x81, 0, 2, 0, 0x0b]);
    let facts = inspect(&module, LIMITS).unwrap();
    assert_eq!(facts.code_section_bytes, 5);
    assert_eq!(facts.code_body_bytes, 3);
}

#[test]
fn imported_functions_and_globals_are_counted_separately_from_definitions() {
    let mut module = HEADER.to_vec();
    section(&mut module, 1, &[1, 0x60, 0, 0]);
    section(
        &mut module,
        2,
        &[2, 1, b'm', 1, b'f', 0, 0, 1, b'm', 1, b'g', 3, 0x7f, 0],
    );
    section(&mut module, 3, &[1, 0]);
    section(&mut module, 6, &[1, 0x7f, 0, 0x41, 0, 0x0b]);
    section(&mut module, 10, &[1, 2, 0, 0x0b]);
    let facts = inspect(&module, LIMITS).unwrap();
    assert_eq!(facts.defined_functions, 1);
    assert_eq!(facts.imported_functions, 1);
    assert_eq!(facts.defined_globals, 1);
    assert_eq!(facts.imported_globals, 1);
}

#[test]
fn malformed_import_and_global_vectors_do_not_yield_partial_facts() {
    for (id, payload) in [(2, &[1, 1, b'm'][..]), (6, &[1, 0x7f, 0, 0x41][..])] {
        let mut module = HEADER.to_vec();
        section(&mut module, id, payload);
        assert!(matches!(
            inspect(&module, LIMITS),
            Err(InspectionError::Parse(_))
        ));
    }
}

#[test]
fn accepts_empty_modules_and_exact_resource_bounds() {
    let empty = inspect(
        HEADER,
        InspectionLimits {
            module_bytes: 8,
            sections: 0,
            exports: 0,
            custom_sections: 0,
        },
    )
    .unwrap();
    assert_eq!(empty.code_section_bytes, 0);
    assert_eq!(empty.defined_functions, 0);
    assert!(empty.exports.is_empty());
    let bytes = fixture();
    inspect(
        &bytes,
        InspectionLimits {
            module_bytes: bytes.len(),
            sections: 8,
            exports: 2,
            custom_sections: 2,
        },
    )
    .unwrap();
}

#[test]
fn rejects_each_resource_bound_before_retaining_excess_entries() {
    let bytes = fixture();
    for (limits, resource) in [
        (
            InspectionLimits {
                module_bytes: bytes.len() - 1,
                ..LIMITS
            },
            InspectionResource::ModuleBytes,
        ),
        (
            InspectionLimits {
                sections: 7,
                ..LIMITS
            },
            InspectionResource::Sections,
        ),
        (
            InspectionLimits {
                exports: 1,
                ..LIMITS
            },
            InspectionResource::Exports,
        ),
        (
            InspectionLimits {
                custom_sections: 1,
                ..LIMITS
            },
            InspectionResource::CustomSections,
        ),
    ] {
        assert!(
            matches!(inspect(&bytes, limits), Err(InspectionError::LimitExceeded { resource: actual, .. }) if actual == resource)
        );
    }
}

#[test]
fn rejects_bad_headers_components_unknown_sections_and_invalid_lengths() {
    assert!(matches!(
        inspect(b"not wasm", LIMITS),
        Err(InspectionError::Parse(_))
    ));
    assert!(matches!(
        inspect(b"\0asm\x0d\0\x01\0", LIMITS),
        Err(InspectionError::UnsupportedEncoding)
    ));
    let unknown = [HEADER, &[42, 0]].concat();
    assert!(matches!(
        inspect(&unknown, LIMITS),
        Err(InspectionError::UnknownSection { id: 42, .. })
    ));
    for tail in [
        &[1, 3, 0][..],
        &[1, 0x80, 0x80, 0x80, 0x80, 0x10][..],
        &[10, 0x7f, 0][..], // No functions, but declared code bytes are absent.
        &[3, 2, 1, 0, 10, 0x7f, 1, 2, 0, 0x0b][..], // Body fits; section extent does not.
    ] {
        assert!(matches!(
            inspect(&[HEADER, tail].concat(), LIMITS),
            Err(InspectionError::Parse(_))
        ));
    }
}

#[test]
fn rejects_duplicate_and_out_of_order_sections_and_unequal_function_counts() {
    for tail in [
        &[7, 1, 0, 7, 1, 0][..],     // Duplicate export section.
        &[10, 1, 0, 3, 1, 0][..],    // Code before functions.
        &[3, 2, 1, 0][..],           // Defined function with no code.
        &[3, 2, 1, 0, 10, 1, 0][..], // Code count mismatch.
        &[10, 1, 1][..],             // Missing function body.
    ] {
        assert!(matches!(
            inspect(&[HEADER, tail].concat(), LIMITS),
            Err(InspectionError::Parse(_))
        ));
    }
}

#[test]
fn rejects_malformed_reported_vectors_and_duplicate_export_names() {
    for (id, payload) in [
        (3, &[0, 0][..]),             // Count zero followed by trailing type index.
        (7, &[1, 1, 0xff, 0, 0][..]), // Non-UTF-8 export name.
        (7, &[1, 1, b'x', 9, 0][..]), // Invalid kind.
        (7, &[0, 0][..]),             // Trailing export bytes.
        (11, &[1, 1, 2, b'a'][..]),   // Truncated passive data.
        (0, &[2, b'x'][..]),          // Truncated custom name.
    ] {
        let mut bytes = HEADER.to_vec();
        section(&mut bytes, id, payload);
        assert!(matches!(
            inspect(&bytes, LIMITS),
            Err(InspectionError::Parse(_))
        ));
    }
    let mut bytes = HEADER.to_vec();
    section(&mut bytes, 7, &[2, 1, b'x', 0, 0, 1, b'x', 2, 0]);
    assert!(matches!(
        inspect(&bytes, LIMITS),
        Err(InspectionError::DuplicateExport { .. })
    ));
}

#[test]
fn does_not_silently_claim_type_or_instruction_validation() {
    let mut bytes = HEADER.to_vec();
    // The structural export is well-formed even though function 99 does not exist.
    section(&mut bytes, 7, &[1, 1, b'x', 0, 99]);
    assert_eq!(inspect(&bytes, LIMITS).unwrap().exports["x"].index, 99);
}

#[test]
fn code_statistics_distinguish_payload_framing_and_indirect_call_kinds() {
    let mut module = HEADER.to_vec();
    section(&mut module, 1, &[1, 0x60, 0, 0]);
    section(&mut module, 3, &[1, 0]);
    // Locals, i32.const 0, call_indirect type=0/table=0,
    // i32.const 0, return_call_indirect type=0/table=0, end.
    section(
        &mut module,
        10,
        &[1, 12, 0, 0x41, 0, 0x11, 0, 0, 0x41, 0, 0x13, 0, 0, 0x0b],
    );
    section(
        &mut module,
        11,
        &[2, 1, 3, b'a', b'b', b'c', 1, 2, b'd', b'e'],
    );
    let statistics = code_statistics(&module, module.len()).unwrap();
    assert_eq!(
        statistics,
        WasmCodeStatistics {
            call_indirect_count: 2,
            data_payload_bytes: 5
        }
    );
    assert!(inspect(&module, LIMITS).unwrap().data_section_bytes > 5);
    assert!(matches!(
        code_statistics(&module, module.len() - 1),
        Err(InspectionError::LimitExceeded {
            resource: InspectionResource::ModuleBytes,
            ..
        })
    ));
    assert_eq!(
        code_statistics(HEADER, HEADER.len()).unwrap(),
        WasmCodeStatistics::default()
    );
}

#[test]
fn code_statistics_reject_malformed_operators_and_data_without_changing_inspect() {
    let mut malformed_code = HEADER.to_vec();
    section(&mut malformed_code, 3, &[1, 0]);
    section(&mut malformed_code, 10, &[1, 2, 0, 0xff]);
    assert!(inspect(&malformed_code, LIMITS).is_ok());
    assert!(matches!(
        code_statistics(&malformed_code, 4096),
        Err(InspectionError::Parse(_))
    ));
    let mut malformed_data = HEADER.to_vec();
    section(&mut malformed_data, 11, &[1, 1, 9, b'a']);
    assert!(matches!(
        code_statistics(&malformed_data, 4096),
        Err(InspectionError::Parse(_))
    ));
    assert!(matches!(
        code_statistics(b"\0asm\x0d\0\x01\0", 4096),
        Err(InspectionError::UnsupportedEncoding)
    ));
}

#[test]
fn code_statistics_count_instructions_not_operand_or_data_bytes() {
    let mut module = HEADER.to_vec();
    section(&mut module, 3, &[1, 0]);
    // The constants contain indirect-call opcode bytes, followed by a direct
    // call and an indirect call after unreachable. Only the encoded indirect
    // instruction counts; this is neither a byte search nor reachability analysis.
    let body = [
        0, 0x41, 0x11, 0x1a, 0x41, 0x13, 0x1a, 0x10, 0, 0x00, 0x41, 0, 0x11, 0, 0, 0x0b,
    ];
    let mut code = vec![1, u8::try_from(body.len()).unwrap()];
    code.extend(body);
    section(&mut module, 10, &code);
    // Active and passive payloads can contain the same opcode bytes as code.
    section(
        &mut module,
        11,
        &[2, 0, 0x41, 0, 0x0b, 3, 0x11, 0x13, 0x10, 1, 2, 0x11, 0x13],
    );
    assert_eq!(
        code_statistics(&module, module.len()).unwrap(),
        WasmCodeStatistics {
            call_indirect_count: 1,
            data_payload_bytes: 5,
        }
    );
}
