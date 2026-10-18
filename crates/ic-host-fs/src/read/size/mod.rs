//! Metadata-only logical lengths; callers retain traversal and cache policy.

use std::{fs, io, path::Path};

/// Sum non-directory metadata lengths without following final symlinks.
///
/// Directory metadata contributes no bytes. Sparse files contribute their logical
/// length; symlinks (including dangling links) contribute their own metadata
/// length. Hard-linked paths are counted separately. Totals saturate at `u64::MAX`.
/// The iterative walk reads metadata and directory entries, never file contents.
/// No entry/depth budget or cleanup is selected; callers admit trees and retain
/// namespace custody. Intermediate path components can resolve through aliases.
/// This is an observation, not hostile-tree confinement or a snapshot.
///
/// # Errors
/// Returns the original metadata, directory-opening or entry-reading I/O error.
pub fn directory_logical_size(path: &Path) -> io::Result<u64> {
    let mut total = 0_u64;
    let mut pending = vec![path.to_owned()];
    while let Some(current) = pending.pop() {
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.is_dir() {
            for entry in fs::read_dir(&current)? {
                let path = entry?.path();
                let metadata = fs::symlink_metadata(&path)?;
                if metadata.is_dir() {
                    pending.push(path);
                } else {
                    total = total.saturating_add(metadata.len());
                }
            }
        } else {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests;
