use super::*;
#[cfg(feature = "wasm")]
use crate::wasm::{InspectionLimits, inspect};

fn check(bytes: &[u8]) {
    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        payload.unwrap();
    }
    #[cfg(feature = "wasm")]
    inspect(
        bytes,
        InspectionLimits {
            module_bytes: bytes.len(),
            sections: 8,
            exports: 10,
            custom_sections: 0,
        },
    )
    .unwrap();
    #[cfg(feature = "wasm-validation")]
    crate::wasm::validate(bytes, bytes.len()).unwrap();
}

#[test]
fn preserves_imports_calls_exports_and_binary_data_across_pages() {
    let imports = [
        FunctionImport {
            module: "host",
            name: "append",
            parameters: 2,
        },
        FunctionImport {
            module: "host",
            name: "finish",
            parameters: 0,
        },
    ];
    let payload = vec![0xff; 65_537];
    let functions = [
        ExportedFunction {
            name: "read\"\\é",
            calls: vec![
                Call {
                    import: 0,
                    arguments: vec![0, 65_537],
                },
                Call {
                    import: 1,
                    arguments: vec![],
                },
            ],
        },
        ExportedFunction {
            name: "write",
            calls: vec![
                Call {
                    import: 0,
                    arguments: vec![65_537, 4],
                },
                Call {
                    import: 1,
                    arguments: vec![],
                },
            ],
        },
    ];
    let data = [
        DataSegment {
            offset: 0,
            bytes: &payload,
        },
        DataSegment {
            offset: 65_537,
            bytes: b"raw\0",
        },
    ];
    let module = CallModule {
        imports: &imports,
        functions: &functions,
        memory: Memory32 {
            pages: 2,
            export: "memory",
        },
        data: &data,
    };
    let bytes = encode(&module, 100_000).unwrap();
    check(&bytes);
    #[cfg(feature = "wasm")]
    {
        let facts = inspect(
            &bytes,
            InspectionLimits {
                module_bytes: bytes.len(),
                sections: 8,
                exports: 3,
                custom_sections: 0,
            },
        )
        .unwrap();
        assert_eq!(facts.imported_functions, 2);
        assert_eq!(facts.defined_functions, 2);
        assert_eq!(facts.data_segments, 2);
        assert_eq!(facts.exports.len(), 3);
    }
    assert!(bytes.windows(payload.len()).any(|window| window == payload));
    assert!(bytes.windows(4).any(|window| window == b"raw\0"));
    let mut calls = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(&bytes) {
        if let wasmparser::Payload::CodeSectionEntry(body) = payload.unwrap() {
            for operator in body.get_operators_reader().unwrap() {
                if let wasmparser::Operator::Call { function_index } = operator.unwrap() {
                    calls.push(function_index);
                }
            }
        }
    }
    assert_eq!(calls, [0, 1, 0, 1]);
}

#[test]
fn empty_catalog_and_empty_function_obey_exact_output_allowance() {
    let empty_functions = [ExportedFunction {
        name: "",
        calls: vec![],
    }];
    for functions in [&[][..], &empty_functions[..]] {
        let module = CallModule {
            imports: &[],
            functions,
            memory: Memory32 {
                pages: 0,
                export: "memory",
            },
            data: &[],
        };
        let bytes = encode(&module, 1024).unwrap();
        check(&bytes);
        assert_eq!(encode(&module, bytes.len()).unwrap(), bytes);
        for limit in [0, 7, bytes.len() - 1] {
            assert_eq!(
                encode(&module, limit),
                Err(EncodingError::LimitExceeded { limit })
            );
        }
    }
}

#[test]
fn refuses_duplicate_exports_and_invalid_import_calls() {
    let imports = [FunctionImport {
        module: "host",
        name: "consume",
        parameters: 1,
    }];
    for call in [
        Call {
            import: 1,
            arguments: vec![0],
        },
        Call {
            import: 0,
            arguments: vec![],
        },
        Call {
            import: 0,
            arguments: vec![0, 1],
        },
    ] {
        let functions = [ExportedFunction {
            name: "entry",
            calls: vec![call],
        }];
        let module = CallModule {
            imports: &imports,
            functions: &functions,
            memory: Memory32 {
                pages: 1,
                export: "memory",
            },
            data: &[],
        };
        assert_eq!(
            encode(&module, 1024),
            Err(EncodingError::InvalidCall {
                function: 0,
                call: 0
            })
        );
    }
    for names in [["memory", "other"], ["same", "same"]] {
        let functions = names.map(|name| ExportedFunction {
            name,
            calls: vec![],
        });
        let module = CallModule {
            imports: &[],
            functions: &functions,
            memory: Memory32 {
                pages: 1,
                export: "memory",
            },
            data: &[],
        };
        assert_eq!(encode(&module, 1024), Err(EncodingError::DuplicateExport));
    }
}

#[test]
fn admits_data_extents_and_preserves_unsigned_wasm32_addresses() {
    let mut module = CallModule {
        imports: &[],
        functions: &[],
        memory: Memory32 {
            pages: 1,
            export: "memory",
        },
        data: &[DataSegment {
            offset: 65_535,
            bytes: b"ab",
        }],
    };
    assert_eq!(
        encode(&module, 1024),
        Err(EncodingError::DataOutOfBounds { segment: 0 })
    );
    module.data = &[DataSegment {
        offset: 65_536,
        bytes: &[],
    }];
    check(&encode(&module, 1024).unwrap());
    module.memory.pages = 65_536;
    module.data = &[DataSegment {
        offset: 0x8000_0000,
        bytes: b"a",
    }];
    let bytes = encode(&module, 1024).unwrap();
    check(&bytes);
    let mut observed = false;
    for payload in wasmparser::Parser::new(0).parse_all(&bytes) {
        if let wasmparser::Payload::DataSection(section) = payload.unwrap() {
            for segment in section {
                let segment = segment.unwrap();
                let wasmparser::DataKind::Active { offset_expr, .. } = segment.kind else {
                    panic!("expected active data")
                };
                assert!(matches!(
                    offset_expr.get_operators_reader().read().unwrap(),
                    wasmparser::Operator::I32Const { value: i32::MIN }
                ));
                assert_eq!(segment.data, b"a");
                observed = true;
            }
        }
    }
    assert!(observed);
}

#[test]
fn refuses_invalid_memory_and_unrepresentable_lengths() {
    let module = CallModule {
        imports: &[],
        functions: &[],
        memory: Memory32 {
            pages: 65_537,
            export: "memory",
        },
        data: &[],
    };
    assert_eq!(encode(&module, 1024), Err(EncodingError::InvalidMemory));
    #[cfg(target_pointer_width = "64")]
    assert_eq!(length(usize::MAX), Err(EncodingError::LengthOverflow));
}
