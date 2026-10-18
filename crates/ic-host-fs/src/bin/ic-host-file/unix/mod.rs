//! Native argument translation; admission and publication remain library-owned.

use super::{USAGE, batch};
use ic_host_fs::{
    durable::{PublicationMode, WriteOptions, write_with},
    read::{hash_file_no_follow, read_file_no_follow},
};
use std::{
    error::Error,
    ffi::{OsStr, OsString},
    io::{self, Write as _},
    path::Path,
};

fn number(argument: &OsStr) -> Result<usize, Box<dyn Error>> {
    Ok(argument
        .to_str()
        .ok_or("limits must be UTF-8 unsigned integers")?
        .parse()?)
}

/// Translate explicit native arguments into the existing FS operations.
pub fn run(arguments: &[OsString]) -> Result<(), Box<dyn Error>> {
    let operation = arguments
        .first()
        .and_then(|value| value.to_str())
        .ok_or(USAGE)?;
    match (operation, arguments.len()) {
        ("hash-file", 3) => {
            let maximum = number(&arguments[2])?;
            let identity = hash_file_no_follow(Path::new(&arguments[1]), maximum as u64)?;
            writeln!(io::stdout().lock(), "{}", identity.sha256)?;
        }
        ("hash-files0", 4) => batch::run(
            number(&arguments[1])?,
            number(&arguments[2])?,
            number(&arguments[3])?,
        )?,
        ("create-private" | "replace-private", 4) => {
            let maximum = number(&arguments[3])?;
            // Source admission precedes all destination effects, including parents.
            let bytes = read_file_no_follow(Path::new(&arguments[1]), maximum)?;
            let mode = if operation == "create-private" {
                PublicationMode::CreateNew
            } else {
                PublicationMode::Replace
            };
            write_with(
                Path::new(&arguments[2]),
                WriteOptions {
                    mode,
                    permissions: 0o600,
                },
                |file| file.write_all(&bytes),
            )?;
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}
