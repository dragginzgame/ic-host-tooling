//! Explicit bounded file operations over the existing filesystem owners.

use std::{
    env,
    error::Error,
    ffi::OsString,
    io::{self, Write as _},
    process::ExitCode,
};

#[cfg(unix)]
mod batch;
#[cfg(unix)]
mod unix;

const USAGE: &str = "usage: ic-host-file hash-file PATH MAX_BYTES | hash-files0 MAX_BYTES MAX_LIST_BYTES MAX_FILES | {create-private|replace-private} SOURCE DESTINATION MAX_BYTES | --version | --help";

fn run(arguments: &[OsString]) -> Result<(), Box<dyn Error>> {
    match arguments {
        [argument] if argument == "--help" => writeln!(io::stdout().lock(), "{USAGE}")?,
        [argument] if argument == "--version" => {
            writeln!(
                io::stdout().lock(),
                "ic-host-file {}",
                env!("CARGO_PKG_VERSION")
            )?;
        }
        _ => {
            #[cfg(unix)]
            unix::run(arguments)?;
            #[cfg(not(unix))]
            return Err(
                io::Error::new(io::ErrorKind::Unsupported, "ic-host-file requires Unix").into(),
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(&env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("host file operation refused: {error}");
            ExitCode::FAILURE
        }
    }
}
