//! Transaction policy over lazily observed Cargo input facts.

use super::{ArtifactCacheError, ResolvedCargoBuildInputs, artifact_cache_fs_error};
use crate::artifacts::inputs::boundaries::CargoInputBoundaries;
use std::path::Path;

impl CargoInputBoundaries {
    pub(super) fn watches(
        &mut self,
        resolved: &ResolvedCargoBuildInputs,
        candidate: &Path,
    ) -> Result<bool, ArtifactCacheError> {
        for exclusion in resolved.exclusions() {
            if candidate.starts_with(self.exclusion(exclusion).map_err(artifact_cache_fs_error)?) {
                return Ok(false);
            }
        }
        for input in resolved.inputs() {
            let (path, is_directory) = self
                .input(
                    input.path(),
                    "canonicalize resolved Cargo build input",
                    "inspect resolved Cargo build input",
                )
                .map_err(artifact_cache_fs_error)?;
            if candidate == *path || (*is_directory && candidate.starts_with(path)) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
