//! ICP CLI command syntax and installation facts, without process execution.
//!
//! Callers select executables, credentials, environments, output formats and
//! capture limits. No command is dispatched and no tool version is admitted here.

mod status;
#[cfg(test)]
mod tests;

use std::{ffi::OsStr, path::Path, process::Command};

pub use status::{InstallationStatusError, decode_installation_status};

/// The already-selected query or update operation; this does not authorize it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallMode {
    /// Append ICP CLI's query switch.
    Query,
    /// Use ICP CLI's default update operation.
    Update,
}

/// The caller-owned Candid argument representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallArguments<'a> {
    /// Pass a textual Candid argument verbatim as one native argument.
    Text(&'a OsStr),
    /// Pass a file containing binary Candid; the caller owns its custody.
    BinaryFile(&'a Path),
}

/// The caller-selected status output contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusOutput {
    /// Leave ICP CLI's output selection unchanged.
    Text,
    /// Request structured JSON.
    Json,
    /// Request only the canister id.
    IdOnly,
}

/// Extend a caller-configured executable with an ICP canister call.
///
/// The base must not already contain the `canister` subcommand. Executable,
/// native arguments, environment, working directory and standard I/O are retained.
/// Callers append Candid interface, response format and target selection themselves.
#[must_use]
pub fn call_command(
    mut command: Command,
    canister: &OsStr,
    method: &OsStr,
    arguments: CallArguments<'_>,
    mode: CallMode,
) -> Command {
    command.args([OsStr::new("canister"), OsStr::new("call"), canister, method]);
    match arguments {
        CallArguments::Text(argument) => {
            command.arg(argument);
        }
        CallArguments::BinaryFile(path) => {
            command
                .arg("--args-file")
                .arg(path)
                .args(["--args-format", "bin"]);
        }
    }
    if mode == CallMode::Query {
        command.arg("--query");
    }
    command
}

/// Extend a caller-configured executable with an ICP canister status request.
///
/// The base must not already contain the `canister` subcommand. Target selection,
/// capture, execution, failure interpretation and installation policy stay local.
#[must_use]
pub fn status_command(mut command: Command, canister: &OsStr, output: StatusOutput) -> Command {
    command.args([OsStr::new("canister"), OsStr::new("status"), canister]);
    match output {
        StatusOutput::Text => {}
        StatusOutput::Json => {
            command.arg("--json");
        }
        StatusOutput::IdOnly => {
            command.arg("--id-only");
        }
    }
    command
}
