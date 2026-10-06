//! Resolution of caller-selected paths whose final components need not exist.

use std::{
    io,
    path::{Component, Path, PathBuf},
};

#[cfg(all(test, unix))]
mod tests;

/// Resolve existing symlinks and normalize a missing suffix against an explicit base.
///
/// Relative paths require an absolute `base`; absolute paths ignore it. Parent
/// traversal out of a missing suffix resumes resolution of existing components.
/// Non-NotFound filesystem errors are preserved. This is trusted-path resolution,
/// not root confinement or immutable selection; ancestors and concurrent writers
/// remain caller-owned. No files are created or removed.
///
/// # Errors
/// Returns invalid relative bases or filesystem resolution failures.
pub fn canonicalize_allow_missing(path: &Path, base: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else if base.is_absolute() {
        base.join(path)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "relative path requires an absolute base",
        ));
    };
    let mut resolved = PathBuf::new();
    let mut missing_depth = 0_usize;
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                let candidate = resolved.join(component.as_os_str());
                if missing_depth == 0 && matches!(component, Component::Normal(_)) {
                    match candidate.canonicalize() {
                        Ok(canonical) => resolved = canonical,
                        Err(error) if error.kind() == io::ErrorKind::NotFound => {
                            resolved = candidate;
                            missing_depth = 1;
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    resolved = candidate;
                    if matches!(component, Component::Normal(_)) && missing_depth > 0 {
                        missing_depth += 1;
                    }
                }
            }
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
                missing_depth = missing_depth.saturating_sub(1);
            }
        }
    }
    Ok(resolved)
}
