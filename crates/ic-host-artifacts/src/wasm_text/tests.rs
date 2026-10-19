use super::*;

#[test]
fn exact_allowances_produce_the_independent_empty_module_golden() {
    let source = "(; UTF-8: é ;) (module)";
    let limits = CompileLimits {
        source_bytes: source.len(),
        module_bytes: 8,
    };
    assert_eq!(compile(source, limits).unwrap(), b"\0asm\x01\0\0\0");
    assert!(matches!(
        compile(source, CompileLimits { source_bytes: source.len() - 1, ..limits }),
        Err(CompileError::SourceLimit { actual, limit })
        if actual == source.len() && limit == source.len() - 1
    ));
    assert!(matches!(
        compile(
            source,
            CompileLimits {
                module_bytes: 7,
                ..limits
            }
        ),
        Err(CompileError::ModuleLimit {
            actual: 8,
            limit: 7
        })
    ));
}

#[test]
fn source_admission_precedes_parsing_and_diagnostics_remain_host_owned() {
    use std::error::Error as _;
    let limits = CompileLimits {
        source_bytes: 0,
        module_bytes: 0,
    };
    assert!(matches!(
        compile("invalid", limits),
        Err(CompileError::SourceLimit { .. })
    ));
    for source in ["", "(module", "(module (func call $missing))"] {
        let error = compile(
            source,
            CompileLimits {
                source_bytes: 1024,
                module_bytes: 1024,
            },
        )
        .unwrap_err();
        assert!(matches!(&error, CompileError::Invalid { diagnostic } if !diagnostic.is_empty()));
        assert!(error.source().is_none());
    }
}

#[test]
fn compiles_stateful_core_instructions_without_a_fixture_policy() {
    let source = r#"(module
        (memory (export "chosen-memory") 1 1)
        (global $count (mut i64) (i64.const 0))
        (data (i32.const 0) "raw\00\ff")
        (func (export "chosen-entry") (result i64)
            global.get $count i64.const 1 i64.add global.set $count
            global.get $count)
    )"#;
    let bytes = compile(
        source,
        CompileLimits {
            source_bytes: 1024,
            module_bytes: 1024,
        },
    )
    .unwrap();
    let mut exports = Vec::new();
    let mut data = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(&bytes) {
        match payload.unwrap() {
            wasmparser::Payload::ExportSection(section) => {
                exports.extend(section.into_iter().map(|export| export.unwrap().name));
            }
            wasmparser::Payload::DataSection(section) => {
                data.extend(section.into_iter().map(|segment| segment.unwrap().data));
            }
            _ => {}
        }
    }
    assert_eq!(exports, ["chosen-memory", "chosen-entry"]);
    assert_eq!(data, [b"raw\0\xff".as_slice()]);
    #[cfg(feature = "wasm-validation")]
    crate::wasm::validate(&bytes, bytes.len()).unwrap();
}

#[test]
fn component_and_nonmodule_binary_forms_do_not_escape_the_core_boundary() {
    let limits = CompileLimits {
        source_bytes: 1024,
        module_bytes: 1024,
    };
    assert!(compile("(component)", limits).is_err());
    assert!(matches!(
        compile(r#"(module binary "wrong")"#, limits),
        Err(CompileError::UnsupportedEncoding)
    ));
}

#[cfg(feature = "wasm-validation")]
#[test]
fn full_validation_is_separate_from_successful_text_compilation() {
    let source = "(module (func (result i32) i64.const 1))";
    let bytes = compile(
        source,
        CompileLimits {
            source_bytes: 1024,
            module_bytes: 1024,
        },
    )
    .unwrap();
    assert!(matches!(
        crate::wasm::validate(&bytes, bytes.len()),
        Err(crate::wasm::ValidationError::Invalid(_))
    ));
}
