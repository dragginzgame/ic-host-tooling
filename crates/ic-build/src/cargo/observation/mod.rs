//! Cargo catalog, resolved-graph and package-tree command rendering and decoding.
//!
//! Callers own the selected command/environment, graph policy and output budgets.
//! Cargo's canonical Rust schema owns metadata decoding; consumers interpret it.
//! No observation creates a cache or grants build/release admission.

#[cfg(test)]
mod tests;

mod workspace;
pub use workspace::{decode_workspace_root, workspace_root_command};

/// Canonical Cargo metadata and build-message types, without consumer mirrors.
pub use cargo_metadata as schema;

use super::selection::{CargoFeatureSelection, graph_arguments};
use std::{path::Path, process::Command};

/// Decode complete Cargo metadata JSON through Cargo's canonical schema.
///
/// The caller must first admit successful, complete output through its selected
/// process runner. This function neither executes Cargo nor admits partial output.
///
/// # Errors
/// Returns the canonical schema's JSON error for malformed or incomplete metadata.
pub fn decode_metadata(bytes: &[u8]) -> Result<schema::Metadata, serde_json::Error> {
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

impl CargoMetadataSpec<'_> {
    /// Render metadata arguments onto a caller-selected command without executing it.
    ///
    /// Preserves the executable, launcher arguments and environment; selects the
    /// manifest's parent as the working directory. Capture policy and admission of
    /// successful, complete output belong to the caller, before [`decode_metadata`].
    ///
    /// # Errors
    /// Returns an I/O error if a relative manifest cannot be anchored to the current directory.
    pub fn command(&self, mut command: Command) -> std::io::Result<Command> {
        command.args(["metadata", "--format-version=1"]);
        configure(
            &mut command,
            self.manifest_path,
            self.locked,
            self.offline,
            self.features,
        )?;
        if !self.include_dependencies {
            command.arg("--no-deps");
        }
        if let Some(target) = self.filter_platform {
            command.arg("--filter-platform").arg(target);
        }
        Ok(command)
    }
}

impl CargoTreeSpec<'_> {
    /// Render a depth-prefixed, non-deduplicated tree command without executing it.
    ///
    /// Preserves the selected executable, launcher arguments and environment, as
    /// [`CargoMetadataSpec::command`] does. Admit successful, complete output
    /// through the caller's runner before `String::from_utf8`; alias and role
    /// correlation remain caller-owned. This method adds no timeout.
    ///
    /// # Errors
    /// Returns `InvalidInput` for empty package selectors, or an I/O error if a
    /// relative manifest cannot be anchored. Cargo diagnoses invalid selectors,
    /// feature combinations and formatting options when the caller executes it.
    pub fn command(&self, mut command: Command) -> std::io::Result<Command> {
        if self.packages.is_empty() || self.packages.iter().any(|package| package.is_empty()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cargo tree requires nonempty package selectors",
            ));
        }
        command.args([
            "tree",
            "--color",
            "never",
            "--target",
            self.target,
            "--edges",
            self.edges,
            "--prefix",
            "depth",
            "--no-dedupe",
            "--charset",
            "ascii",
            "--format",
            self.format,
        ]);
        configure(
            &mut command,
            self.manifest_path,
            self.locked,
            self.offline,
            self.features,
        )?;
        for package in self.packages {
            command.arg("--package").arg(package);
        }
        Ok(command)
    }
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
