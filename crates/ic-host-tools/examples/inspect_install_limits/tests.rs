use super::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

fn leb(mut value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let low = u8::try_from(value & 0x7f).unwrap();
        value >>= 7;
        bytes.push(low | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            return bytes;
        }
    }
}

fn section(module: &mut Vec<u8>, id: u8, payload: &[u8]) {
    module.push(id);
    module.extend(leb(u32::try_from(payload.len()).unwrap()));
    module.extend(payload);
}

fn module(functions: u32, imported_globals: u32) -> Vec<u8> {
    let mut bytes = b"\0asm\x01\0\0\0".to_vec();
    section(&mut bytes, 1, &[1, 0x60, 0, 0]);
    let mut imports = leb(imported_globals + 1);
    imports.extend([1, b'm', 1, b'f', 0, 0]); // One imported function.
    for _ in 0..imported_globals {
        imports.extend([1, b'm', 1, b'g', 3, 0x7f, 0]);
    }
    section(&mut bytes, 2, &imports);
    let mut declarations = leb(functions);
    declarations.resize(declarations.len() + usize::try_from(functions).unwrap(), 0);
    section(&mut bytes, 3, &declarations);
    let mut code = leb(functions);
    for _ in 0..functions {
        code.extend([2, 0, 0x0b]);
    }
    section(&mut bytes, 10, &code);
    bytes
}

struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ic-host-limits-example-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = File::create_new(&path).unwrap();
        io::Write::write_all(&mut file, bytes).unwrap();
        Self(path)
    }
    fn run(&self, max_bytes: usize, json: bool) -> Result<(ExitCode, Vec<u8>), Box<dyn Error>> {
        let mut args = vec![
            self.0.as_os_str().to_owned(),
            max_bytes.to_string().into(),
            "10".into(),
            "0".into(),
            "0".into(),
            "0".into(),
        ];
        if json {
            args.push("--json".into());
        }
        let mut output = Vec::new();
        let status = run(args.into_iter(), &mut output)?;
        Ok((status, output))
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn text_and_json_have_identical_enforcement_with_imports_and_soft_warnings() {
    for (functions, globals, accepted) in
        [(50_000, 1_000, true), (50_001, 1, false), (1, 1_001, false)]
    {
        let bytes = module(functions, globals);
        let input = Input::new(&bytes);
        let (text_status, text) = input.run(bytes.len(), false).unwrap();
        let (json_status, json) = input.run(bytes.len(), true).unwrap();
        assert_eq!(text_status, json_status);
        assert_eq!(
            text_status,
            if accepted {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        );
        let text = String::from_utf8(text).unwrap();
        assert!(text.contains(&format!("within_selected_limits={accepted}")));
        let json: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(json["within_selected_limits"], accepted);
        assert_eq!(json["defined_functions"]["observed"], functions);
        assert_eq!(json["imported_functions"], 1);
        assert_eq!(json["globals"]["observed"], globals);
        assert_eq!(json["reference_revision"], REFERENCE_REVISION);
        // Raw soft warnings are diagnostic, not an additional rejection policy.
        assert_eq!(json["raw_warning"], true);
    }
}

#[test]
fn malformed_truncated_and_raw_over_budget_inputs_fail_for_both_formats() {
    for bytes in [b"not Wasm".as_slice(), b"\0asm\x01\0\0\0\x01\x04\x01"] {
        let input = Input::new(bytes);
        for json in [false, true] {
            assert!(input.run(bytes.len(), json).is_err());
        }
    }
    let bytes = module(1, 0);
    let input = Input::new(&bytes);
    for json in [false, true] {
        assert!(input.run(bytes.len() - 1, json).is_err());
    }
}

#[test]
fn code_body_boundary_controls_the_decision_not_full_section_bytes() {
    let bytes = module(1, 0);
    let mut facts = inspect(
        &bytes,
        InspectionLimits {
            module_bytes: bytes.len(),
            sections: 10,
            exports: 0,
            custom_sections: 0,
        },
    )
    .unwrap();
    // Reuse the report API's exact/+1 boundary contract without a 12 MiB fixture.
    facts.code_body_bytes = usize::try_from(REFERENCE_LIMITS.code_body_bytes).unwrap();
    facts.code_section_bytes = facts.code_body_bytes + 3;
    assert!(within_limits(REFERENCE_LIMITS.report(&facts)));
    facts.code_body_bytes += 1;
    assert!(!within_limits(REFERENCE_LIMITS.report(&facts)));
}
