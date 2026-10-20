//! Filesystem facts scoped to one pass of pre-effect input boundary validation.

use crate::artifacts::cache_fs::{CacheFsError, canonicalize_allow_missing};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub(in crate::artifacts) struct CargoInputBoundaries {
    exclusions: BTreeMap<PathBuf, PathBuf>,
    inputs: BTreeMap<PathBuf, (PathBuf, bool)>,
}

impl CargoInputBoundaries {
    pub(in crate::artifacts) fn exclusion(&mut self, path: &Path) -> Result<&Path, CacheFsError> {
        if !self.exclusions.contains_key(path) {
            let canonical = canonicalize_allow_missing(path).map_err(|source| CacheFsError {
                operation: "resolve Cargo build input exclusion",
                path: path.to_owned(),
                source,
            })?;
            self.exclusions.insert(path.to_owned(), canonical);
        }
        Ok(&self.exclusions[path])
    }

    pub(in crate::artifacts) fn input(
        &mut self,
        path: &Path,
        resolve_operation: &'static str,
        inspect_operation: &'static str,
    ) -> Result<&(PathBuf, bool), CacheFsError> {
        if !self.inputs.contains_key(path) {
            let canonical = path.canonicalize().map_err(|source| CacheFsError {
                operation: resolve_operation,
                path: path.to_owned(),
                source,
            })?;
            let is_directory = fs::metadata(&canonical)
                .map_err(|source| CacheFsError {
                    operation: inspect_operation,
                    path: canonical.clone(),
                    source,
                })?
                .is_dir();
            self.inputs
                .insert(path.to_owned(), (canonical, is_directory));
        }
        Ok(&self.inputs[path])
    }
}
