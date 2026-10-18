//! Selected native executables from Cargo's own package and artifact messages.

use super::{
    messages::{MessageDecoder, artifact_matches},
    observation::schema,
    selection::CargoSelection,
};
use ic_host_process::{
    child::OwnedChild,
    tool::{
        OutputLimit, OutputLimits, OutputStream, SuccessfulExit, communicate_child_with_observer,
    },
};
use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

#[cfg(test)]
mod tests;

/// Native executable target selected structurally before any Cargo command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeTargetKind {
    /// A package binary selected with `--bin`.
    Binary,
    /// A package example selected with `--example`.
    Example,
}

impl NativeTargetKind {
    const fn argument(self) -> &'static str {
        match self {
            Self::Binary => "--bin",
            Self::Example => "--example",
        }
    }

    const fn cargo_kind(self) -> schema::TargetKind {
        match self {
            Self::Binary => schema::TargetKind::Bin,
            Self::Example => schema::TargetKind::Example,
        }
    }
}

/// Exact native target and caller-selected recipe and decoding allowance.
#[derive(Clone, Debug)]
pub struct NativeExecutableSpec<'a> {
    /// Unique package name in the selected Cargo metadata graph.
    pub package: &'a str,
    /// Cargo target name, without guessing an output filename.
    pub target_name: &'a str,
    /// Binary or example, selecting Cargo's corresponding build target.
    pub target_kind: NativeTargetKind,
    /// Profile, feature and network/lock selections shared by metadata and build.
    /// Commands additionally require locked operation.
    pub selection: CargoSelection,
    /// Maximum metadata bytes or bytes in one Cargo JSON message, including LF.
    /// Total streamed diagnostic output and build duration have no default limit.
    pub max_message_bytes: usize,
}

/// Cargo-reported paths; mutable build-directory custody remains caller-owned.
#[derive(Debug)]
pub struct NativeExecutable {
    /// Absolute executable reported for the exact selected package and target.
    pub executable: PathBuf,
    /// Absolute target root reported by the same Cargo command's metadata.
    pub target_directory: PathBuf,
}

/// Build a selected native target and return Cargo's reported executable.
///
/// The factory supplies the same Cargo prefix, working directory, environment
/// and network/tool policy for metadata and build. Commands add `--locked`, exact
/// package/target selection and Cargo's JSON diagnostics format. Stdout is a
/// bounded-per-message protocol; rendered compiler diagnostics and piped stderr
/// stream to `diagnostics`. Other stderr configurations remain intact. Stdin is
/// null. `spawn` selects direct-child or owned-group custody and cleanup policy.
/// `admit` runs before each command factory. `interrupted` is polled during both
/// metadata and build capture; cancellation retains Host's typed cleanup evidence.
/// The caller owns signal handlers and cancellation meaning.
///
/// Callers exclude source/config/tool mutation and retain artifact custody. No
/// cache, installation, release admission, retry or elapsed-time default is added.
/// Interrupted or rejected capture uses the caller-selected child cleanup.
///
/// # Errors
/// Refuses invalid selection before command effects, missing/ambiguous metadata
/// or artifacts, malformed/oversized messages, sink failures and command failures.
/// Original Host execution/status/cleanup causes remain downcastable.
pub fn build_native_executable(
    spec: &NativeExecutableSpec<'_>,
    mut cargo: impl FnMut() -> Command,
    mut diagnostics: impl Write,
    mut admit: impl FnMut() -> io::Result<()>,
    mut spawn: impl FnMut(&mut Command) -> io::Result<OwnedChild>,
    mut interrupted: impl FnMut() -> bool,
) -> Result<NativeExecutable, Box<dyn Error + Send + Sync>> {
    if spec.package.is_empty() || spec.target_name.is_empty() || spec.max_message_bytes == 0 {
        return Err(protocol("empty native target selection or message allowance").into());
    }
    spec.selection.profile_target_dir().map_err(protocol)?;
    let arguments = CargoSelection {
        locked: true,
        ..spec.selection.clone()
    };
    admit()?;
    let mut command = cargo();
    command.args(["metadata", "--no-deps", "--format-version", "1"]);
    command.args(arguments.metadata_arguments());
    let mut bytes = Vec::new();
    execute(
        &mut command,
        &mut spawn,
        &mut interrupted,
        |stream, chunk| match stream {
            OutputStream::Stdout => append(&mut bytes, chunk, spec.max_message_bytes),
            OutputStream::Stderr => diagnostics.write_all(chunk),
        },
    )?;
    let (package_id, target_directory) = selection(serde_json::from_slice(&bytes)?, spec)?;
    admit()?;
    let mut command = cargo();
    command.args([
        "build",
        "--message-format=json",
        "-p",
        spec.package,
        spec.target_kind.argument(),
        spec.target_name,
    ]);
    command.args(arguments.build_arguments());
    let mut decoder = MessageDecoder::new(spec.max_message_bytes, false);
    let mut executable = None;
    execute(
        &mut command,
        &mut spawn,
        &mut interrupted,
        |stream, chunk| {
            if stream == OutputStream::Stderr {
                return diagnostics.write_all(chunk);
            }
            decoder.feed(chunk, |decoded| {
                selected_message(
                    decoded,
                    &package_id,
                    spec,
                    &mut executable,
                    &mut diagnostics,
                )
            })
        },
    )?;
    decoder.finish(|decoded| {
        selected_message(
            decoded,
            &package_id,
            spec,
            &mut executable,
            &mut diagnostics,
        )
    })?;
    Ok(NativeExecutable {
        executable: executable.ok_or_else(|| protocol("Cargo omitted the selected executable"))?,
        target_directory,
    })
}

fn selection(
    metadata: schema::Metadata,
    spec: &NativeExecutableSpec<'_>,
) -> io::Result<(String, PathBuf)> {
    let mut matching = metadata
        .packages
        .into_iter()
        .filter(|package| package.name.as_str() == spec.package);
    let package = matching
        .next()
        .ok_or_else(|| protocol("Cargo omitted selected package"))?;
    if matching.next().is_some() {
        return Err(protocol("Cargo package name is ambiguous"));
    }
    if package
        .targets
        .iter()
        .filter(|target| matches_target(target, spec))
        .count()
        != 1
    {
        return Err(protocol("Cargo target is missing or ambiguous"));
    }
    if package.id.repr.is_empty() {
        return Err(protocol("Cargo omitted package identity"));
    }
    if !metadata.target_directory.is_absolute() {
        return Err(protocol("Cargo omitted absolute target directory"));
    }
    Ok((
        package.id.repr,
        metadata.target_directory.into_std_path_buf(),
    ))
}

fn matches_target(target: &schema::Target, spec: &NativeExecutableSpec<'_>) -> bool {
    target.name == spec.target_name && target.kind.contains(&spec.target_kind.cargo_kind())
}

fn selected_message(
    message: schema::Message,
    package_id: &str,
    spec: &NativeExecutableSpec<'_>,
    executable: &mut Option<PathBuf>,
    diagnostics: &mut impl Write,
) -> io::Result<()> {
    match message {
        schema::Message::CompilerMessage(message) => {
            if let Some(rendered) = message.message.rendered {
                diagnostics.write_all(rendered.as_bytes())?;
            }
        }
        schema::Message::CompilerArtifact(artifact)
            if artifact_matches(
                &artifact,
                package_id,
                spec.target_name,
                &spec.target_kind.cargo_kind(),
            ) =>
        {
            let path = artifact
                .executable
                .filter(|path| path.is_absolute())
                .ok_or_else(|| protocol("Cargo omitted absolute executable path"))?
                .into_std_path_buf();
            if executable.replace(path).is_some() {
                return Err(protocol("Cargo reported multiple selected executables"));
            }
        }
        _ => {}
    }
    Ok(())
}

fn execute(
    command: &mut Command,
    spawn: &mut impl FnMut(&mut Command) -> io::Result<OwnedChild>,
    interrupted: &mut impl FnMut() -> bool,
    output: impl FnMut(OutputStream, &[u8]) -> io::Result<()>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    command.stdin(Stdio::null()).stdout(Stdio::piped());
    let mut child = spawn(command)?;
    communicate_child_with_observer(
        &mut child,
        None,
        OutputLimits {
            stdout: OutputLimit::Truncate(0),
            stderr: OutputLimit::Truncate(0),
            timeout: None,
        },
        SuccessfulExit::Cleanup,
        interrupted,
        output,
    )?
    .require_success()?;
    Ok(())
}

fn append(buffer: &mut Vec<u8>, bytes: &[u8], maximum: usize) -> io::Result<()> {
    if bytes.len() > maximum.saturating_sub(buffer.len()) {
        return Err(protocol("Cargo JSON message exceeds caller allowance"));
    }
    buffer.try_reserve(bytes.len()).map_err(io::Error::other)?;
    buffer.extend_from_slice(bytes);
    Ok(())
}

fn protocol(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
