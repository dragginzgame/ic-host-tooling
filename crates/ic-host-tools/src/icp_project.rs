//! Bounded environment/network projection from `icp project show` effective YAML.
//!
//! ICP owns project discovery, implicit environments and recipe expansion. This
//! module only interprets the resulting document. Callers own executable admission,
//! capture limits, selected targets, credentials, funding and network lifecycle.

use std::{collections::BTreeSet, fmt};

use serde::Deserialize;

/// Refusal to interpret one effective ICP project document.
#[derive(Debug)]
pub enum ProjectParseError {
    /// The caller's complete input-byte allowance was exceeded before parsing.
    InputTooLarge {
        /// Bytes provided by the caller.
        actual: usize,
        /// Maximum admitted input bytes.
        maximum: usize,
    },
    /// YAML syntax or the required environment field types were invalid.
    InvalidYaml(serde_yaml_ng::Error),
    /// An environment name or network name was empty.
    EmptyEnvironmentField {
        /// Zero-based environment position in the effective document.
        index: usize,
    },
}

impl fmt::Display for ProjectParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge { actual, maximum } => write!(
                formatter,
                "ICP project input is {actual} bytes; maximum is {maximum}"
            ),
            Self::InvalidYaml(error) => {
                write!(formatter, "invalid effective ICP project YAML: {error}")
            }
            Self::EmptyEnvironmentField { index } => write!(
                formatter,
                "ICP environment {index} has an empty name or network"
            ),
        }
    }
}

impl std::error::Error for ProjectParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidYaml(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
struct EffectiveProject {
    #[serde(deserialize_with = "deserialize_environments")]
    environments: Vec<Environment>,
}

#[derive(Deserialize)]
struct Environment {
    name: String,
    network: Network,
}

#[derive(Deserialize)]
struct Network {
    name: String,
}

// BTreeMap deserialization would overwrite duplicate keys. Retain input order
// and reject duplicate or contradictory identities before selecting a network.
fn deserialize_environments<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Environment>, D::Error> {
    struct EnvironmentVisitor;
    impl<'de> serde::de::Visitor<'de> for EnvironmentVisitor {
        type Value = Vec<Environment>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an effective ICP environment map")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            use serde::de::Error as _;
            let mut names = BTreeSet::new();
            let mut environments = Vec::new();
            while let Some((key, environment)) = map.next_entry::<String, Environment>()? {
                if key != environment.name {
                    return Err(A::Error::custom("environment key and name disagree"));
                }
                if !names.insert(key) {
                    return Err(A::Error::custom("duplicate environment identity"));
                }
                environments.push(environment);
            }
            Ok(environments)
        }
    }
    deserializer.deserialize_map(EnvironmentVisitor)
}

/// Return the network of one exact environment in effective ICP project YAML.
///
/// Consume `icp project show` output, rather than a raw manifest: this parser does
/// not resolve missing/default environments, includes, networks or recipes.
/// Unknown fields are ignored; required environment facts remain strict. A missing
/// selection returns `None`, and names are not trimmed or case-folded for lookup.
///
/// # Errors
/// Rejects input above `maximum_input_bytes` before YAML parsing, malformed or
/// multiple documents, missing/invalid fields, empty names and duplicate
/// environment identities. Callers retain their domain-specific failure policy.
pub fn environment_network(
    yaml: &[u8],
    environment: &str,
    maximum_input_bytes: usize,
) -> Result<Option<String>, ProjectParseError> {
    if yaml.len() > maximum_input_bytes {
        return Err(ProjectParseError::InputTooLarge {
            actual: yaml.len(),
            maximum: maximum_input_bytes,
        });
    }
    let project: EffectiveProject =
        serde_yaml_ng::from_slice(yaml).map_err(ProjectParseError::InvalidYaml)?;
    for (index, item) in project.environments.iter().enumerate() {
        if item.name.trim().is_empty() || item.network.name.trim().is_empty() {
            return Err(ProjectParseError::EmptyEnvironmentField { index });
        }
    }
    Ok(project
        .environments
        .into_iter()
        .find(|item| item.name == environment)
        .map(|item| item.network.name))
}

#[cfg(test)]
mod tests {
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
}
