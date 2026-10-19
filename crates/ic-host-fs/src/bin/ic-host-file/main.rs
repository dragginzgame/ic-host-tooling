//! Explicit bounded file operations over the existing filesystem owners.

use clap::Parser as _;
use std::{error::Error, path::PathBuf, process::ExitCode};

#[cfg(unix)]
mod batch;
#[cfg(unix)]
mod unix;

#[derive(clap::Parser)]
#[command(
    name = "ic-host-file",
    version,
    about = "Bounded file hashing and private publication"
)]
struct Arguments {
    #[command(subcommand)]
    operation: Operation,
}

#[derive(clap::Subcommand)]
enum Operation {
    /// Hash one no-follow regular file.
    HashFile { path: PathBuf, max_bytes: usize },
    /// Hash a bounded NUL-separated path list from stdin.
    HashFiles0 {
        max_bytes: usize,
        max_list_bytes: usize,
        max_files: usize,
    },
    /// Create a private destination without replacing an existing entry.
    CreatePrivate {
        source: PathBuf,
        destination: PathBuf,
        max_bytes: usize,
    },
    /// Replace a destination with privately published bytes.
    ReplacePrivate {
        source: PathBuf,
        destination: PathBuf,
        max_bytes: usize,
    },
}

fn run(operation: Operation) -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    unix::run(operation)?;
    #[cfg(not(unix))]
    {
        let _ = operation;
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "ic-host-file requires Unix",
        )
        .into());
    }
    #[cfg(unix)]
    Ok(())
}

fn main() -> ExitCode {
    let arguments = match Arguments::try_parse() {
        Ok(arguments) => arguments,
        Err(error) => {
            let refused = error.use_stderr();
            return if error.print().is_err() || refused {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match run(arguments.operation) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("host file operation refused: {error}");
            ExitCode::FAILURE
        }
    }
}
