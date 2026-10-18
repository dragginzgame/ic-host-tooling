//! One bounded, ordered batch over the canonical stream and file-identity owners.

use ic_host_artifacts::artifact::read_reader;
use ic_host_fs::read::hash_file_no_follow;
use std::{
    error::Error,
    ffi::OsStr,
    io::{self, Write as _},
    os::unix::ffi::OsStrExt as _,
    path::Path,
};

/// Admit a caller-bounded path list and all files before emitting the batch.
pub fn run(maximum: usize, max_list_bytes: usize, max_files: usize) -> Result<(), Box<dyn Error>> {
    let input = read_reader(io::stdin().lock(), max_list_bytes)?;
    if input.is_empty() {
        return Ok(());
    }
    let paths = input
        .strip_suffix(&[0])
        .ok_or("file list must be NUL-terminated")?
        .split(|&byte| byte == 0);
    let count = paths.clone().count();
    if count > max_files || paths.clone().any(<[u8]>::is_empty) {
        return Err("file list exceeds MAX_FILES or contains an empty path".into());
    }
    // Every output record adds 64 digest bytes and one tab to its input record.
    let capacity = count
        .checked_mul(65)
        .and_then(|extra| input.len().checked_add(extra))
        .ok_or("hash output size overflow")?;
    let mut output = Vec::new();
    output.try_reserve_exact(capacity)?;
    for path in paths {
        let identity = hash_file_no_follow(Path::new(OsStr::from_bytes(path)), maximum as u64)?;
        output.extend_from_slice(identity.sha256.to_string().as_bytes());
        output.push(b'\t');
        output.extend_from_slice(path);
        output.push(0);
    }
    // A later list/file refusal must not emit a successful prefix. A stdout IO
    // failure can still leave a prefix; callers must admit exit status first.
    io::stdout().lock().write_all(&output)?;
    Ok(())
}
