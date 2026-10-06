//! Read-only executable selection from caller-supplied paths and search order.

#[cfg(test)]
mod tests;

use std::{
    fmt, fs, io,
    os::unix::{ffi::OsStrExt as _, fs::PermissionsExt as _},
    path::{Path, PathBuf},
};

/// Executable selection failed before any admission or execution.
#[derive(Debug)]
pub enum ResolutionError {
    /// The request is empty, contains NUL, or is a bare `.` or `..`.
    InvalidRequest,
    /// The working directory is relative or contains NUL.
    InvalidWorkingDirectory,
    /// A caller-supplied search directory contains NUL.
    InvalidSearchDirectory {
        /// Zero-based position in the caller's search order.
        index: usize,
    },
    /// No regular file with Unix executable permission bits was found.
    NotFound,
    /// An explicitly requested path is not a regular file.
    NotRegularFile,
    /// An explicitly requested file has no Unix executable permission bits.
    NotExecutable,
    /// Metadata or canonicalization failed. Search stops on errors other than
    /// missing candidates rather than silently selecting a later directory.
    Io {
        /// Search-directory position, or `None` for an explicitly requested path.
        directory: Option<usize>,
        /// Underlying filesystem failure, without rendered input paths.
        source: io::Error,
    },
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest => f.write_str("invalid executable request"),
            Self::InvalidWorkingDirectory => f.write_str(
                "executable resolution requires an absolute working directory without NUL",
            ),
            Self::InvalidSearchDirectory { index } => {
                write!(f, "executable search directory {index} contains NUL")
            }
            Self::NotFound => {
                f.write_str("executable was not found in the supplied search directories")
            }
            Self::NotRegularFile => f.write_str("requested executable is not a regular file"),
            Self::NotExecutable => f.write_str("requested file has no executable permission bits"),
            Self::Io {
                directory: Some(index),
                ..
            } => write!(
                f,
                "executable filesystem resolution failed in search directory {index}"
            ),
            Self::Io {
                directory: None, ..
            } => f.write_str("requested executable filesystem resolution failed"),
        }
    }
}

impl std::error::Error for ResolutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Resolve one executable candidate without reading ambient PATH or running it.
///
/// Requests containing `/` are literal paths, including `./tool`; absolute paths
/// are used directly and relative paths are rooted at `current_dir`. Other names
/// search only `search_directories`, in order. Relative search directories and
/// explicitly supplied empty entries are rooted at `current_dir`; an empty list
/// searches nothing. The working directory must be absolute, even if unused.
/// No HOME/default installation directory, shell expansion or fallback is added.
///
/// Selection requires a regular file with at least one Unix execute permission
/// bit. Search skips missing candidates (including dangling symlinks), directories
/// and nonexecutable files, and stops on other filesystem errors.
/// Once an eligible candidate is observed, canonicalization errors stop selection,
/// including if that candidate disappears. All search
/// directories are checked for NUL before a name search; literal requests ignore
/// the unused search list. Symlinks are followed and the result is canonical and
/// absolute. Permission bits do not prove effective user access or interpreter
/// validity. Caller-owned trusted path trees and exclusion of concurrent writers
/// remain necessary; this path is not a frozen file capability.
///
/// A selected path must still undergo [`super::AdmittedTool::admit`] with the
/// consumer's digest, byte bound and exact version authority before execution.
/// No candidate is retried or replaced after that admission fails.
///
/// # Errors
/// Returns typed invalid-input, missing/nonexecutable or filesystem failures.
pub fn resolve_executable(
    requested: &Path,
    current_dir: &Path,
    search_directories: &[PathBuf],
) -> Result<PathBuf, ResolutionError> {
    if !current_dir.is_absolute() || contains_nul(current_dir) {
        return Err(ResolutionError::InvalidWorkingDirectory);
    }
    if requested.as_os_str().is_empty() || contains_nul(requested) {
        return Err(ResolutionError::InvalidRequest);
    }
    // Inspect literal bytes: Path::components normalizes away `./`, which must
    // not turn an explicit relative path into a search-directory request.
    if requested.as_os_str().as_bytes().contains(&b'/') {
        return candidate(&current_dir.join(requested), None);
    }
    if requested == Path::new(".") || requested == Path::new("..") {
        return Err(ResolutionError::InvalidRequest);
    }
    for (index, directory) in search_directories.iter().enumerate() {
        if contains_nul(directory) {
            return Err(ResolutionError::InvalidSearchDirectory { index });
        }
    }
    for (index, directory) in search_directories.iter().enumerate() {
        let path = current_dir.join(directory).join(requested);
        match candidate(&path, Some(index)) {
            Ok(path) => return Ok(path),
            Err(
                ResolutionError::NotFound
                | ResolutionError::NotRegularFile
                | ResolutionError::NotExecutable,
            ) => {}
            Err(error) => return Err(error),
        }
    }
    Err(ResolutionError::NotFound)
}

fn contains_nul(path: &Path) -> bool {
    path.as_os_str().as_bytes().contains(&0)
}

fn candidate(path: &Path, directory: Option<usize>) -> Result<PathBuf, ResolutionError> {
    let metadata = fs::metadata(path).map_err(|source| {
        // Only an absent metadata candidate may advance a search. A later
        // canonicalization failure must not silently select another executable.
        if directory.is_some() && source.kind() == io::ErrorKind::NotFound {
            ResolutionError::NotFound
        } else {
            ResolutionError::Io { directory, source }
        }
    })?;
    if !metadata.is_file() {
        return Err(ResolutionError::NotRegularFile);
    }
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(ResolutionError::NotExecutable);
    }
    fs::canonicalize(path).map_err(|source| ResolutionError::Io { directory, source })
}
