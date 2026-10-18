//! Filesystem discovery of the nearest explicit Cargo workspace declaration.
//!
//! This is not Cargo membership resolution: standalone packages without a
//! `[workspace]` return no match. Use `observation::workspace_root_for` for
//! Cargo-resolved membership (including `package.workspace`).

use std::fmt;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const WORKSPACE_MANIFEST_RELATIVE: &str = "Cargo.toml";

/// Typed failure while locating an explicit Cargo workspace.
#[derive(Debug)]
pub enum DiscoveryError {
    /// Filesystem metadata could not be read.
    Inspect {
        /// Failed lookup path.
        path: PathBuf,
        /// Original filesystem cause.
        source: io::Error,
    },

    /// The start is neither a regular file nor a directory.
    UnsupportedPath {
        /// Rejected start path.
        path: PathBuf,
    },

    /// A manifest marker is not a regular file.
    ExpectedFile {
        /// Rejected marker path.
        path: PathBuf,
    },

    /// The manifest contents could not be read.
    ReadManifest {
        /// Failed lookup path.
        path: PathBuf,
        /// Original filesystem cause.
        source: io::Error,
    },

    /// The manifest is not valid TOML.
    ParseManifest {
        /// Failed lookup path.
        path: PathBuf,
        /// Original parse cause.
        source: Box<toml::de::Error>,
    },

    /// The start path could not be canonicalized.
    Canonicalize {
        /// Failed lookup path.
        path: PathBuf,
        /// Original filesystem cause.
        source: io::Error,
    },
}

/// Resolve the nearest explicit workspace from an existing file or directory.
///
/// # Errors
/// Returns typed filesystem and manifest failures without skipping invalid markers.
pub fn declared_workspace_root_from(path: &Path) -> Result<Option<PathBuf>, DiscoveryError> {
    let start = discovery_start(path)?;

    for candidate in start.ancestors() {
        let manifest_path = candidate.join(WORKSPACE_MANIFEST_RELATIVE);
        if !project_file_exists(&manifest_path)? {
            continue;
        }

        let manifest =
            fs::read_to_string(&manifest_path).map_err(|source| DiscoveryError::ReadManifest {
                path: manifest_path.clone(),
                source,
            })?;
        if manifest_declares_workspace(&manifest).map_err(|source| {
            DiscoveryError::ParseManifest {
                path: manifest_path,
                source: Box::new(source),
            }
        })? {
            return Ok(Some(candidate.to_path_buf()));
        }
    }

    Ok(None)
}

fn manifest_declares_workspace(source: &str) -> Result<bool, toml::de::Error> {
    Ok(toml::from_str::<toml::Value>(source)?
        .get("workspace")
        .is_some())
}

fn discovery_start(path: &Path) -> Result<PathBuf, DiscoveryError> {
    let metadata = fs::metadata(path).map_err(|source| DiscoveryError::Inspect {
        path: path.to_path_buf(),
        source,
    })?;
    let canonical = path
        .canonicalize()
        .map_err(|source| DiscoveryError::Canonicalize {
            path: path.to_path_buf(),
            source,
        })?;
    if metadata.is_dir() {
        return Ok(canonical);
    }
    if metadata.is_file() {
        return canonical.parent().map(Path::to_path_buf).ok_or_else(|| {
            DiscoveryError::UnsupportedPath {
                path: path.to_path_buf(),
            }
        });
    }
    Err(DiscoveryError::UnsupportedPath {
        path: path.to_path_buf(),
    })
}

fn project_file_exists(path: &Path) -> Result<bool, DiscoveryError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(DiscoveryError::Inspect {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if metadata.file_type().is_symlink() {
        return match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => Ok(true),
            Ok(_) => Err(DiscoveryError::ExpectedFile {
                path: path.to_path_buf(),
            }),
            Err(source) => Err(DiscoveryError::Inspect {
                path: path.to_path_buf(),
                source,
            }),
        };
    }
    if metadata.is_file() {
        Ok(true)
    } else {
        Err(DiscoveryError::ExpectedFile {
            path: path.to_path_buf(),
        })
    }
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inspect { path, source }
            | Self::ReadManifest { path, source }
            | Self::Canonicalize { path, source } => write!(
                f,
                "workspace discovery failed at {}: {source}",
                path.display()
            ),
            Self::ParseManifest { path, source } => {
                write!(f, "invalid Cargo manifest {}: {source}", path.display())
            }
            Self::UnsupportedPath { path } | Self::ExpectedFile { path } => {
                write!(f, "unsupported workspace discovery path {}", path.display())
            }
        }
    }
}
impl std::error::Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Inspect { source, .. }
            | Self::ReadManifest { source, .. }
            | Self::Canonicalize { source, .. } => Some(source),
            Self::ParseManifest { source, .. } => Some(source.as_ref()),
            Self::UnsupportedPath { .. } | Self::ExpectedFile { .. } => None,
        }
    }
}
#[cfg(test)]
mod tests;
