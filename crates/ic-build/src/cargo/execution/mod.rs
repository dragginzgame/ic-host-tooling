//! Cargo process communication and exact artifact observations shared by build recipes.
//!
//! Callers select recipes, environment, source custody, process spawning,
//! budgets, cancellation, build-script interpretation and release admission.

use super::{messages::MessageDecoder, observation::schema};
use ic_host_process::{
    child::OwnedChild,
    tool::{
        CompletedExecution, OutputLimits, OutputStream, SuccessfulExit, ToolError,
        communicate_child_with_observer,
    },
};
use std::{
    collections::BTreeSet,
    io,
    path::{Path, PathBuf},
};

mod invocation;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use invocation::command_identity;
pub use invocation::{
    CargoExecution, CargoExecutionPolicy, CargoInvocation, CargoRecipe, execute_cargo,
};
pub(crate) use invocation::{CargoTool, TOOL_IDENTITY_BYTES};

/// Independent execution and Cargo protocol evidence from one drained child.
///
/// A failed Cargo exit remains primary even if its output is not valid JSON.
/// Successful execution requires checking `messages` before admitting artifacts.
/// Protocol or message-sink refusal stops decoding but still drains the child.
#[must_use = "check execution status and protocol evidence before admitting artifacts"]
pub(crate) struct CargoCommunication {
    /// Original Host execution result, including bounded output and cleanup evidence.
    pub execution: Result<CompletedExecution, ToolError>,
    /// Original decoder or message-sink refusal, if any.
    pub messages: io::Result<()>,
}

/// Drain a caller-spawned Cargo child and decode its stdout once.
///
/// Configure JSON Cargo messages and piped stdout/stderr before spawning. Text
/// diagnostics are accepted; consumers receive canonical Cargo message types.
/// All raw chunks reach `output`, even after a protocol refusal. An output-sink
/// failure retains Host's interruption/cleanup behavior. Limits and cancellation
/// are explicit; no elapsed-time deadline or global state is introduced.
/// Observer panic unwinds through the caller's owned child and cleanup policy.
/// The caller must check both execution status and protocol evidence.
pub(crate) fn communicate_cargo(
    child: &mut OwnedChild,
    limits: OutputLimits,
    message_bytes: usize,
    cancelled: impl FnMut() -> bool,
    mut output: impl FnMut(OutputStream, &[u8]) -> io::Result<()>,
    mut message: impl FnMut(schema::Message) -> io::Result<()>,
) -> CargoCommunication {
    let mut decoder = MessageDecoder::new(message_bytes, true);
    let mut protocol_error = None;
    let execution = communicate_child_with_observer(
        child,
        None,
        limits,
        SuccessfulExit::Cleanup,
        cancelled,
        |stream, bytes| {
            if stream == OutputStream::Stdout && protocol_error.is_none() {
                protocol_error = decoder.feed(bytes, &mut message).err();
            }
            output(stream, bytes)
        },
    );
    let messages = if execution
        .as_ref()
        .is_ok_and(|evidence| evidence.status.success())
    {
        protocol_error.map_or_else(|| decoder.finish(&mut message), Err)
    } else {
        protocol_error.map_or(Ok(()), Err)
    };
    CargoCommunication {
        execution,
        messages,
    }
}

/// Cargo's nonempty set of outputs cannot identify one exact cdylib Wasm.
#[derive(Debug, thiserror::Error)]
pub enum CargoArtifactError {
    /// Preserve the original manifest-resolution failure.
    #[error("Cargo artifact manifest cannot be resolved: {0}")]
    Io(#[from] io::Error),
    /// Preserve the admitted manifest and number of distinct reported outputs.
    #[error(
        "Cargo did not report exactly one cdylib Wasm for {manifest}; observed {count} outputs"
    )]
    Selection {
        /// Canonical selected manifest.
        manifest: PathBuf,
        /// Distinct reported Wasm outputs.
        count: usize,
    },
}

/// Select one reported cdylib Wasm for an exact admitted manifest.
///
/// No filename inference, stale-file fallback or artifact lifetime is provided.
/// Callers must retain/capture the selected bytes before another build can replace them.
///
/// # Errors
/// Returns original path-resolution failures or a missing/ambiguous selection.
pub fn select_cdylib_wasm(
    messages: &[schema::Message],
    manifest: &Path,
) -> Result<PathBuf, CargoArtifactError> {
    let manifest = manifest.canonicalize()?;
    let mut outputs = BTreeSet::new();
    for message in messages {
        let schema::Message::CompilerArtifact(artifact) = message else {
            continue;
        };
        if artifact
            .target
            .crate_types
            .contains(&schema::CrateType::CDyLib)
            && artifact.manifest_path.canonicalize()? == manifest
        {
            outputs.extend(wasm_artifact_paths(artifact).map(Path::to_path_buf));
        }
    }
    let count = outputs.len();
    match outputs.pop_first() {
        Some(path) if count == 1 => Ok(path),
        _ => Err(CargoArtifactError::Selection { manifest, count }),
    }
}

pub(crate) fn wasm_artifact_paths(artifact: &schema::Artifact) -> impl Iterator<Item = &Path> {
    artifact
        .filenames
        .iter()
        .filter(|path| path.extension() == Some("wasm"))
        .map(|path| path.as_std_path())
}
