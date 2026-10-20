//! One external-selection reader for pure subset checks and archive admission.

use crate::cargo::lockfile::{external_package, optional_string, required_string};
use ic_host_artifacts::artifact::Sha256Digest;
use std::{collections::BTreeMap, error::Error, fmt};

type ExternalRows = BTreeMap<(String, String, String), Option<String>>;

/// External lockfile comparison failed without changing either input.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExternalLockError {
    /// An input exceeds the caller's per-document byte allowance.
    LimitExceeded {
        /// `original` or `candidate` input role.
        input: &'static str,
        /// Caller-selected byte allowance.
        limit: usize,
    },
    /// An input is not UTF-8.
    Utf8 {
        /// Input role.
        input: &'static str,
        /// Original decoding failure.
        source: std::str::Utf8Error,
    },
    /// An input is not valid TOML.
    Parse {
        /// Input role.
        input: &'static str,
        /// Original parser failure.
        source: toml::de::Error,
    },
    /// An input has no package array.
    MissingPackages {
        /// Input role.
        input: &'static str,
    },
    /// A package record or external identity field is malformed.
    InvalidPackage {
        /// Input role.
        input: &'static str,
        /// Zero-based package-array position.
        index: usize,
        /// Failed structural or field requirement.
        reason: &'static str,
    },
    /// An external name/version/source identity appears more than once.
    DuplicateSelection {
        /// Input role.
        input: &'static str,
        /// Package name.
        package: String,
        /// Package version.
        version: String,
    },
    /// A candidate identity or checksum does not match the original.
    SelectionChanged {
        /// Candidate package name.
        package: String,
        /// Candidate package version.
        version: String,
    },
}

impl fmt::Display for ExternalLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded { input, limit } => {
                write!(f, "{input} lockfile exceeds {limit} bytes")
            }
            Self::Utf8 { input, source } => write!(f, "invalid {input} lockfile UTF-8: {source}"),
            Self::Parse { input, source } => write!(f, "invalid {input} lockfile: {source}"),
            Self::MissingPackages { input } => write!(f, "missing {input} lock packages"),
            Self::InvalidPackage {
                input,
                index,
                reason,
            } => {
                write!(f, "{input} lock package {index}: {reason}")
            }
            Self::DuplicateSelection {
                input,
                package,
                version,
            } => {
                write!(
                    f,
                    "duplicate {input} package selection: {package}@{version}"
                )
            }
            Self::SelectionChanged { package, version } => {
                write!(
                    f,
                    "packaging changed external selection: {package}@{version}"
                )
            }
        }
    }
}

impl Error for ExternalLockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Utf8 { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Require every candidate external package to match an original selection.
///
/// Both documents are bounded by `max_bytes` before decoding. Compare exact
/// name/version/source identities and optional checksum fields; registry
/// checksums must be canonical lowercase SHA-256. Duplicate external identities
/// refuse even when their checksums agree. Extra original packages are allowed.
/// An empty candidate package array is a valid subset.
///
/// Unsourced tables are outside this comparison: callers admit their local
/// package names, versions, paths, feature graph and nonempty-graph requirements
/// independently. This does not validate dependency edges or admit a release.
/// No I/O, resolution, tool execution, mutation or network access occurs.
///
/// # Errors
/// Returns typed byte-bound, UTF-8, TOML, record, duplicate or selection failures.
pub fn verify_external_lock_subset(
    original: &[u8],
    candidate: &[u8],
    max_bytes: usize,
) -> Result<(), ExternalLockError> {
    for (input, bytes) in [("original", original), ("candidate", candidate)] {
        if bytes.len() > max_bytes {
            return Err(ExternalLockError::LimitExceeded {
                input,
                limit: max_bytes,
            });
        }
    }
    let original = external_rows(original, "original")?;
    let candidate = external_rows(candidate, "candidate")?;
    verify_rows(&original, &candidate)
}

fn external_rows(bytes: &[u8], input: &'static str) -> Result<ExternalRows, ExternalLockError> {
    let text =
        std::str::from_utf8(bytes).map_err(|source| ExternalLockError::Utf8 { input, source })?;
    let lock: toml::Value =
        toml::from_str(text).map_err(|source| ExternalLockError::Parse { input, source })?;
    let mut rows = BTreeMap::new();
    for (index, package) in lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or(ExternalLockError::MissingPackages { input })?
        .iter()
        .enumerate()
    {
        let invalid = |reason| ExternalLockError::InvalidPackage {
            input,
            index,
            reason,
        };
        // Local identity admission stays with each consumer. Non-table records
        // cannot be Cargo packages, regardless of whether they expose a source.
        let external = external_package(package).map_err(invalid)?;
        let Some(table) = external else { continue };
        let text = |key| required_string(table, key).map_err(invalid);
        let source = text("source")?;
        let checksum = optional_string(table, "checksum").map_err(invalid)?;
        if source.starts_with("registry+") {
            checksum
                .ok_or_else(|| invalid("missing lock checksum"))?
                .parse::<Sha256Digest>()
                .map_err(|_| invalid("invalid package checksum"))?;
        }
        let name = text("name")?;
        let version = text("version")?;
        if rows
            .insert(
                (name.into(), version.into(), source.into()),
                checksum.map(str::to_owned),
            )
            .is_some()
        {
            return Err(ExternalLockError::DuplicateSelection {
                input,
                package: name.into(),
                version: version.into(),
            });
        }
    }
    Ok(rows)
}

fn verify_rows(original: &ExternalRows, candidate: &ExternalRows) -> Result<(), ExternalLockError> {
    for (key @ (package, version, _), checksum) in candidate {
        if original.get(key) != Some(checksum) {
            return Err(ExternalLockError::SelectionChanged {
                package: package.clone(),
                version: version.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(any(unix, test))]
pub(super) fn admit_lock(entry: &[u8], payload: &[u8]) -> Result<(), Box<dyn Error>> {
    let entry = external_rows(entry, "original")?;
    let payload = external_rows(payload, "candidate")?;
    if payload.is_empty() {
        return Err("empty package external graph".into());
    }
    verify_rows(&entry, &payload)?;
    Ok(())
}

#[cfg(test)]
mod tests;
