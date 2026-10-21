//! Retained Wasm acquisition and bounded evidence export for host consumers.
use super::{
    CapturedArtifact, ResolvedCargoBuildInputs, WasmBuildOutcome, WasmBuildSpec,
    build_wasm_canisters_cached, resolve_cargo_build_inputs,
};
use ic_host_process::child::OwnedChild;
use serde::Serialize;
use std::{fs, io, path::Path, process::Command};

/// Build and source identities retained through caller-owned measurement.
///
/// Callers must hold source custody throughout acquisition and consumption.
/// Freshness checks detect persistent changes, not edits reverted between checks.
/// Keep this owner alive through measurement, then call `verify_current` before
/// accepting results. No process deadline, recipe or report publication is added.
#[derive(Debug)]
pub struct WasmBuildEvidence {
    spec: WasmBuildSpec,
    inputs: ResolvedCargoBuildInputs,
    build: WasmBuildOutcome,
}

/// Provenance for the exact exported Wasm bytes; not deployment approval.
#[derive(Debug, Serialize)]
pub struct WasmArtifactReceipt {
    /// SHA-256 of the captured output.
    pub wasm_sha256: String,
    /// Cache identity of the retained build.
    pub build_fingerprint: String,
    /// Semantic source identity used by the build.
    pub input_digest: String,
    /// Full input observation used by freshness validation.
    pub validation_digest: String,
}

impl WasmBuildEvidence {
    /// Acquire one retained build and reject source changes during acquisition.
    ///
    /// The caller selects child custody through `spawn`, as with the underlying
    /// Wasm build. No signal handler, retry or elapsed-time deadline is added.
    ///
    /// # Errors
    /// Returns build/input errors as typed I/O sources, or a freshness refusal.
    pub fn acquire(
        spec: WasmBuildSpec,
        spawn: impl FnMut(&mut Command) -> io::Result<OwnedChild>,
    ) -> io::Result<Self> {
        let inputs = resolve_cargo_build_inputs(&spec).map_err(io::Error::other)?;
        let build = build_wasm_canisters_cached(&spec, spawn).map_err(io::Error::other)?;
        let evidence = Self {
            spec,
            inputs,
            build,
        };
        evidence.verify_current()?;
        Ok(evidence)
    }

    /// Refuse changed source or a build not matching the retained input identity.
    ///
    /// # Errors
    /// Returns input-observation errors or `InvalidData` for mismatched evidence.
    pub fn verify_current(&self) -> io::Result<()> {
        let current = resolve_cargo_build_inputs(&self.spec).map_err(io::Error::other)?;
        if self.build.record().fingerprint() != self.inputs.fingerprint()
            || current.fingerprint() != self.inputs.fingerprint()
            || current.validation_digest() != self.inputs.validation_digest()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "build inputs changed",
            ));
        }
        Ok(())
    }

    /// Capture a selected artifact under a byte cap and return its provenance.
    ///
    /// The caller owns the destination parent and custody against other writers.
    /// Existing destinations follow `CapturedArtifact::retain_to` replacement
    /// semantics. The destination must be outside this build's exact cache entry.
    /// Copies stream through bounded scratch; no full-Wasm allocation is needed.
    /// The receipt reuses the completed copy's digest; it is not an independent
    /// storage read-back check. The staged file must still be readable.
    /// Keep this owner alive and verify freshness after measurement as well.
    ///
    /// # Errors
    /// Returns invalid selection/path, freshness, capture, hash or retention errors.
    /// Failed retention preserves its typed recovery paths; existing evidence is
    /// unchanged on admission, capture, hash or freshness refusal.
    pub fn export_artifact(
        &self,
        index: usize,
        destination: &Path,
        maximum_bytes: usize,
    ) -> io::Result<WasmArtifactReceipt> {
        self.verify_current()?;
        let record = self.build.record();
        let source = record.artifacts().get(index).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "artifact index out of range")
        })?;
        let parent = destination.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "missing destination parent")
        })?;
        if parent
            .canonicalize()?
            .starts_with(record.exact_cache_path().canonicalize()?)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination is inside retained cache",
            ));
        }
        let (captured, identity) =
            CapturedArtifact::capture_with_identity(source, parent, maximum_bytes)?;
        // Preserve unreadable-stage refusal without hashing the same bytes again.
        fs::File::open(captured.path())?;
        self.verify_current()?;
        captured.retain_to(destination).map_err(io::Error::other)?;
        Ok(WasmArtifactReceipt {
            wasm_sha256: identity.sha256.to_string(),
            build_fingerprint: record.fingerprint().to_string(),
            input_digest: self.inputs.input_digest().to_string(),
            validation_digest: self.inputs.validation_digest().to_string(),
        })
    }
}

#[cfg(all(test, unix))]
mod tests;
