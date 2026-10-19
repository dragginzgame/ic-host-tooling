use super::*;
#[cfg(feature = "candid")]
use crate::candid::service::{ServiceLimits, inspect as inspect_service};
use ic_host_artifacts::wasm::{Export, InspectionLimits, inspect};
use std::collections::BTreeSet;

#[test]
fn projection_preserves_names_modes_and_internal_exports_without_acceptance_policy() {
    // This fixture tests projection over structural facts, not Wasm validation.
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
    for (name, kind) in [
        ("canister_query read with spaces", ExportKind::Function),
        ("canister_update read with spaces", ExportKind::Function),
        ("canister_composite_query composed", ExportKind::Function),
        (
            "canister_update <ic-cdk internal> timer_executor",
            ExportKind::Function,
        ),
        ("canister_query ", ExportKind::Function),
        ("canister_query not_a_function", ExportKind::Global),
        ("canister_init", ExportKind::Function),
        ("canister_query", ExportKind::Function),
        ("get_candid_pointer", ExportKind::Function),
    ] {
        facts.exports.insert(name, Export { kind, index: 0 });
    }
    let methods = exported_methods(&facts).collect::<BTreeSet<_>>();
    assert_eq!(
        methods,
        BTreeSet::from([
            ExportedMethod {
                name: "read with spaces",
                mode: MethodMode::Query
            },
            ExportedMethod {
                name: "read with spaces",
                mode: MethodMode::Update
            },
            ExportedMethod {
                name: "composed",
                mode: MethodMode::CompositeQuery
            },
            ExportedMethod {
                name: "<ic-cdk internal> timer_executor",
                mode: MethodMode::Update
            },
            ExportedMethod {
                name: "",
                mode: MethodMode::Query
            },
        ])
    );
}

#[cfg(feature = "candid")]
#[test]
fn declared_and_exported_modes_can_be_compared_without_discarding_mismatches() {
    let methods = inspect_service(
        "service : { read : () -> () query; notify : () -> () oneway; };",
        ServiceLimits {
            source_bytes: 128,
            methods: 2,
        },
    )
    .unwrap();
    let mut module = b"\0asm\x01\0\0\0\x01\x04\x01\x60\0\0\x03\x02\x01\0".to_vec();
    let mut exports = vec![2];
    for name in ["canister_update read", "canister_update notify"] {
        exports.push(u8::try_from(name.len()).unwrap());
        exports.extend_from_slice(name.as_bytes());
        exports.extend_from_slice(&[0, 0]);
    }
    module.extend_from_slice(&[7, u8::try_from(exports.len()).unwrap()]);
    module.extend_from_slice(&exports);
    module.extend_from_slice(&[10, 4, 1, 2, 0, 11]);
    let facts = inspect(
        &module,
        InspectionLimits {
            module_bytes: 1024,
            sections: 4,
            exports: 2,
            custom_sections: 0,
        },
    )
    .unwrap();
    let declared = methods
        .iter()
        .map(|method| method.as_export())
        .collect::<BTreeSet<_>>();
    let exported = exported_methods(&facts).collect::<BTreeSet<_>>();
    assert_eq!(
        declared.difference(&exported).copied().collect::<Vec<_>>(),
        vec![ExportedMethod {
            name: "read",
            mode: MethodMode::Query
        }]
    );
    assert_eq!(
        exported.difference(&declared).copied().collect::<Vec<_>>(),
        vec![ExportedMethod {
            name: "read",
            mode: MethodMode::Update
        }]
    );
}
