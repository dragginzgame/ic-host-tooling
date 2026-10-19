//! Inspect explicit raw Wasm inputs without executing or transforming them.

use clap::Parser as _;
use ic_host_artifacts::{
    artifact::ArtifactError,
    wasm::{InspectionError, InspectionLimits},
};
use std::{fmt, io, io::Write, process::ExitCode};

mod args;
mod report;

// The executable owns failures across input and reporting; Clap owns parsing.
#[derive(Debug)]
enum Error {
    Input(ArtifactError),
    Inspection(InspectionError),
    Output(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(source) => write!(formatter, "input admission failed: {source}"),
            Self::Inspection(source) => write!(formatter, "Wasm inspection failed: {source}"),
            Self::Output(source) => write!(formatter, "report output failed: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(source) => Some(source),
            Self::Inspection(source) => Some(source),
            Self::Output(source) => Some(source),
        }
    }
}

fn main() -> ExitCode {
    let command = match args::Command::try_parse() {
        Ok(command) => command,
        Err(error) => {
            let refused = error.use_stderr();
            return if error.print().is_err() || refused {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    let limits = InspectionLimits {
        module_bytes: command.module_bytes,
        sections: command.sections,
        exports: command.exports,
        custom_sections: command.custom_sections,
    };
    let result = ic_host_fs::read::read_file(&command.path, limits.module_bytes)
        .map_err(Error::Input)
        .and_then(|bytes| {
            report::write(
                &bytes,
                limits,
                command.expected,
                command.details,
                &mut io::stdout().lock(),
            )
        });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{error}");
            ExitCode::FAILURE
        }
    }
}
