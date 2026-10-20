//! Bounded environment/network projection from `icp project show` effective YAML.
//!
//! ICP owns project discovery, implicit environments and recipe expansion. This
//! module only interprets the resulting document. Callers own executable admission,
//! capture limits, selected targets, credentials, funding and network lifecycle.

use std::fmt;

use ic_host_text::yaml::{self, DecodePolicy};
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
    InvalidYaml(yaml::DecodeError),
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

// The YAML boundary rejects duplicate keys throughout the document. This visitor
// retains input order and checks the ICP-specific key/name identity contract.
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
            let mut environments = Vec::new();
            while let Some((key, environment)) = map.next_entry::<String, Environment>()? {
                if key != environment.name {
                    return Err(A::Error::custom("environment key and name disagree"));
                }
                environments.push(environment);
            }
            Ok(environments)
        }
    }
    // Require an actual environment mapping, including when empty.
    deserializer.deserialize_any(EnvironmentVisitor)
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
/// environment identities. Unknown tags in ignored effective fields are allowed;
/// merge keys are not expanded. Callers retain their domain-specific failure policy.
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
    let project: EffectiveProject = yaml::from_slice(
        yaml,
        maximum_input_bytes,
        DecodePolicy {
            ignore_unknown_tags: true,
            preserve_merge_keys: true,
        },
    )
    .map_err(ProjectParseError::InvalidYaml)?;
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
mod tests;
