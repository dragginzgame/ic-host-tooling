//! Strict semantic lockfile comparison after caller-selected workspace version changes.

use std::collections::HashMap;

/// One unsourced workspace package whose version may change; all other parsed
/// lockfile content must remain identical. Release version policy is caller-owned.
#[derive(Clone, Copy, Debug)]
pub struct WorkspaceVersionProjection<'a> {
    /// Exact package name, selecting only records with no source field.
    pub package: &'a str,
    /// Expected version in the saved input.
    pub previous: &'a str,
    /// Only permitted replacement version.
    pub next: &'a str,
}

/// Strict version projection failed without changing either input.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LockProjectionError {
    /// No changes selected, or a name/version is empty or a name repeated.
    #[error("invalid workspace version selection")]
    InvalidSelection,
    /// One selected input is not valid TOML.
    #[error("invalid {input} lockfile: {source}")]
    Parse {
        /// `before` or `after` input role.
        input: &'static str,
        /// Original parser error.
        source: toml::de::Error,
    },
    /// The saved lockfile has no package array.
    #[error("lockfile has no package array")]
    MissingPackages,
    /// A name must identify exactly one unsourced package.
    #[error("workspace package {package} has {matches} unsourced records; expected one")]
    PackageSelection {
        /// Requested workspace package.
        package: String,
        /// Number of matching unsourced records.
        matches: usize,
    },
    /// The saved selected package does not have its expected prior version.
    #[error("unexpected previous workspace version for {package}")]
    PreviousVersion {
        /// Requested workspace package.
        package: String,
    },
    /// The after input differs from the exact permitted projection.
    #[error("lockfile differs from the selected workspace version projection")]
    ChangedInputs,
}

/// Verify that only explicitly selected unsourced package versions changed.
///
/// This is independent of any renderer. Parsed equality permits TOML comments
/// and table-field ordering differences, but preserves array order, dependencies,
/// source/checksum records and every other value. Registry records with the same
/// name are never projected. No file IO, dependency resolution, version selection
/// or release effect occurs. Callers own byte limits and saved-input custody.
///
/// # Errors
/// Refuses invalid/duplicate selections, parse failures, missing/ambiguous workspace
/// packages, unexpected prior versions and any other parsed difference.
pub fn verify_workspace_version_projection(
    before: &str,
    after: &str,
    projections: &[WorkspaceVersionProjection<'_>],
) -> Result<(), LockProjectionError> {
    if projections.is_empty() {
        return Err(LockProjectionError::InvalidSelection);
    }
    let mut selected = HashMap::with_capacity(projections.len());
    for item in projections {
        if item.package.is_empty()
            || item.previous.is_empty()
            || item.next.is_empty()
            || selected.insert(item.package, Vec::new()).is_some()
        {
            return Err(LockProjectionError::InvalidSelection);
        }
    }
    let mut expected: toml::Value =
        toml::from_str(before).map_err(|source| LockProjectionError::Parse {
            input: "before",
            source,
        })?;
    let packages = expected
        .get_mut("package")
        .and_then(toml::Value::as_array_mut)
        .ok_or(LockProjectionError::MissingPackages)?;
    for (index, package) in packages.iter().enumerate() {
        if package.get("source").is_none()
            && let Some(matches) = package
                .get("name")
                .and_then(toml::Value::as_str)
                .and_then(|name| selected.get_mut(name))
        {
            matches.push(index);
        }
    }
    for projection in projections {
        let matches = &selected[projection.package];
        for &index in matches {
            let package = &mut packages[index];
            if package.get("version").and_then(toml::Value::as_str) != Some(projection.previous) {
                return Err(LockProjectionError::PreviousVersion {
                    package: projection.package.to_owned(),
                });
            }
            package["version"] = toml::Value::String(projection.next.to_owned());
        }
        if matches.len() != 1 {
            return Err(LockProjectionError::PackageSelection {
                package: projection.package.to_owned(),
                matches: matches.len(),
            });
        }
    }
    let actual: toml::Value =
        toml::from_str(after).map_err(|source| LockProjectionError::Parse {
            input: "after",
            source,
        })?;
    if expected != actual {
        return Err(LockProjectionError::ChangedInputs);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
