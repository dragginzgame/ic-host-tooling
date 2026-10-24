//! Derive generated roots from explicit nested Cargo workspaces, before hashing.

use super::super::{cache_fs::canonicalize_allow_missing, digest::InputPathError};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

pub(super) fn exclude_nested_workspace_targets<'a>(
    inputs: impl Iterator<Item = &'a Path>,
    declared_package_files: &[PathBuf],
    exclusions: &mut Vec<PathBuf>,
) -> io::Result<()> {
    let roots = inputs
        .map(|path| {
            path.canonicalize().map_err(|source| {
                io::Error::new(
                    source.kind(),
                    InputPathError {
                        path: path.to_owned(),
                        source,
                    },
                )
            })
        })
        .collect::<io::Result<BTreeSet<_>>>()?;
    let mut pending = roots.clone();
    let declared_package_files = declared_package_files
        .iter()
        .map(|path| canonicalize_allow_missing(path))
        .collect::<io::Result<Vec<_>>>()?;
    let mut visited = BTreeSet::new();
    let mut excluded = exclusions
        .iter()
        .map(|path| canonicalize_allow_missing(path))
        .collect::<io::Result<Vec<_>>>()?;
    while let Some(path) = pending.pop_first() {
        let mut discover = || -> io::Result<()> {
            if excluded.iter().any(|root| path.starts_with(root))
                || !fs::metadata(&path)?.is_dir()
                || !visited.insert(path.clone())
            {
                return Ok(());
            }
            // Immediate input-root targets retain their existing admission policy.
            // An arbitrary source directory named `target` is never a marker.
            if !roots.contains(&path) && declares_workspace(&path)? {
                let target = path.join("target");
                if generated_target(&target)?
                    && !roots.iter().any(|input| input.starts_with(&target))
                    && !declared_package_files
                        .iter()
                        .any(|source| source.starts_with(&target))
                {
                    excluded.push(target.clone());
                    exclusions.push(target);
                }
            }
            for entry in fs::read_dir(&path)? {
                let child = entry?.path().canonicalize()?;
                if !excluded.iter().any(|root| child.starts_with(root))
                    && fs::metadata(&child)?.is_dir()
                {
                    pending.insert(child);
                }
            }
            Ok(())
        };
        discover()
            .map_err(|source| io::Error::new(source.kind(), InputPathError { path, source }))?;
    }
    exclusions.sort();
    exclusions.dedup();
    Ok(())
}

fn declares_workspace(directory: &Path) -> io::Result<bool> {
    let manifest = directory.join("Cargo.toml");
    match fs::metadata(&manifest) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    }
    let source = match fs::read_to_string(manifest) {
        Ok(source) => source,
        Err(error) if error.kind() == io::ErrorKind::InvalidData => return Ok(false),
        Err(error) => return Err(error),
    };
    // Invalid TOML is ordinary watched data, never authority for an exclusion.
    Ok(toml::from_str::<toml::Value>(&source)
        .is_ok_and(|manifest| manifest.get("workspace").is_some_and(toml::Value::is_table)))
}

fn generated_target(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        // A link is not authority to exclude its destination. Keep the input
        // walk's existing follow-link/error policy for it instead.
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error),
    }
}
