//! Guarded in-place exports beside caller-selected protected paths.

use crate::{identity::FileIdentity, path::canonicalize_allow_missing};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// A protected-path refusal or a failed stage of an in-place export.
///
/// Write failures may follow truncation or a partial write; Sync means all bytes
/// were written but synchronization failed. No error removes or rolls back output.
#[derive(Debug, thiserror::Error)]
pub enum NonAliasingWriteError {
    /// The output resolves to a protected pathname or shares its observed inode.
    #[error("output {output:?} aliases protected path {protected:?}")]
    Alias {
        /// Caller-selected output.
        output: PathBuf,
        /// Protected path whose contents must not be overwritten.
        protected: PathBuf,
    },
    /// The selected output has no parent component.
    #[error("export {path:?}: {source}")]
    InvalidTarget {
        /// Caller-selected output.
        path: PathBuf,
        /// Original invalid-input cause.
        source: io::Error,
    },
    /// Creating the selected parent directory failed.
    #[error("export {path:?}: {source}")]
    CreateDirectory {
        /// Parent selected for creation.
        path: PathBuf,
        /// Original filesystem cause.
        source: io::Error,
    },
    /// Path admission, opening, truncation or writing failed.
    #[error("export {path:?}: {source}")]
    Write {
        /// Caller-selected output.
        path: PathBuf,
        /// Original filesystem cause.
        source: io::Error,
    },
    /// All bytes were written but synchronizing the output failed.
    #[error("export {path:?}: {source}")]
    Sync {
        /// Caller-selected output.
        path: PathBuf,
        /// Original synchronization cause.
        source: io::Error,
    },
}

/// Refuse aliases of protected paths without creating or changing any entry.
///
/// Compare resolved names, including missing suffixes and symlinks, and device/
/// inode identity where both entries exist. Relative paths use the current
/// physical directory. The caller selects the missing-target symlink-expansion
/// depth ceiling; existing links retain platform canonicalization limits.
/// Permission and traversal failures retain their native causes.
///
/// This observation does not reserve paths or confine IO. Callers own namespace
/// custody and exclusion of concurrent changes to protected paths.
///
/// # Errors
/// Returns an alias refusal or a path/metadata failure.
pub fn validate_non_aliasing(
    output: &Path,
    protected: &[&Path],
    symlink_limit: usize,
) -> Result<(), NonAliasingWriteError> {
    let base = if output.is_relative() || protected.iter().any(|path| path.is_relative()) {
        fs::canonicalize(".").map_err(|source| write_error(output, source))?
    } else {
        PathBuf::new()
    };
    let resolved = canonicalize_allow_missing(output, &base, symlink_limit)
        .map_err(|source| write_error(output, source))?;
    let output_metadata =
        optional_metadata(output).map_err(|source| write_error(output, source))?;
    for &path in protected {
        let protected_resolved = canonicalize_allow_missing(path, &base, symlink_limit)
            .map_err(|source| write_error(output, source))?;
        let metadata = optional_metadata(path).map_err(|source| write_error(output, source))?;
        if resolved == protected_resolved
            || output_metadata
                .as_ref()
                .zip(metadata.as_ref())
                .is_some_and(|(left, right)| {
                    FileIdentity::from_metadata(left) == FileIdentity::from_metadata(right)
                })
        {
            return Err(alias_error(output, path));
        }
    }
    Ok(())
}

/// Write and synchronize an in-place export that must not overwrite protected files.
///
/// Admit aliases before parent creation, open without truncation, then recheck
/// both output pathname and retained descriptor before truncating/writing.
/// Missing parents are created with ordinary platform permissions; file creation
/// uses ordinary 0666 reduced by umask. Existing files retain their permissions.
/// Symlinks are intentionally followed when they do not alias a protected path.
///
/// This is not atomic replacement or root confinement. Parent links are not
/// synchronized, errors can leave visible partial output, and special files can
/// block. Callers own trusted path selection, namespace custody and concurrent
/// writers; the checks do not reserve protected identities after admission.
///
/// # Errors
/// Returns the original admission, parent creation, write or synchronization stage.
pub fn write_non_aliasing(
    output: &Path,
    bytes: &[u8],
    protected: &[&Path],
    symlink_limit: usize,
) -> Result<(), NonAliasingWriteError> {
    validate_non_aliasing(output, protected, symlink_limit)?;
    let parent = output
        .parent()
        .ok_or_else(|| NonAliasingWriteError::InvalidTarget {
            path: output.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "export target must have a parent",
            ),
        })?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    fs::create_dir_all(parent).map_err(|source| NonAliasingWriteError::CreateDirectory {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(output)
        .map_err(|source| write_error(output, source))?;
    validate_non_aliasing(output, protected, symlink_limit)?;
    write_opened(&mut file, output, bytes, protected, fs::File::sync_all)
}

fn write_opened(
    file: &mut fs::File,
    output: &Path,
    bytes: &[u8],
    protected: &[&Path],
    sync: impl FnOnce(&fs::File) -> io::Result<()>,
) -> Result<(), NonAliasingWriteError> {
    let metadata = file
        .metadata()
        .map_err(|source| write_error(output, source))?;
    for &path in protected {
        if optional_metadata(path)
            .map_err(|source| write_error(output, source))?
            .is_some_and(|protected| {
                FileIdentity::from_metadata(&metadata) == FileIdentity::from_metadata(&protected)
            })
        {
            return Err(alias_error(output, path));
        }
    }
    file.set_len(0)
        .map_err(|source| write_error(output, source))?;
    file.write_all(bytes)
        .map_err(|source| write_error(output, source))?;
    sync(file).map_err(|source| NonAliasingWriteError::Sync {
        path: output.to_path_buf(),
        source,
    })
}

fn optional_metadata(path: &Path) -> io::Result<Option<fs::Metadata>> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn alias_error(output: &Path, protected: &Path) -> NonAliasingWriteError {
    NonAliasingWriteError::Alias {
        output: output.to_path_buf(),
        protected: protected.to_path_buf(),
    }
}

fn write_error(path: &Path, source: io::Error) -> NonAliasingWriteError {
    NonAliasingWriteError::Write {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
