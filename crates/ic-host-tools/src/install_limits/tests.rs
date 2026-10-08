use super::*;
use ic_host_artifacts::wasm::{InspectionLimits, inspect};

#[test]
fn reference_comparison_uses_body_bytes_local_functions_and_all_globals() {
    let mut facts = inspect(
        b"\0asm\x01\0\0\0",
        InspectionLimits {
            module_bytes: 8,
            sections: 0,
            exports: 0,
            custom_sections: 0,
        },
    )
    .unwrap();
    facts.code_body_bytes = 12 * 1024 * 1024;
    facts.code_section_bytes = facts.code_body_bytes + 3;
    facts.defined_functions = 50_000;
    facts.imported_functions = 500;
    facts.defined_globals = 999;
    facts.imported_globals = 1;
    let report = REFERENCE_LIMITS.report(&facts);
    for usage in [
        report.code_body_bytes,
        report.defined_functions,
        report.globals,
    ] {
        assert!(!usage.exceeded());
        assert_eq!(usage.headroom(), 0);
    }
    facts.code_body_bytes += 1;
    facts.defined_functions += 1;
    facts.imported_globals += 1;
    let report = REFERENCE_LIMITS.report(&facts);
    for usage in [
        report.code_body_bytes,
        report.defined_functions,
        report.globals,
    ] {
        assert!(usage.exceeded());
        assert_eq!(usage.headroom(), -1);
    }
}

#[test]
fn caller_limits_and_large_headroom_are_not_replaced_by_reference_policy() {
    let usage = LimitUsage {
        observed: u64::MAX,
        limit: 0,
    };
    assert_eq!(usage.headroom(), -i128::from(u64::MAX));
    assert!(usage.exceeded());
    let facts = inspect(
        b"\0asm\x01\0\0\0",
        InspectionLimits {
            module_bytes: 8,
            sections: 0,
            exports: 0,
            custom_sections: 0,
        },
    )
    .unwrap();
    let report = InstallLimits {
        code_body_bytes: 7,
        defined_functions: 3,
        globals: 2,
    }
    .report(&facts);
    assert_eq!(report.code_body_bytes.headroom(), 7);
    assert_eq!(report.defined_functions.headroom(), 3);
    assert_eq!(report.globals.headroom(), 2);
}
