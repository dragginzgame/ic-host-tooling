//! Host-side artifact discovery, freshness, and Wasm build helpers.
//!
//! These functions keep integration-test artifacts in caller-selected target
//! directories and contain no application-specific package or profile policy.
//! Independent Wasm batches remain sequential; [`WasmBuildInputSnapshot`]
//! prepares a fixed specification set for concurrent readers under a caller-held
//! source lease. Independent batches resolve current inputs on each call.
//!
//! Successful Wasm and generic artifact records retain their read-only exact
//! paths against cache replacement and pruning. Retain the record (or a clone)
//! through consumption; paths and artifact descriptors alone carry no ownership.
//! Batch reports own their successful records even when later entries fail.
//!
//! ```no_run
//! use ic_build::artifacts::{
//!     ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec,
//!     WasmBuildSpec, build_wasm_canisters_cached, prepare_artifact_cache,
//!     resolve_cargo_build_inputs,
//! };
//! use ic_host_artifacts::artifact::hash_reader;
//! use ic_host_process::tool::{AdmittedTool, ExecutionContext, OutputLimits};
//! use std::path::Path;
//!
//! // The caller holds its source lease and configures the Cargo shim and actual
//! // compiler identity on spec. The admitted optimizer accepts INPUT -o OUTPUT.
//! fn build_deployable(
//!     spec: &WasmBuildSpec, post_cache: &Path, destination: &Path,
//!     optimizer: &AdmittedTool, context: &ExecutionContext<'_>, limits: OutputLimits,
//!     max_output_bytes: usize,
//! ) -> Result<Vec<u8>, Box<dyn std::error::Error>>
//! {
//!     let resolved = resolve_cargo_build_inputs(spec)?;
//!     let wasm = build_wasm_canisters_cached(spec, ic_host_process::child::OwnedChild::spawn_direct)?;
//!     let input = &wasm.record().artifacts()[0];
//!     let post_spec = ArtifactCacheSpec::new(post_cache, "deploy", "optimizer/v1")
//!         .with_cargo_build_inputs("cargo", spec, &resolved)
//!         .with_input("wasm", input)
//!         .with_tool("optimizer", optimizer.path())
//!         .with_arguments(["<input>", "-o", "<output>"])
//!         .with_environment(context.environment.iter().cloned())
//!         .with_identity_bytes("optimizer-cwd", context.current_dir.as_os_str().as_encoded_bytes())
//!         .with_output("deploy", destination);
//!     let deploy = match prepare_artifact_cache(&post_spec)? {
//!         ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
//!         ArtifactCachePreparation::Build(transaction) => {
//!             let output = transaction.output_path("deploy")?;
//!             optimizer.run(&[input.into(), "-o".into(), output.clone().into()], context, limits)?.require_success()?;
//!             hash_reader(std::fs::File::open(&output)?, max_output_bytes as u64)?;
//!             transaction.commit()?
//!         }
//!     };
//!     drop(wasm); // The post-link transaction has finished consuming the input.
//!     let bytes = ic_host_fs::read::read_file(
//!         deploy.record().artifacts()[0].path(), max_output_bytes)?;
//!     drop(deploy); // Reading is complete; maintenance may now reclaim it.
//!     Ok(bytes)
//! }
//! ```

#[cfg(unix)]
pub mod candid;

mod cache_fs;
mod digest;
mod inputs;
mod phase;
mod staging;
mod transaction;
mod transaction_batch;
mod wasm_batch;
mod wasm_cache;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(all(test, unix))]
mod host_tests;

pub use cache_fs::{ArtifactCacheMaintenance, ArtifactCachePruneReport, CacheRetentionLimits};
pub use digest::InputDigest;
pub use inputs::{
    InputEvidence, InputEvidenceError, InputEvidenceSpec, InputEvidenceValue, parse_cargo_depfile,
    read_cargo_depfile_inputs,
};
pub use phase::WasmBuildPhase;
pub use staging::{
    ArtifactStagingDirectory, CaptureRetentionError, CapturedArtifact, produce_and_retain_artifact,
};
pub use transaction::{
    ArtifactBuildTransaction, ArtifactCacheArtifact, ArtifactCacheError, ArtifactCacheOutcome,
    ArtifactCachePreparation, ArtifactCacheRecord, ArtifactCacheSpec, ArtifactCacheTimings,
    ArtifactOutputValidation, prepare_artifact_cache, prune_artifact_cache,
};
pub use transaction_batch::{
    ArtifactCacheBatchEntry, ArtifactCacheBatchFailure, ArtifactCacheBatchFailureTimings,
    ArtifactCacheBatchMetrics, ArtifactCacheBatchReport, LabeledArtifactCacheSpec,
    build_artifact_caches_batch,
};
pub use wasm_batch::{
    LabeledWasmBuildSpec, WasmBuildBatchConfig, WasmBuildBatchContractError, WasmBuildBatchEntry,
    WasmBuildBatchMetrics, WasmBuildBatchProgressEvent, WasmBuildBatchReport,
    WasmBuildInputSnapshot, WasmBuildInputSnapshotMetrics, build_wasm_canisters_cached_batch,
};
pub use wasm_cache::{
    CargoBuildFacts, CargoBuildInput, CargoInputProbePolicy, ResolvedCargoBuildInputs,
    SharedIncrementalTargetInspection, SharedIncrementalTargetMaintenance,
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceFailureMode,
    SharedIncrementalTargetMaintenanceOutcome, WasmBuildCacheMode, WasmBuildError,
    WasmBuildFailureDetails, WasmBuildFailureTimings, WasmBuildOutcome, WasmBuildProgressConfig,
    WasmBuildProgressEvent, WasmBuildProgressOutcome, WasmBuildRecord, WasmBuildSpec,
    WasmBuildTimings, WasmInputResolutionTimings, build_wasm_canisters_cached,
    build_wasm_canisters_cached_with_progress, inspect_shared_incremental_target,
    maintain_shared_incremental_target, maintain_shared_incremental_target_at_most_every,
    prune_wasm_build_cache, resolve_cargo_build_inputs,
    resolve_cargo_build_inputs_with_probe_policy,
};
