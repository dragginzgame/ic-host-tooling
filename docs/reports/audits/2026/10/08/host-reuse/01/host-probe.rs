use ic_host_artifacts::wasm::{inspect, InspectionLimits};
use ic_host_tools::install_limits::REFERENCE_LIMITS;
fn main() {
    for (name, expected) in [
        ("imported-function-limit", Some((50000, 1, 0))),
        ("global-before-function", Some((1, 1, 1))),
        ("truncated-code", None),
    ] {
        let bytes = std::fs::read(format!("../{name}.wasm")).unwrap();
        let facts = inspect(&bytes, InspectionLimits {
            module_bytes: 1024 * 1024, sections: 32, exports: 16, custom_sections: 16,
        });
        match expected {
            Some((defined, imported, globals)) => {
                let facts = facts.unwrap();
                assert_eq!((facts.defined_functions, facts.imported_functions, facts.imported_globals), (defined, imported, globals));
                let report = REFERENCE_LIMITS.report(&facts);
                assert!(!report.defined_functions.exceeded());
                println!("{name}: defined={} imported={} imported_globals={} code_payload={} code_body={} function_limit_exceeded={}", facts.defined_functions, facts.imported_functions, facts.imported_globals, facts.code_section_bytes, facts.code_body_bytes, report.defined_functions.exceeded());
            }
            None => {assert!(facts.is_err()); println!("{name}: rejected: {}", facts.unwrap_err());}
        }
    }
}
