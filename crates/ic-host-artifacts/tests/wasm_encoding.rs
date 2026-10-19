//! Public encoder contracts without depending on upstream encoder identities.

#[test]
fn owned_module_encoding_keeps_import_names_and_caller_policy_explicit() {
    use ic_host_artifacts::wasm_encoding::{
        Call, CallModule, DataSegment, ExportedFunction, FunctionImport, Memory32, encode,
    };
    let imports = [FunctionImport {
        module: "caller",
        name: "consume",
        parameters: 2,
    }];
    let functions = [ExportedFunction {
        name: "caller-selected-entry",
        calls: vec![Call {
            import: 0,
            arguments: vec![0, 4],
        }],
    }];
    let data = [DataSegment {
        offset: 0,
        bytes: b"raw\0",
    }];
    let bytes = encode(
        &CallModule {
            imports: &imports,
            functions: &functions,
            memory: Memory32 {
                pages: 1,
                export: "caller-selected-memory",
            },
            data: &data,
        },
        1024,
    )
    .unwrap();
    assert!(bytes.starts_with(b"\0asm\x01\0\0\0"));
    for name in [
        b"caller-selected-entry".as_slice(),
        b"caller-selected-memory",
    ] {
        assert!(bytes.windows(name.len()).any(|window| window == name));
    }
    #[cfg(feature = "wasm-validation")]
    ic_host_artifacts::wasm::validate(&bytes, bytes.len()).unwrap();
}
