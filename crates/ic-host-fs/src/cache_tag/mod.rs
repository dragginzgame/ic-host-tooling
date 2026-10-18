//! Cache-directory tag recognition and durable replacement; callers select paths and text.

use crate::{
    durable::{NamedWriteError, PublicationMode, WriteOptions, write_with},
    read::open_regular_file,
};
use std::{
    fs,
    io::{self, Read as _, Write as _},
    path::Path,
};

/// Standard cache-directory signature, recognized without interpreting trailing bytes.
pub const SIGNATURE: &str = "Signature: 8a477f597d28d172789f06886806bc55";

/// Preserve a regular tag with the signature, or durably replace an invalid tag.
///
/// Requires a regular final tag entry and reads only the fixed-width signature.
/// Existing valid tags are not replaced; their contents and permissions remain.
/// Missing, invalid, unreadable
/// or symlink tags attempt replacement at `directory/CACHEDIR.TAG`; symlink
/// referents are untouched. New contents are the signature followed by the exact
/// caller-selected `suffix` (normally a newline and comments). `permissions`
/// selects the new file mode through the existing durable writer.
///
/// Callers retain stable parent namespace custody, suffix budgets and cache-root
/// selection. The durable pathname writer creates missing parents with its
/// ordinary directory semantics; this does not admit private roots or perform
/// cache maintenance.
///
/// # Errors
/// Returns the original durable-write failure with publication/cleanup evidence.
pub fn ensure(
    directory: &Path,
    suffix: &[u8],
    permissions: u32,
) -> Result<(), NamedWriteError<io::Error>> {
    let path = directory.join("CACHEDIR.TAG");
    let mut signature = [0_u8; SIGNATURE.len()];
    if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_file())
        && open_regular_file(&path)
            .and_then(|mut file| file.read_exact(&mut signature).map_err(Into::into))
            .is_ok()
        && signature == SIGNATURE.as_bytes()
    {
        return Ok(());
    }
    write_with(
        &path,
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions,
        },
        |file| {
            file.write_all(SIGNATURE.as_bytes())?;
            file.write_all(suffix)
        },
    )
}

#[cfg(test)]
mod tests;
