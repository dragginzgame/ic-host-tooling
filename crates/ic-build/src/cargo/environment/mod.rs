//! Read-only Cargo configuration facts; callers retain environment and admission policy.

#[cfg(test)]
mod tests;

use ic_host_artifacts::artifact::Sha256Digest;
use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};
use toml::Value;

/// One observed configuration's location, raw-byte identity and compiler overrides.
///
/// Full configuration contents, which may contain credentials, are not retained.
pub struct ConfigurationFile {
    /// Original lookup path, preserving file/directory symlink replacement checks.
    pub path: PathBuf,
    /// Canonical containing directory plus the original file name, for stable labels.
    pub location: PathBuf,
    /// SHA-256 of the same UTF-8 bytes parsed during this observation.
    pub sha256: Sha256Digest,
    /// Present build compiler/wrapper fields, in canonical field order.
    pub compiler_overrides: Vec<&'static str>,
}

/// Configuration observation failure; diagnostics exclude configuration contents.
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    /// Original filesystem failure at its selected lookup path.
    #[error("{operation} {}: {source}", path.display())]
    Io {
        /// Failed read or location operation.
        operation: &'static str,
        /// Selected lookup path.
        path: PathBuf,
        /// Original filesystem error.
        source: io::Error,
    },
    /// Invalid TOML or include structure, without embedding source text.
    #[error("invalid Cargo configuration {}: {message}", path.display())]
    Invalid {
        /// Selected configuration path.
        path: PathBuf,
        /// Structural reason, excluding configuration values.
        message: String,
    },
}

/// Enumerate invocation ancestors' `.cargo` directories and the caller-selected Cargo home.
///
/// Relative home values resolve against the absolute invocation directory. No ambient
/// environment is read, files selected, or directories created.
///
/// # Errors
/// Returns invalid input when the invocation directory is not absolute.
pub fn configuration_directories(
    invocation: &Path,
    cargo_home: Option<&OsStr>,
    default_home: Option<&OsStr>,
) -> io::Result<Vec<PathBuf>> {
    if !invocation.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cargo invocation directory must be absolute",
        ));
    }
    let mut directories: Vec<_> = invocation
        .ancestors()
        .map(|path| path.join(".cargo"))
        .collect();
    let home = cargo_home
        .map(PathBuf::from)
        .or_else(|| default_home.map(|home| PathBuf::from(home).join(".cargo")));
    if let Some(home) = home {
        directories.push(if home.is_absolute() {
            home
        } else {
            invocation.join(home)
        });
    }
    Ok(directories)
}

/// Select Cargo's preferred existing file in one directory: `config` before `config.toml`.
/// Callers needing conservative custody of both paths may supply both to observation.
#[must_use]
pub fn preferred_configuration(directory: &Path) -> Option<PathBuf> {
    let extensionless = directory.join("config");
    if extensionless.exists() {
        return Some(extensionless);
    }
    let toml = directory.join("config.toml");
    toml.exists().then_some(toml)
}

/// Observe selected roots and their include trees in order, deduplicating lookup paths.
///
/// Roots must be absolute. Required missing/invalid files fail; optional absent includes
/// are skipped. Includes resolve beside the configured entry, not a symlink referent.
/// Recorded lookup paths are not read or expanded again in the same call. Active
/// recursion is suppressed without merging distinct lookup aliases.
///
/// Callers own source custody, root selection and re-observation before acceptance.
/// This does not resolve tools, execute Cargo, admit compilers or grant cache/release validity.
///
/// # Errors
/// Returns original filesystem errors or a structural refusal for relative roots,
/// invalid TOML and malformed includes. Error text excludes configuration contents.
pub fn observe_configurations(
    roots: &[PathBuf],
) -> Result<Vec<ConfigurationFile>, ConfigurationError> {
    let mut files = Vec::new();
    let mut active = BTreeSet::new();
    for root in roots {
        append_configuration(&mut files, root, &mut active, false)?;
    }
    Ok(files)
}

/// Whether a variable overrides the selected Rust compiler, Rustdoc or their wrappers.
/// The caller decides whether to admit or refuse such an override.
#[must_use]
pub fn is_compiler_override(name: &str) -> bool {
    matches!(
        name,
        "RUSTC"
            | "RUSTC_WRAPPER"
            | "RUSTC_WORKSPACE_WRAPPER"
            | "RUSTDOC"
            | "CARGO_BUILD_RUSTC"
            | "CARGO_BUILD_RUSTC_WRAPPER"
            | "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"
            | "CARGO_BUILD_RUSTDOC"
    )
}

/// Whether a variable supplies Rust flags or Cargo build/profile/target settings.
/// Output-only `CARGO_TARGET_DIR` is excluded; callers still select their identity policy.
#[must_use]
pub fn is_build_variable(name: &str) -> bool {
    matches!(
        name,
        "RUSTFLAGS" | "RUSTDOCFLAGS" | "CARGO_ENCODED_RUSTFLAGS" | "CARGO_ENCODED_RUSTDOCFLAGS"
    ) || name.starts_with("CARGO_BUILD_")
        || name.starts_with("CARGO_PROFILE_")
        || name.starts_with("CARGO_TARGET_") && name != "CARGO_TARGET_DIR"
}

fn append_configuration(
    files: &mut Vec<ConfigurationFile>,
    config: &Path,
    active: &mut BTreeSet<PathBuf>,
    optional: bool,
) -> Result<(), ConfigurationError> {
    if !config.is_absolute() {
        return Err(invalid_configuration(
            config,
            "configuration path must be absolute",
        ));
    }
    if files.iter().any(|file| file.path == config) {
        return Ok(());
    }
    let contents = match fs::read_to_string(config) {
        Ok(contents) => contents,
        Err(error) if optional && error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(ConfigurationError::Io {
                operation: "read Cargo configuration",
                path: config.to_owned(),
                source,
            });
        }
    };
    let parent = config.parent().ok_or_else(|| {
        invalid_configuration(config, "configuration path has no parent directory")
    })?;
    let location =
        parent
            .canonicalize()
            .map_err(|source| ConfigurationError::Io {
                operation: "resolve Cargo configuration directory",
                path: parent.to_owned(),
                source,
            })?
            .join(config.file_name().ok_or_else(|| {
                invalid_configuration(config, "configuration path has no file name")
            })?);
    if !active.insert(location.clone()) {
        return Ok(());
    }
    let configuration = toml::from_str::<Value>(&contents)
        .map_err(|_| invalid_configuration(config, "invalid TOML"))?;
    let compiler_overrides = [
        "rustc",
        "rustc-wrapper",
        "rustc-workspace-wrapper",
        "rustdoc",
    ]
    .into_iter()
    .filter(|field| {
        configuration
            .get("build")
            .and_then(|build| build.get(field))
            .is_some()
    })
    .collect();
    files.push(ConfigurationFile {
        path: config.to_owned(),
        location: location.clone(),
        sha256: Sha256Digest::compute(contents.as_bytes()),
        compiler_overrides,
    });
    if let Some(include) = configuration.get("include") {
        for (included, optional) in configuration_includes(include, config)? {
            let included = if included.is_absolute() {
                Cow::Borrowed(included)
            } else {
                Cow::Owned(parent.join(included))
            };
            append_configuration(files, &included, active, optional)?;
        }
    }
    active.remove(&location);
    Ok(())
}

fn configuration_includes<'a>(
    include: &'a Value,
    config: &Path,
) -> Result<Vec<(&'a Path, bool)>, ConfigurationError> {
    let values = match include {
        Value::Array(values) => values.as_slice(),
        value => std::slice::from_ref(value),
    };
    values
        .iter()
        .map(|value| match value {
            Value::String(path) => Ok((Path::new(path), false)),
            Value::Table(table) => {
                let path = table.get("path").and_then(Value::as_str).ok_or_else(|| {
                    invalid_configuration(
                        config,
                        "Cargo configuration include table requires a string `path`",
                    )
                })?;
                let optional = table
                    .get("optional")
                    .map(|value| {
                        value.as_bool().ok_or_else(|| {
                            invalid_configuration(
                                config,
                                "Cargo configuration include `optional` must be a boolean",
                            )
                        })
                    })
                    .transpose()?
                    .unwrap_or(false);
                Ok((Path::new(path), optional))
            }
            _ => Err(invalid_configuration(
                config,
                "Cargo configuration `include` must contain paths or include tables",
            )),
        })
        .collect()
}

fn invalid_configuration(path: &Path, message: &str) -> ConfigurationError {
    ConfigurationError::Invalid {
        path: path.to_owned(),
        message: message.to_owned(),
    }
}
