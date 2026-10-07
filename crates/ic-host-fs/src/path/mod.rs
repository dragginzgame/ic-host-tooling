//! Resolution of caller-selected paths whose final components need not exist.

use std::{
    collections::HashSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

#[cfg(all(test, unix))]
mod tests;

enum ResolutionStep {
    Component(PathBuf),
    EndSymlink(PathBuf),
    RequireDirectory,
}

/// Resolve existing symlinks and normalize a missing suffix against an explicit base.
///
/// Relative paths require an absolute `base`; absolute paths ignore it. Dangling
/// symlinks resolve to their missing targets, including relative targets and
/// chains. Parent traversal out of a missing suffix resumes existing-component
/// resolution. Native traversal errors are retained; cycles encountered while
/// normalizing missing targets return [`io::ErrorKind::InvalidInput`].
/// Non-NotFound filesystem errors are preserved. This is trusted-path resolution,
/// not root confinement or immutable selection; ancestors and concurrent writers
/// remain caller-owned. No files are created or removed.
///
/// # Errors
/// Returns invalid relative bases or filesystem resolution failures.
pub fn canonicalize_allow_missing(path: &Path, base: &Path) -> io::Result<PathBuf> {
    canonicalize_allow_missing_with_symlink_limit(path, base, usize::MAX)
}

/// Resolve the same paths with a caller-selected missing-target symlink depth bound.
///
/// `max_symlink_depth` bounds simultaneously active dangling-target expansions,
/// including targets reached after missing-suffix normalization. Completed
/// expansions do not charge later independent links. Zero rejects any such
/// expansion but still permits ordinary missing paths. Existing-target resolution
/// retains the native filesystem's symlink limits and errors. This does not
/// bound path length, filesystem latency or all filesystem operations.
///
/// Opening, confinement and identity guarantees are the same as
/// [`canonicalize_allow_missing`]; the caller owns the chosen depth allowance.
///
/// # Errors
/// Returns path/base errors or [`io::ErrorKind::InvalidInput`] on a missing-target
/// cycle or exhausted depth allowance. No path is returned on failure.
pub fn canonicalize_allow_missing_with_symlink_limit(
    path: &Path,
    base: &Path,
    max_symlink_depth: usize,
) -> io::Result<PathBuf> {
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
    // Preserve native traversal checks (including file/., file/ and file/..)
    // before Path::components removes separators and current-directory parts.
    match absolute.canonicalize() {
        Ok(canonical) => return Ok(canonical),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let mut resolved = PathBuf::new();
    let mut missing_depth = 0_usize;
    let mut pending = Vec::new();
    let mut active_symlinks = HashSet::new();
    push_steps(&absolute, &mut pending);
    while let Some(step) = pending.pop() {
        let path = match step {
            ResolutionStep::Component(path) => path,
            ResolutionStep::EndSymlink(path) => {
                active_symlinks.remove(&path);
                continue;
            }
            ResolutionStep::RequireDirectory => {
                if missing_depth == 0 {
                    resolved = resolved.join(".").canonicalize()?;
                }
                continue;
            }
        };
        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                    let candidate = resolved.join(component.as_os_str());
                    if missing_depth == 0 && matches!(component, Component::Normal(_)) {
                        match candidate.canonicalize() {
                            Ok(canonical) => resolved = canonical,
                            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                                match fs::symlink_metadata(&candidate) {
                                    Ok(metadata) if metadata.is_symlink() => {
                                        if active_symlinks.len() >= max_symlink_depth
                                            || !active_symlinks.insert(candidate.clone())
                                        {
                                            return Err(io::Error::new(
                                                io::ErrorKind::InvalidInput,
                                                "missing-target symlink cycle or depth limit",
                                            ));
                                        }
                                        let target = fs::read_link(&candidate)?;
                                        pending.push(ResolutionStep::EndSymlink(candidate));
                                        // Relative targets start at the current resolved
                                        // parent. Absolute targets carry a root component.
                                        push_steps(&target, &mut pending);
                                    }
                                    Err(error) if error.kind() != io::ErrorKind::NotFound => {
                                        return Err(error);
                                    }
                                    _ => {
                                        resolved = candidate;
                                        missing_depth = 1;
                                    }
                                }
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
                    if missing_depth == 0 {
                        // After leaving a missing suffix, parent traversal must
                        // still reject an existing regular file used as a directory.
                        resolved = resolved.join(component.as_os_str()).canonicalize()?;
                    } else {
                        resolved.pop();
                        missing_depth -= 1;
                    }
                }
            }
        }
    }
    Ok(resolved)
}

fn push_steps(path: &Path, pending: &mut Vec<ResolutionStep>) {
    // components() drops trailing slashes and '/.'. Preserve their requirement
    // when missing-prefix normalization later reaches an existing entry.
    let bytes = path.as_os_str().as_encoded_bytes();
    let separator = std::path::MAIN_SEPARATOR as u8;
    if bytes.ends_with(b"/")
        || bytes.ends_with(b"/.")
        || bytes.ends_with(&[separator])
        || bytes.ends_with(&[separator, b'.'])
    {
        pending.push(ResolutionStep::RequireDirectory);
    }
    pending.extend(
        path.components()
            .rev()
            .map(|component| ResolutionStep::Component(PathBuf::from(component.as_os_str()))),
    );
}
