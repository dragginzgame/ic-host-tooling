use super::*;

#[test]
fn block_flow_quoted_and_expanded_environment_facts_are_exact() {
    for yaml in [
        "networks: {local: {name: local}}\ncanisters: [{name: example}]\nenvironments:\n  'demo-extra': {name: demo-extra, network: {name: ic}}\n  demo:\n    name: \"demo\"\n    network:\n      name: local\n      configuration: {mode: managed, gateway: {port: !Fixed 8002}}\n    canisters: {example: [some/path, {name: example}]}\n  local: {name: local, network: {name: local}}\n  ic: {name: ic, network: {name: ic}}\n",
        "environments: {demo-extra: {name: demo-extra, network: {name: ic}}, demo: {name: demo, network: {name: 'local'}}, local: {name: local, network: {name: local}}, ic: {name: ic, network: {name: ic}}}",
    ] {
        for (name, expected) in [
            ("demo", Some("local")),
            ("demo-extra", Some("ic")),
            ("local", Some("local")),
            ("ic", Some("ic")),
            ("Demo", None),
            ("missing", None),
        ] {
            assert_eq!(
                environment_network(yaml.as_bytes(), name, yaml.len())
                    .unwrap()
                    .as_deref(),
                expected
            );
        }
    }
}

#[test]
fn byte_allowance_precedes_parsing_and_admits_exact_boundary() {
    let yaml = b"environments: {demo: {name: demo, network: {name: local}}}";
    assert_eq!(
        environment_network(yaml, "demo", yaml.len())
            .unwrap()
            .as_deref(),
        Some("local")
    );
    assert!(matches!(
        environment_network(yaml, "demo", yaml.len() - 1),
        Err(ProjectParseError::InputTooLarge { .. })
    ));
    assert!(matches!(
        environment_network(b"[", "demo", 0),
        Err(ProjectParseError::InputTooLarge { .. })
    ));
    assert!(matches!(
        environment_network(b"", "demo", 0),
        Err(ProjectParseError::InvalidYaml(_))
    ));
    assert_eq!(
        environment_network(b"environments: {}", "demo", 32).unwrap(),
        None
    );
}

#[test]
fn malformed_missing_and_duplicate_facts_refuse_projection() {
    for yaml in [
        "[",
        "{}",
        "environments: null",
        "environments: []",
        "environments: {demo: {name: demo}}",
        "environments: {demo: {name: demo, network: local}}",
        "environments: {demo: {name: demo, network: {}}}",
        "environments: {demo: {name: demo, name: other, network: {name: local}}}",
        "environments: {demo: {name: demo, network: {name: local, name: ic}}}",
        "environments: {}\n---\nenvironments: {}",
        "environments: {}\nenvironments: {}",
        "environments: {demo: {name: other, network: {name: local}}}",
        "environments: {demo: {name: demo, network: {name: local}}, demo: {name: demo, network: {name: ic}}}",
    ] {
        for selected in ["demo", "missing"] {
            assert!(
                matches!(
                    environment_network(yaml.as_bytes(), selected, yaml.len()),
                    Err(ProjectParseError::InvalidYaml(_))
                ),
                "{yaml}"
            );
        }
    }
    for yaml in [
        "environments: {'': {name: '', network: {name: local}}}",
        "environments: {demo: {name: demo, network: {name: ' '}}}",
    ] {
        assert!(matches!(
            environment_network(yaml.as_bytes(), "demo", yaml.len()),
            Err(ProjectParseError::EmptyEnvironmentField { .. })
        ));
    }
}
