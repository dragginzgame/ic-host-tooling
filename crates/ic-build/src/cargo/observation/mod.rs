//! Bounded Cargo catalog, resolved-graph and package-tree observations.
//!
//! Callers own the selected command/environment, graph policy and output budgets.
//! Cargo's canonical Rust schema owns metadata decoding; consumers interpret it.
//! No observation creates a cache or grants build/release admission.

#[cfg(test)]
mod tests;

mod workspace;
pub use workspace::workspace_root_for;

/// Canonical Cargo metadata and build-message types, without consumer mirrors.
pub use cargo_metadata as schema;

use super::selection::{CargoFeatureSelection, graph_arguments};
use ic_host_process::tool::{CompletedExecution, OutputLimits, capture_command};
use std::{path::Path, process::Command};

// Decode complete captured bytes once through Cargo's canonical schema.
pub(crate) fn decode_metadata(bytes: &[u8]) -> Result<schema::Metadata, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// One Cargo metadata observation; package/role interpretation stays with callers.
pub struct CargoMetadataSpec<'a> {
    /// Exact manifest selecting the workspace/graph.
    pub manifest_path: &'a Path,
    /// Include dependency packages and Cargo's resolve graph; otherwise use `--no-deps`.
    pub include_dependencies: bool,
    /// Optional target filter for the resolved graph, independent of catalog policy.
    pub filter_platform: Option<&'a str>,
    /// Require an unchanged existing lockfile.
    pub locked: bool,
    /// Prohibit Cargo network access.
    pub offline: bool,
    /// Features selected by the consuming recipe.
    pub features: &'a CargoFeatureSelection,
}

/// One package-selected Cargo tree, returned without imposing role graph policy.
pub struct CargoTreeSpec<'a> {
    /// Exact manifest selecting the workspace/graph.
    pub manifest_path: &'a Path,
    /// Cargo package ID specifications; Cargo diagnoses missing or ambiguous selectors.
    pub packages: &'a [&'a str],
    /// Target whose dependencies Cargo resolves.
    pub target: &'a str,
    /// Cargo edge selection, for example `normal` or `normal,build`.
    pub edges: &'a str,
    /// Caller-selected Cargo tree format, retaining aliases/features when requested.
    pub format: &'a str,
    /// Require an unchanged existing lockfile.
    pub locked: bool,
    /// Prohibit Cargo network access.
    pub offline: bool,
    /// Features selected by the consuming recipe.
    pub features: &'a CargoFeatureSelection,
}

/// Observe Cargo metadata through the caller's selected command and capture limits.
///
/// The command carries the executable, complete environment and any caller-bound
/// launcher arguments. This adds the metadata subcommand and recipe arguments,
/// selects the manifest's parent as the working directory and sets no deadline
/// beyond `limits.timeout`. Output must be complete before canonical decoding.
///
/// # Errors
/// Returns the original typed Host spawn/capture/exit failure (including bounded
/// diagnostics and truncation evidence), or the canonical schema's JSON error.
pub fn observe_metadata(
    spec: &CargoMetadataSpec<'_>,
    limits: OutputLimits,
    mut command: Command,
) -> Result<schema::Metadata, Box<dyn std::error::Error>> {
    command.args(["metadata", "--format-version=1"]);
    configure(
        &mut command,
        spec.manifest_path,
        spec.locked,
        spec.offline,
        spec.features,
    )?;
    if !spec.include_dependencies {
        command.arg("--no-deps");
    }
    if let Some(target) = spec.filter_platform {
        command.arg("--filter-platform").arg(target);
    }
    let output = capture(&mut command, limits)?;
    Ok(decode_metadata(&output.stdout)?)
}

/// Observe a depth-prefixed, non-deduplicated package-selected Cargo tree.
///
/// Capture, environment and deadline semantics match [`observe_metadata`]. The
/// text is complete UTF-8; package/alias correlation remains caller-owned.
///
/// # Errors
/// Returns typed Host spawn/capture/exit failures or invalid UTF-8. Cargo itself
/// rejects invalid selectors, feature combinations and tree formatting options.
pub fn observe_tree(
    spec: &CargoTreeSpec<'_>,
    limits: OutputLimits,
    mut command: Command,
) -> Result<String, Box<dyn std::error::Error>> {
    if spec.packages.is_empty() || spec.packages.iter().any(|package| package.is_empty()) {
        return Err("Cargo tree requires nonempty package selectors".into());
    }
    command.args([
        "tree",
        "--color",
        "never",
        "--target",
        spec.target,
        "--edges",
        spec.edges,
        "--prefix",
        "depth",
        "--no-dedupe",
        "--charset",
        "ascii",
        "--format",
        spec.format,
    ]);
    configure(
        &mut command,
        spec.manifest_path,
        spec.locked,
        spec.offline,
        spec.features,
    )?;
    for package in spec.packages {
        command.arg("--package").arg(package);
    }
    let output = capture(&mut command, limits)?;
    Ok(String::from_utf8(output.stdout)?)
}

fn configure(
    command: &mut Command,
    manifest: &Path,
    locked: bool,
    offline: bool,
    features: &CargoFeatureSelection,
) -> std::io::Result<()> {
    let manifest = if manifest.is_absolute() {
        manifest.to_path_buf()
    } else {
        std::env::current_dir()?.join(manifest)
    };
    command.current_dir(manifest.parent().unwrap_or_else(|| Path::new(".")));
    command.arg("--manifest-path").arg(manifest);
    command.args(graph_arguments(features, locked, offline));
    Ok(())
}

fn capture(
    command: &mut Command,
    limits: OutputLimits,
) -> Result<CompletedExecution, ic_host_process::tool::ToolError> {
    capture_command(command, limits)
        .and_then(CompletedExecution::require_success)
        .and_then(CompletedExecution::require_complete)
}
