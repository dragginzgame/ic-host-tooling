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

/// Check that every existing path component is a physical directory.
///
/// Missing components are permitted, but no directories are created. Symlinks
/// (including dangling links), non-directories and parent traversal (`..`) reject.
/// Relative paths use the current working directory; callers own empty-path,
/// pathname-content and absolute-path policy. This is a point-in-time check,
/// not root confinement or a held directory capability. Callers retain exclusion
/// of concurrent namespace mutation through subsequent filesystem operations.
///
/// # Errors
/// Returns [`io::ErrorKind::InvalidInput`] for parent traversal,
/// [`io::ErrorKind::NotADirectory`] for symlinks/non-directories, or the original
/// filesystem error other than a permitted not-found result.
pub fn validate_physical_directory_allow_missing(path: &Path) -> io::Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        if component == Component::ParentDir {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "physical directory path may not contain '..'",
            ));
        }
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotADirectory,
                    format!(
                        "path component is not a physical directory: {}",
                        current.display()
                    ),
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
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
/// `max_symlink_depth` bounds simultaneously active dangling-target expansions,
/// including targets reached after missing-suffix normalization. Completed
/// expansions do not charge later independent links. Zero rejects any such
/// expansion but still permits ordinary missing paths. Existing-target resolution
/// retains the native filesystem's symlink limits and errors. This does not
/// bound path length, filesystem latency or all filesystem operations.
///
/// Select `usize::MAX` explicitly when no additional missing-target depth bound
/// is needed. Opening, confinement and the chosen depth allowance remain caller-owned.
///
/// # Errors
/// Returns path/base errors or [`io::ErrorKind::InvalidInput`] on a missing-target
/// cycle or exhausted depth allowance. No path is returned on failure.
pub fn canonicalize_allow_missing(
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
