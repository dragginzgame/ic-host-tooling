//! Inspect explicit raw Wasm inputs without executing or transforming them.

use std::{env, fmt, io, io::Write, num::ParseIntError, process::ExitCode};

use ic_host_artifacts::{artifact::ArtifactError, wasm::InspectionError};

mod args;
mod report;

// The executable owns failures across argument parsing, input and reporting.
#[derive(Debug)]
enum Error {
    MissingArgument(&'static str),
    ExtraArgument,
    NonUtf8Number(&'static str),
    InvalidNumber {
        name: &'static str,
        source: ParseIntError,
    },
    Input(ArtifactError),
    Inspection(InspectionError),
    Output(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingArgument(name) => {
                write!(formatter, "missing {name}; {}", args::USAGE)
            }
            Self::ExtraArgument => write!(formatter, "unexpected argument; {}", args::USAGE),
            Self::NonUtf8Number(name) => {
                write!(formatter, "{name} must be a UTF-8 unsigned integer")
            }
            Self::InvalidNumber { name, source } => write!(formatter, "invalid {name}: {source}"),
            Self::Input(source) => write!(formatter, "input admission failed: {source}"),
            Self::Inspection(source) => write!(formatter, "Wasm inspection failed: {source}"),
            Self::Output(source) => write!(formatter, "report output failed: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidNumber { source, .. } => Some(source),
            Self::Input(source) => Some(source),
            Self::Inspection(source) => Some(source),
            Self::Output(source) => Some(source),
            Self::MissingArgument(_) | Self::ExtraArgument | Self::NonUtf8Number(_) => None,
        }
    }
}

fn main() -> ExitCode {
    let result = args::parse(env::args_os().skip(1)).and_then(|command| match command {
        args::Command::Help => {
            writeln!(io::stdout().lock(), "{}", args::USAGE).map_err(Error::Output)
        }
        args::Command::Inspect { path, limits } => {
            let bytes =
                ic_host_fs::read::read_file(&path, limits.module_bytes).map_err(Error::Input)?;
            report::write(&bytes, limits, &mut io::stdout().lock())
        }
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{error}");
            ExitCode::FAILURE
        }
    }
}
