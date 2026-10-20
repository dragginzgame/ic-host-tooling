//! Shared transition from an unretained exact entry to invocation-owned construction.

use super::{WasmBuildError, create_dir_all, remove_unretained_entry, wasm_cache_fs_error};
use ic_host_fs::owned_directory::OwnedDirectory;
use std::path::Path;

pub(super) fn prepare_exact_cache_entry(path: &Path) -> Result<OwnedDirectory, WasmBuildError> {
    remove_unretained_entry(path).map_err(wasm_cache_fs_error)?;
    create_dir_all(path, "create content-addressed Cargo target directory")?;
    Ok(OwnedDirectory::new(path.to_owned()))
}
