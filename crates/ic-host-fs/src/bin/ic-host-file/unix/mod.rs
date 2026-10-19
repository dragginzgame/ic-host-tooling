//! Typed CLI translation; admission and publication remain library-owned.

use super::{Operation, batch};
use ic_host_fs::{
    durable::{PublicationMode, WriteOptions, write_with},
    read::{hash_file_no_follow, read_file_no_follow},
};
use std::{
    error::Error,
    io::{self, Write as _},
};

/// Translate an admitted command into the existing FS operations.
pub fn run(operation: Operation) -> Result<(), Box<dyn Error>> {
    let create_new = matches!(&operation, Operation::CreatePrivate { .. });
    match operation {
        Operation::HashFile { path, max_bytes } => {
            let identity = hash_file_no_follow(&path, max_bytes as u64)?;
            writeln!(io::stdout().lock(), "{}", identity.sha256)?;
        }
        Operation::HashFiles0 {
            max_bytes,
            max_list_bytes,
            max_files,
        } => batch::run(max_bytes, max_list_bytes, max_files)?,
        Operation::CreatePrivate {
            source,
            destination,
            max_bytes,
        }
        | Operation::ReplacePrivate {
            source,
            destination,
            max_bytes,
        } => {
            let mode = if create_new {
                PublicationMode::CreateNew
            } else {
                PublicationMode::Replace
            };
            let bytes = read_file_no_follow(&source, max_bytes)?;
            write_with(
                &destination,
                WriteOptions {
                    mode,
                    permissions: 0o600,
                },
                |file| file.write_all(&bytes),
            )?;
        }
    }
    Ok(())
}
