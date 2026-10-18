use super::phase::WasmBuildPhase;
use crate::cargo::{
    execution::{communicate_cargo, wasm_artifact_paths},
    messages::artifact_matches,
    observation::schema,
    selection::CargoSelection,
};
use ic_host_fs::owned_directory::{OwnedDirectory, remove_path_if_present};
use ic_host_process::{
    child::{CleanupPolicy, OwnedChild},
    tool::{
        CompletedExecution, ExecutionError, ExecutionEvidence, ExecutionFailure,
        ExecutionOperation, OutputLimit, OutputLimits, OutputStream, SuccessfulExit, ToolError,
        capture_command, communicate_child,
    },
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    ffi::{OsStr, OsString},
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc, RwLock,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};
use toml::Value as TomlValue;

mod acquisition;
use acquisition::prepare_exact_cache_entry;

use super::{
    cache_fs::{
        ArtifactCacheMaintenance, ArtifactCachePruneReport, CacheFsError, CacheRetentionLimits,
        RetainedCacheEntry, cache_entry_last_used, cache_maintenance_due,
        canonicalize_allow_missing, directory_logical_size,
        ensure_cache_directory_tag as ensure_cache_tag, is_sha256_directory, lock_cache_file,
        lock_cache_file_with_wait_observer, perform_scheduled_cache_maintenance,
        prune_direct_child_directories, record_cache_entry_use as record_entry_use,
        record_cache_maintenance, remove_unretained_entry,
    },
    digest::{
        FileDigest, InputDigest, InputHasher, LabeledPathDigestCache, copy_file_atomic,
        destination_matches_bytes, digest_bytes, digest_file, digest_labeled_paths_composable,
        materialize_verified_output, os_bytes, read_stamp_with_limit,
    },
};

const CACHE_FORMAT_VERSION: &str = "ic-testkit-wasm-build-v1";
// Version the recipe key separately from retained artifact/stamp formats.
const BUILD_FINGERPRINT_DOMAIN: &str = "ic-build-explicit-environment-fingerprint";
const ARTIFACT_STAMP_DIGEST_LABEL: &str = "artifact-sha256:";
// Consumer policy: diagnostic prefixes, complete probe output, and no deadline.
const CARGO_DIAGNOSTIC_BYTES: usize = 1024 * 1024;
const TOOL_IDENTITY_BYTES: usize = 64 * 1024;
const CARGO_METADATA_BYTES: usize = 16 * 1024 * 1024;

const DEFAULT_TARGET: &str = "wasm32-unknown-unknown";

/// Complete caller-owned description of one cacheable Cargo Wasm build.
///
/// The selected package graph, sources, semantic workspace projection, Cargo
/// configuration, Rust toolchain files, target, profile arguments, explicit
/// complete child environment and additional watched
/// inputs contribute to the build fingerprint. The complete workspace
/// manifest and lockfile remain conservative mutation-validation inputs.
#[derive(Clone, Eq, PartialEq)]
pub struct WasmBuildSpec {
    workspace_root: PathBuf,
    target_dir: PathBuf,
    packages: Vec<String>,
    example: Option<String>,
    cargo_selection: CargoSelection,
    environment: BTreeMap<OsString, OsString>,
    additional_inputs: Vec<PathBuf>,
    target: String,
    cargo_program: OsString,
    rustc_program: OsString,
    cargo_message_bytes: usize,
    cache_mode: WasmBuildCacheMode,
    pruning: Option<(CacheRetentionLimits, Duration)>,
    shared_incremental_maintenance_config: Option<SharedIncrementalTargetMaintenanceConfig>,
}

impl std::fmt::Debug for WasmBuildSpec {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WasmBuildSpec")
            .field("workspace_root", &self.workspace_root)
            .field("target_dir", &self.target_dir)
            .field("packages", &self.packages)
            .field("example", &self.example)
            .field("cargo_selection", &self.cargo_selection)
            .field(
                "environment_names",
                &self.environment.keys().collect::<Vec<_>>(),
            )
            .field("additional_inputs", &self.additional_inputs)
            .field("target", &self.target)
            .field("cargo_program", &self.cargo_program)
            .field("rustc_program", &self.rustc_program)
            .field("cargo_message_bytes", &self.cargo_message_bytes)
            .field("cache_mode", &self.cache_mode)
            .field("pruning", &self.pruning)
            .field(
                "shared_incremental_maintenance_config",
                &self.shared_incremental_maintenance_config,
            )
            .finish()
    }
}

/// Failure handling for integrated shared incremental-target maintenance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SharedIncrementalTargetMaintenanceFailureMode {
    /// Fail the Wasm acquisition when scheduled maintenance fails.
    #[default]
    Strict,
    /// Preserve the acquisition and attach a structured failed-maintenance outcome.
    BestEffort,
}

/// Scheduled shared incremental-target maintenance attached to a Wasm acquisition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedIncrementalTargetMaintenanceConfig {
    policy: CacheRetentionLimits,
    minimum_interval: Duration,
    failure_mode: SharedIncrementalTargetMaintenanceFailureMode,
}

/// Cargo-target ownership mode for one exact cached Wasm build.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildCacheMode {
    /// Build each exact fingerprint in its own content-addressed Cargo target.
    Isolated,
    /// Build misses in caller-owned shared Cargo incremental state, then cache final Wasm files.
    SharedIncremental {
        /// Mutable Cargo target directory shared across source fingerprints.
        target_dir: PathBuf,
    },
}

/// Whether a cacheable Wasm build ran Cargo or reused exact matching artifacts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildOutcome {
    /// Cargo ran and a new successful stamp was published.
    Built(WasmBuildRecord),
    /// Existing artifacts and their content-addressed stamp matched exactly.
    Reused(WasmBuildRecord),
}

/// Details shared by built and reused Wasm outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBuildRecord {
    facts: CargoBuildFacts,
    fingerprint: InputDigest,
    input_digest: InputDigest,
    retention: RetainedCacheEntry,
    artifacts: Arc<[PathBuf]>,
    timings: WasmBuildTimings,
    maintenance: Option<ArtifactCacheMaintenance>,
    shared_incremental_maintenance: Option<SharedIncrementalTargetMaintenanceOutcome>,
}

/// Timings for cache coordination, input resolution, and Cargo execution.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildTimings {
    lock_wait: Duration,
    shared_incremental_lock_wait: Option<Duration>,
    input_resolution: WasmInputResolutionTimings,
    cargo_build: Option<Duration>,
    cache_maintenance: Option<Duration>,
    total: Duration,
}

/// Detailed timings for exact Wasm build-input resolution.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmInputResolutionTimings {
    tool_identity: Duration,
    cargo_metadata: Duration,
    input_discovery: Duration,
    content_hashing: Duration,
    total: Duration,
}

/// Partial phase timings retained when a Wasm acquisition fails.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildFailureTimings {
    exact_cache_coordination: Duration,
    shared_target_coordination: Option<Duration>,
    input_resolution: WasmInputResolutionTimings,
    shared_target_maintenance: Option<Duration>,
    cargo_build: Option<Duration>,
    artifact_publication: Option<Duration>,
    exact_cache_maintenance: Option<Duration>,
    cleanup: Option<Duration>,
    total: Duration,
}

/// Structured phase and partial timings for one failed Wasm entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmBuildFailureDetails {
    phase: WasmBuildPhase,
    timings: WasmBuildFailureTimings,
}

/// One exact local Cargo source or configuration input under a stable logical label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoBuildInput {
    label: PathBuf,
    path: PathBuf,
}

/// Tool and recipe observations already used to fingerprint resolved inputs.
///
/// These facts do not establish successful compilation or bind an external
/// execution. Consumers retain source custody, execution binding and provenance
/// redaction. Environment values and probe bytes may contain secrets; Debug
/// reports only their sizes. Reading these observations performs no I/O.
#[derive(Clone, Eq, PartialEq)]
pub struct CargoBuildFacts {
    /// Selected profile, features and lock/network policy.
    pub selection: CargoSelection,
    /// Selected compilation target.
    pub target: String,
    /// Complete selected environment, before Build adds its owned target directory.
    pub environment: BTreeMap<OsString, OsString>,
    /// Exact successful Cargo probe stdout followed by stderr, as fingerprinted.
    pub cargo_identity: Vec<u8>,
    /// Exact successful compiler probe stdout followed by stderr, as fingerprinted.
    pub rustc_identity: Vec<u8>,
}

impl std::fmt::Debug for CargoBuildFacts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CargoBuildFacts")
            .field("selection", &self.selection)
            .field("target", &self.target)
            .field("environment_entries", &self.environment.len())
            .field("cargo_identity_bytes", &self.cargo_identity.len())
            .field("rustc_identity_bytes", &self.rustc_identity.len())
            .finish()
    }
}

impl CargoBuildFacts {
    fn observed(spec: &WasmBuildSpec, cargo_identity: Vec<u8>, rustc_identity: Vec<u8>) -> Self {
        Self {
            selection: spec.cargo_selection.clone(),
            target: spec.target.clone(),
            environment: spec.environment.clone(),
            cargo_identity,
            rustc_identity,
        }
    }
}

/// Resolved exact inputs and identity for one [`WasmBuildSpec`].
///
/// The snapshot can be resolved again after an external operation to detect
/// source, configuration, toolchain, argument, or environment changes.
/// Clones share immutable input and exclusion lists while retaining independent
/// timing values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedCargoBuildInputs {
    facts: CargoBuildFacts,
    fingerprint: InputDigest,
    input_digest: InputDigest,
    validation_digest: InputDigest,
    inputs: Arc<[CargoBuildInput]>,
    exclusions: Arc<[PathBuf]>,
    wasm_outputs: Result<ResolvedWasmOutputs, String>,
    timings: WasmInputResolutionTimings,
}

pub(super) struct WasmBuildBatchInputResolver<'a, 'session> {
    specs: Vec<&'a WasmBuildSpec>,
    groups: Vec<BatchResolutionGroup>,
    group_by_index: Vec<usize>,
    resolved: Vec<Option<Result<ResolvedCargoBuildInputs, (WasmBuildPhase, WasmBuildError)>>>,
    reuse: Option<&'session WasmBuildInputSnapshotState>,
    metrics: WasmBuildBatchInputMetrics,
}

pub(super) struct WasmBuildInputSnapshotState {
    snapshots: Vec<(WasmBuildSpec, ResolvedCargoBuildInputs)>,
    preparation_metrics: WasmBuildBatchInputMetrics,
    preparation_timings: WasmInputResolutionTimings,
    reader_reuses: AtomicUsize,
    invalidation: Arc<RwLock<bool>>,
}

struct BatchResolutionGroup {
    indexes: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WasmArtifactTarget {
    package_id: String,
    name: String,
    kind: schema::TargetKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedWasmOutputs {
    targets: Arc<[WasmArtifactTarget]>,
    artifact_paths: Arc<[PathBuf]>,
}

struct ResolvedLocalInputs {
    validation_inputs: Vec<(PathBuf, PathBuf)>,
    workspace_projection: Option<InputDigest>,
    wasm_targets: Result<Vec<WasmArtifactTarget>, String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct WasmBuildBatchInputMetrics {
    pub(super) runs: usize,
    pub(super) reuses: usize,
    pub(super) prepared_reuses: usize,
}

#[derive(Eq, Hash, PartialEq)]
struct BatchResolutionKey<'a> {
    workspace_root: &'a Path,
    cargo_program: &'a OsStr,
    rustc_program: &'a OsStr,
    metadata_arguments: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
}

/// Lock-coordinated disk-usage observation for a caller-owned shared Cargo target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedIncrementalTargetInspection {
    target_dir: PathBuf,
    logical_size_bytes: u64,
    last_used: SystemTime,
    lock_wait: Duration,
}

/// Result of explicit shared Cargo target maintenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedIncrementalTargetMaintenance {
    target_dir: PathBuf,
    logical_size_bytes_before: u64,
    logical_size_bytes_after: u64,
    last_used_before: SystemTime,
    cleared: bool,
    lock_wait: Duration,
    maintenance: Duration,
}

/// Result of interval-limited shared Cargo target maintenance.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedIncrementalTargetMaintenanceOutcome {
    /// The configured shared target does not exist, so nothing was created or inspected.
    Missing {
        /// Configured target path. A missing path cannot necessarily be canonicalized.
        target_dir: PathBuf,
    },
    /// A successful matching maintenance pass is still inside the requested interval.
    Skipped {
        /// Canonical shared Cargo target directory.
        target_dir: PathBuf,
        /// Time spent waiting for another process using the shared target.
        lock_wait: Duration,
        /// Time spent checking the small cross-process schedule marker.
        schedule_check: Duration,
    },
    /// Retention was evaluated under the shared-target lock.
    Performed {
        /// Completed retention report.
        maintenance: SharedIncrementalTargetMaintenance,
        /// Time spent checking the small cross-process schedule marker.
        schedule_check: Duration,
    },
    /// Integrated best-effort maintenance failed without invalidating the Wasm acquisition.
    Failed {
        /// Canonical shared Cargo target directory.
        target_dir: PathBuf,
        /// Time spent waiting for another process using the shared target.
        lock_wait: Duration,
        /// Rendered maintenance failure retained for diagnostics.
        message: String,
    },
}

/// Observation settings for one cacheable Wasm build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmBuildProgressConfig {
    heartbeat_interval: Option<Duration>,
    emit_cargo_output: bool,
}

/// Final cache state reported by a successful observed build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBuildProgressOutcome {
    /// Cargo ran and exact artifacts were published.
    Built,
    /// Exact artifacts were reused without Cargo.
    Reused,
}

/// Structured progress emitted by an observed cacheable Wasm build.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildProgressEvent {
    /// One build/cache acquisition started.
    Started,
    /// One exact Cargo input-resolution pass completed.
    InputsResolved {
        /// Complete exact build fingerprint.
        fingerprint: InputDigest,
        /// Semantic selected-source/configuration digest.
        input_digest: InputDigest,
        /// Time spent on this resolution pass.
        elapsed: Duration,
    },
    /// No reusable exact entry existed for this fingerprint.
    CacheMiss {
        /// Missing exact fingerprint.
        fingerprint: InputDigest,
    },
    /// Exact artifacts were found and materialized when necessary.
    CacheHit {
        /// Reused exact fingerprint.
        fingerprint: InputDigest,
    },
    /// The build is about to wait for a caller-owned shared Cargo target.
    SharedTargetLockStarted {
        /// Shared target selected by the build specification.
        target_dir: PathBuf,
    },
    /// Exclusive shared-target ownership was acquired.
    SharedTargetLockAcquired {
        /// Canonical shared target directory.
        target_dir: PathBuf,
        /// Time spent waiting for another process.
        wait: Duration,
    },
    /// Scheduled shared-target retention is about to be evaluated under lock.
    SharedTargetMaintenanceStarted {
        /// Canonical shared target selected by the build specification.
        target_dir: PathBuf,
    },
    /// Scheduled shared-target retention completed or was skipped.
    SharedTargetMaintenanceFinished {
        /// Structured retention result attached to the successful acquisition.
        outcome: SharedIncrementalTargetMaintenanceOutcome,
    },
    /// Cargo compilation started.
    CargoStarted {
        /// Cargo target receiving compilation state.
        target_dir: PathBuf,
    },
    /// One raw Cargo output chunk was read without lossy UTF-8 conversion.
    CargoOutput {
        /// Child-process stream that produced the bytes.
        stream: OutputStream,
        /// Raw output bytes in per-stream read order.
        bytes: Vec<u8>,
    },
    /// The current acquisition phase remained active without another event.
    Heartbeat {
        /// Phase that is still making or waiting for progress.
        phase: WasmBuildPhase,
        /// Time elapsed since this phase started.
        elapsed: Duration,
    },
    /// Cargo exited and all captured output was drained.
    CargoFinished {
        /// Whether Cargo reported success.
        success: bool,
        /// Portable exit code when the platform exposes one.
        code: Option<i32>,
        /// Complete Cargo execution duration.
        elapsed: Duration,
    },
    /// The complete cacheable build operation succeeded.
    Finished {
        /// Whether Cargo ran or an exact entry was reused.
        outcome: WasmBuildProgressOutcome,
        /// Exact fingerprint selected by the operation.
        fingerprint: InputDigest,
        /// Total operation duration.
        elapsed: Duration,
    },
}

impl Default for WasmBuildProgressConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Some(Duration::from_secs(10)),
            emit_cargo_output: true,
        }
    }
}

impl WasmBuildProgressConfig {
    /// Observe acquisition progress and emit a heartbeat at least every ten quiet seconds.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Select the maximum quiet interval between phase-aware heartbeat events.
    ///
    /// A zero interval is rejected before any build work begins.
    #[must_use]
    pub const fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }

    /// Disable time-based heartbeats while retaining phase and output events.
    #[must_use]
    pub const fn without_heartbeats(mut self) -> Self {
        self.heartbeat_interval = None;
        self
    }

    /// Select whether raw Cargo stdout/stderr chunks are forwarded.
    ///
    /// Output is always captured for structured build failures.
    /// Pending chunks use a bounded queue; slow observers can delay Cargo's
    /// writes rather than accumulate an unbounded forwarding backlog.
    #[must_use]
    pub const fn with_cargo_output(mut self, emit: bool) -> Self {
        self.emit_cargo_output = emit;
        self
    }

    /// Configured heartbeat interval, or `None` when disabled.
    #[must_use]
    pub const fn heartbeat_interval(self) -> Option<Duration> {
        self.heartbeat_interval
    }

    /// Whether raw Cargo output chunks are emitted to the observer.
    #[must_use]
    pub const fn emits_cargo_output(self) -> bool {
        self.emit_cargo_output
    }
}

struct ProgressReporter<'a> {
    config: WasmBuildProgressConfig,
    observer: Option<&'a mut dyn FnMut(WasmBuildProgressEvent)>,
    last_event: Instant,
    failure_phase: Option<WasmBuildPhase>,
    failure_timings: WasmBuildFailureTimings,
}

impl<'a> ProgressReporter<'a> {
    fn silent() -> Self {
        Self {
            config: WasmBuildProgressConfig {
                heartbeat_interval: None,
                emit_cargo_output: false,
            },
            observer: None,
            last_event: Instant::now(),
            failure_phase: None,
            failure_timings: WasmBuildFailureTimings::default(),
        }
    }

    fn new(
        observation: Option<(
            WasmBuildProgressConfig,
            &'a mut dyn FnMut(WasmBuildProgressEvent),
        )>,
    ) -> Result<Self, WasmBuildError> {
        let Some((config, observer)) = observation else {
            return Ok(Self::silent());
        };
        if config.heartbeat_interval == Some(Duration::ZERO) {
            return Err(WasmBuildError::InvalidSpec {
                message: "Wasm build progress heartbeat interval must be greater than zero"
                    .to_owned(),
            });
        }
        Ok(ProgressReporter {
            config,
            observer: Some(observer),
            last_event: Instant::now(),
            failure_phase: None,
            failure_timings: WasmBuildFailureTimings::default(),
        })
    }

    fn emit(&mut self, event: WasmBuildProgressEvent) {
        if let Some(observer) = &mut self.observer {
            observer(event);
            self.last_event = Instant::now();
        }
    }

    const fn is_observed(&self) -> bool {
        self.observer.is_some()
    }

    fn heartbeat_due_in(&self) -> Option<Duration> {
        self.config
            .heartbeat_interval
            .map(|interval| interval.saturating_sub(self.last_event.elapsed()))
    }

    fn emit_heartbeat(&mut self, phase: WasmBuildPhase, elapsed: Duration) {
        self.emit(WasmBuildProgressEvent::Heartbeat { phase, elapsed });
    }

    fn emit_heartbeat_if_due(&mut self, phase: WasmBuildPhase, elapsed: Duration) {
        if self.heartbeat_due_in() == Some(Duration::ZERO) {
            self.emit_heartbeat(phase, elapsed);
        }
    }

    fn run_phase<T, F>(&mut self, phase: WasmBuildPhase, operation: F) -> T
    where
        T: Send,
        F: FnOnce() -> T + Send,
    {
        let started = Instant::now();
        self.begin_phase(phase);
        let result = if !self.is_observed() || self.config.heartbeat_interval.is_none() {
            operation()
        } else {
            thread::scope(|scope| {
                let (finished, completion) = mpsc::sync_channel(0);
                let worker = scope.spawn(move || {
                    let result = operation();
                    let _ = finished.send(());
                    result
                });
                loop {
                    let wait = self
                        .heartbeat_due_in()
                        .expect("observed phase must have a heartbeat interval");
                    match completion.recv_timeout(wait) {
                        Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                            return worker
                                .join()
                                .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            self.emit_heartbeat(phase, started.elapsed());
                        }
                    }
                }
            })
        };
        self.record_phase(phase, started.elapsed());
        result
    }

    const fn begin_phase(&mut self, phase: WasmBuildPhase) {
        self.failure_phase = Some(phase);
    }

    fn record_phase(&mut self, phase: WasmBuildPhase, elapsed: Duration) {
        self.failure_phase = Some(phase);
        let timings = &mut self.failure_timings;
        // Select the authoritative timing slot once. Optional slots distinguish
        // an unrun phase from one that completed with zero elapsed time.
        let timing = match phase {
            WasmBuildPhase::Specification => return,
            WasmBuildPhase::ExactCacheCoordination => &mut timings.exact_cache_coordination,
            WasmBuildPhase::CargoIdentity | WasmBuildPhase::RustcIdentity => {
                &mut timings.input_resolution.tool_identity
            }
            WasmBuildPhase::CargoMetadata => &mut timings.input_resolution.cargo_metadata,
            WasmBuildPhase::InputDiscovery => &mut timings.input_resolution.input_discovery,
            WasmBuildPhase::ContentHashing => &mut timings.input_resolution.content_hashing,
            WasmBuildPhase::SharedTargetCoordination => timings
                .shared_target_coordination
                .get_or_insert(Duration::ZERO),
            WasmBuildPhase::SharedTargetMaintenance => timings
                .shared_target_maintenance
                .get_or_insert(Duration::ZERO),
            WasmBuildPhase::CargoBuild => timings.cargo_build.get_or_insert(Duration::ZERO),
            WasmBuildPhase::ArtifactPublication => {
                timings.artifact_publication.get_or_insert(Duration::ZERO)
            }
            WasmBuildPhase::ExactCacheMaintenance => timings
                .exact_cache_maintenance
                .get_or_insert(Duration::ZERO),
            WasmBuildPhase::Cleanup => timings.cleanup.get_or_insert(Duration::ZERO),
        };
        *timing = timing.saturating_add(elapsed);
        if matches!(
            phase,
            WasmBuildPhase::CargoIdentity
                | WasmBuildPhase::RustcIdentity
                | WasmBuildPhase::CargoMetadata
                | WasmBuildPhase::InputDiscovery
                | WasmBuildPhase::ContentHashing
        ) {
            timings.input_resolution.total = timings.input_resolution.total.saturating_add(elapsed);
        }
    }

    fn failure_details(&self, error: &WasmBuildError, total: Duration) -> WasmBuildFailureDetails {
        let phase = self
            .failure_phase
            .unwrap_or_else(|| classify_unobserved_failure(error));
        let mut timings = self.failure_timings;
        timings.total = total;
        WasmBuildFailureDetails { phase, timings }
    }
}

fn classify_unobserved_failure(error: &WasmBuildError) -> WasmBuildPhase {
    match error {
        WasmBuildError::InvalidSpec { .. } => WasmBuildPhase::Specification,
        WasmBuildError::CommandSpawn { phase, .. }
        | WasmBuildError::CommandFailed { phase, .. } => *phase,
        WasmBuildError::InvalidMetadata { .. } => WasmBuildPhase::CargoMetadata,
        WasmBuildError::InvalidCargoConfiguration { .. } => WasmBuildPhase::InputDiscovery,
        WasmBuildError::MissingArtifacts { .. }
        | WasmBuildError::PreparedInputSnapshotInvalidated => WasmBuildPhase::ArtifactPublication,
        WasmBuildError::InputsChangedDuringAcquisition { .. } => WasmBuildPhase::ContentHashing,
        WasmBuildError::FailedBuildCleanup { .. } => WasmBuildPhase::Cleanup,
        WasmBuildError::Io { source, .. }
            if source
                .get_ref()
                .is_some_and(<dyn std::error::Error + Send + Sync>::is::<ToolError>) =>
        {
            WasmBuildPhase::CargoBuild
        }
        WasmBuildError::Io { .. } => WasmBuildPhase::ExactCacheCoordination,
    }
}

/// Structured failure from a cacheable Wasm build.
#[non_exhaustive]
#[derive(Debug)]
pub enum WasmBuildError {
    /// The caller supplied an incomplete or inconsistent specification.
    InvalidSpec {
        /// Diagnostic describing the rejected input or evidence.
        message: String,
    },
    /// A filesystem operation failed.
    Io {
        /// Filesystem or process operation that failed.
        operation: &'static str,
        /// Exact path involved in the failed operation.
        path: PathBuf,
        /// Original underlying error, including its typed failure evidence.
        source: io::Error,
    },
    /// An external command could not be launched.
    CommandSpawn {
        /// Build phase in which the command failed.
        phase: WasmBuildPhase,
        /// Selected executable that could not be launched.
        program: OsString,
        /// Original underlying error, including its typed failure evidence.
        source: io::Error,
    },
    /// An external command completed unsuccessfully.
    /// Cargo-build streams retain at most 1 MiB of raw bytes each, with an explicit marker
    /// if truncated; optional progress callbacks still receive all output.
    CommandFailed {
        /// Build phase in which the command failed.
        phase: WasmBuildPhase,
        /// Original child exit status.
        status: ExitStatus,
        /// Retained standard output from the original Cargo invocation.
        stdout: String,
        /// Retained standard error from the original Cargo invocation.
        stderr: String,
    },
    /// Cargo metadata did not contain the expected package graph.
    InvalidMetadata {
        /// Diagnostic describing the rejected input or evidence.
        message: String,
    },
    /// A discovered Cargo configuration could not be interpreted exactly.
    InvalidCargoConfiguration {
        /// Exact path involved in the failed operation.
        path: PathBuf,
        /// Diagnostic describing the unsupported or malformed Cargo configuration.
        message: String,
    },
    /// Cargo succeeded without producing every declared Wasm artifact.
    MissingArtifacts {
        /// Expected artifact paths that were not produced.
        paths: Vec<PathBuf>,
    },
    /// Declared inputs changed while acquiring or building Wasm artifacts.
    InputsChangedDuringAcquisition {
        /// Input identity captured before the operation.
        before: InputDigest,
        /// Input identity observed during revalidation.
        after: InputDigest,
    },
    /// Another concurrent reader invalidated the prepared input snapshot before publication.
    PreparedInputSnapshotInvalidated,
    /// A build failed and its incomplete fingerprint directory could not be removed.
    FailedBuildCleanup {
        /// Original build failure before cleanup was attempted.
        build_error: Box<Self>,
        /// Exact path involved in the failed operation.
        path: PathBuf,
        /// Original underlying error, including its typed failure evidence.
        source: io::Error,
    },
}

impl WasmBuildSpec {
    /// Set the maximum bytes in one Cargo build message, including its newline.
    ///
    /// The default is 16 MiB. This decoding budget does not affect fingerprints
    /// or impose a build deadline. Overflow refuses publication after Cargo
    /// finishes; diagnostics continue streaming and the child is not terminated.
    #[must_use]
    pub const fn with_cargo_message_bytes(mut self, maximum: usize) -> Self {
        self.cargo_message_bytes = maximum;
        self
    }

    /// Describe one Cargo build targeting `wasm32-unknown-unknown`.
    ///
    /// The output subdirectory is derived from [`Self::with_cargo_selection`]:
    /// the default, `dev`, and `test` use `debug`; `release` and `bench` use
    /// `release`; custom profiles use their names. The derived directory must
    /// be one normal path component so artifacts remain inside their targets.
    /// Relative `workspace_root` and exact `target_dir` paths are resolved from
    /// the caller's working directory. Shared targets are workspace-relative.
    /// Package names are sorted and deduplicated for identity, discovery,
    /// Cargo invocation, and artifact paths.
    /// Misses share the workspace-relative `target/ic-testkit-incremental`
    /// Cargo target by default. Use [`Self::with_isolated_builds`] when compiler
    /// state must be independent, and select retention limits explicitly.
    /// By default, acquisition builds package libraries with Cargo's `--lib`
    /// flag. Use [`Self::with_example`] for one named `cdylib` example.
    /// Each default-library package must declare a `cdylib` with Cargo's default
    /// library name: package hyphens become underscores. Artifact paths use that
    /// admitted name; genuinely renamed libraries are refused.
    ///
    /// `environment` is the complete environment for probes and builds; child
    /// inheritance is cleared. Select PATH, `HOME`/`CARGO_HOME`, Rustup and build
    /// inputs at the caller as required. CARGO and RUSTC in this map select the
    /// initial tool programs; otherwise the literal programs cargo/rustc are used.
    #[must_use]
    pub fn new<I, K, V>(
        workspace_root: &Path,
        target_dir: &Path,
        packages: &[&str],
        environment: I,
    ) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        let environment: BTreeMap<OsString, OsString> = environment
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        let mut packages = packages
            .iter()
            .map(|package| (*package).to_owned())
            .collect::<Vec<_>>();
        packages.sort();
        packages.dedup();
        Self {
            workspace_root: workspace_root.to_owned(),
            target_dir: target_dir.to_owned(),
            packages,
            example: None,
            cargo_selection: CargoSelection::default(),
            cargo_program: environment
                .get(OsStr::new("CARGO"))
                .cloned()
                .unwrap_or_else(|| "cargo".into()),
            rustc_program: environment
                .get(OsStr::new("RUSTC"))
                .cloned()
                .unwrap_or_else(|| "rustc".into()),
            environment,
            additional_inputs: Vec::new(),
            target: DEFAULT_TARGET.to_owned(),
            cargo_message_bytes: CARGO_METADATA_BYTES,
            cache_mode: WasmBuildCacheMode::SharedIncremental {
                target_dir: PathBuf::from("target/ic-testkit-incremental"),
            },
            pruning: None,
            shared_incremental_maintenance_config: None,
        }
    }

    /// Select one named `cdylib` example instead of the default package library.
    ///
    /// Requires exactly one package. Metadata must declare that exact example;
    /// its selection contributes to identity and outputs live under `examples/`.
    /// Raw example/other-target selectors in profile arguments remain refused.
    #[must_use]
    pub fn with_example(mut self, name: impl Into<String>) -> Self {
        self.example = Some(name.into());
        self
    }

    /// Borrow the explicit example selection, if this is not a library recipe.
    #[must_use]
    pub fn example(&self) -> Option<&str> {
        self.example.as_deref()
    }

    /// Prepare this recipe's Cargo command without starting a process or cache.
    ///
    /// External execution uses `target_dir` directly; cache/maintenance choices
    /// are not executed. Resolve inputs first and borrow their admitted output
    /// paths, retain source/target custody, execute this command with caller-owned
    /// deadline/cancellation/cleanup, and validate inputs before capturing outputs.
    /// Keep the selected arguments and environment intact. Command success alone
    /// does not admit an artifact or release.
    ///
    /// # Errors
    /// Returns invalid selections or target-directory resolution errors.
    pub fn cargo_build_command(&self) -> Result<Command, WasmBuildError> {
        validate_spec(self)?;
        cargo_build_command(self, &self.target_dir)
    }

    /// Replace typed Cargo profile, feature and network/lock selections.
    ///
    /// The same selection configures metadata, builds and cache identity. Package,
    /// target, output paths and diagnostics remain owned by this specification.
    #[must_use]
    pub fn with_cargo_selection(mut self, selection: CargoSelection) -> Self {
        self.cargo_selection = selection;
        self
    }

    /// Replace the complete environment used by identity probes, metadata and builds.
    ///
    /// Inheritance is cleared. Absent names stay absent; duplicate names use the
    /// last supplied value. Consumers own capture, filtering and recapture policy.
    /// Cargo target directories remain owned by the build specification.
    #[must_use]
    pub fn with_environment<I, K, V>(mut self, environment: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        self.environment = environment
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        self
    }

    /// Borrow the explicitly selected child environment; values may contain secrets.
    #[must_use]
    pub const fn environment(&self) -> &BTreeMap<OsString, OsString> {
        &self.environment
    }

    /// Add files or directories not discoverable through Cargo's local dependency graph.
    ///
    /// Relative paths are resolved from the workspace root. Use this for build
    /// script configuration, generated schemas, or other externally read inputs.
    #[must_use]
    pub fn with_additional_inputs<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.additional_inputs
            .extend(paths.into_iter().map(Into::into));
        self
    }

    /// Override the Cargo compilation target.
    #[must_use]
    pub fn with_target(mut self, target: &str) -> Self {
        target.clone_into(&mut self.target);
        self
    }

    /// Override the Cargo executable used by metadata, identity, and build commands.
    #[must_use]
    pub fn with_cargo_program(mut self, program: impl Into<OsString>) -> Self {
        self.cargo_program = program.into();
        self
    }

    /// Override the Rust compiler executable used to fingerprint the toolchain.
    ///
    /// This selects the `-vV` identity probe; it does not change the compiler
    /// invoked by Cargo or infer a compiler hidden inside a Cargo shim. Supply
    /// the actual shim-selected compiler. To configure Cargo and its identity
    /// together, select `RUSTC` in the complete environment; that explicit environment
    /// selection takes precedence over this identity-only setting.
    #[must_use]
    pub fn with_rustc_identity_program(mut self, program: impl Into<OsString>) -> Self {
        self.rustc_program = program.into();
        self
    }

    /// Build cache misses in one caller-owned shared Cargo incremental target.
    ///
    /// Exact final Wasm artifacts still live in the content-addressed cache.
    /// The shared target is coordinated across processes but is never pruned
    /// or removed by IC Build after a failed build.
    #[must_use]
    pub fn with_shared_incremental_target(mut self, target_dir: impl Into<PathBuf>) -> Self {
        self.cache_mode = WasmBuildCacheMode::SharedIncremental {
            target_dir: target_dir.into(),
        };
        self
    }

    /// Build each fingerprint in a separate Cargo target instead of sharing state.
    ///
    /// Select this when the test explicitly requires isolated compiler state.
    /// Retained targets need a caller-selected prune policy to bound disk use.
    #[must_use]
    pub fn with_isolated_builds(mut self) -> Self {
        self.cache_mode = WasmBuildCacheMode::Isolated;
        self
    }

    /// Schedule caller-owned shared-target retention as part of acquisition.
    ///
    /// This option requires shared incremental mode (the default). Every
    /// acquisition coordinates through that target, including an exact hit,
    /// so a missing target can be created and receive its first schedule
    /// marker immediately. Matching recent passes only check the marker; due
    /// passes reuse the acquisition's exact Cargo input resolution before
    /// evaluating retention. The structured result is attached to the build
    /// record and emitted through observed progress. Maintenance failures fail
    /// the acquisition and do not record a successful schedule marker.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance_at_most_every(
        mut self,
        policy: CacheRetentionLimits,
        minimum_interval: Duration,
    ) -> Self {
        self.shared_incremental_maintenance_config = Some(
            SharedIncrementalTargetMaintenanceConfig::new(policy, minimum_interval),
        );
        self
    }

    /// Attach an explicit shared-target maintenance configuration.
    ///
    /// This is the configurable counterpart to
    /// [`Self::with_shared_incremental_target_maintenance_at_most_every`] and
    /// supports strict or best-effort failure handling.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance(
        mut self,
        config: SharedIncrementalTargetMaintenanceConfig,
    ) -> Self {
        self.shared_incremental_maintenance_config = Some(config);
        self
    }

    /// Apply cache retention under the build operation's existing process lock.
    ///
    /// Maintenance is best-effort: its structured result is attached to the
    /// successful build record and cannot turn ready artifacts into a build
    /// failure. The active fingerprint is protected from this pruning pass.
    #[must_use]
    pub const fn with_prune_policy(mut self, policy: CacheRetentionLimits) -> Self {
        self.pruning = Some((policy, Duration::ZERO));
        self
    }

    /// Apply exact-entry retention at most once per `minimum_interval`.
    ///
    /// The active fingerprint remains protected. A zero interval is equivalent
    /// to [`Self::with_prune_policy`]. The interval covers attempted
    /// maintenance, including a nonfatal failed attempt. This schedule never
    /// owns or scans a caller-owned shared incremental Cargo target.
    #[must_use]
    pub const fn with_prune_policy_at_most_every(
        mut self,
        policy: CacheRetentionLimits,
        minimum_interval: Duration,
    ) -> Self {
        self.pruning = Some((policy, minimum_interval));
        self
    }

    /// Workspace containing the selected Cargo packages.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Cargo target directory containing artifacts, lock, and stamps.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    /// Selected Cargo package names in sorted order, without duplicates.
    #[must_use]
    pub fn packages(&self) -> &[String] {
        &self.packages
    }

    /// Cargo-target ownership mode used for cache misses.
    #[must_use]
    pub const fn cache_mode(&self) -> &WasmBuildCacheMode {
        &self.cache_mode
    }

    /// Exact-entry retention policy attached to this specification, when configured.
    #[must_use]
    pub const fn prune_policy(&self) -> Option<CacheRetentionLimits> {
        match self.pruning {
            Some((policy, _)) => Some(policy),
            None => None,
        }
    }

    /// Positive minimum interval between exact-entry retention attempts.
    ///
    /// Both always-run builders, including an explicit zero interval, return
    /// `None`. Use [`Self::prune_policy`] to distinguish disabled maintenance.
    #[must_use]
    pub const fn prune_interval(&self) -> Option<Duration> {
        match self.pruning {
            Some((_, interval)) if !interval.is_zero() => Some(interval),
            _ => None,
        }
    }

    /// Shared incremental-target maintenance attached to this specification.
    #[must_use]
    pub const fn shared_incremental_target_maintenance(
        &self,
    ) -> Option<SharedIncrementalTargetMaintenanceConfig> {
        self.shared_incremental_maintenance_config
    }
}

impl WasmBuildOutcome {
    /// Read the common build record.
    #[must_use]
    pub const fn record(&self) -> &WasmBuildRecord {
        match self {
            Self::Built(record) | Self::Reused(record) => record,
        }
    }

    /// Report whether exact matching artifacts were reused.
    #[must_use]
    pub const fn is_reused(&self) -> bool {
        matches!(self, Self::Reused(_))
    }
}

impl WasmBuildRecord {
    /// Borrow the input observations bound to this artifact's fingerprint.
    ///
    /// Both built and reused records retain the selected recipe and tool probes.
    /// Reuse does not claim that a compiler ran during this acquisition. These
    /// facts perform no I/O; consumers own execution binding and redaction.
    #[must_use]
    pub const fn facts(&self) -> &CargoBuildFacts {
        &self.facts
    }

    /// Exact build fingerprint used by the atomic cache stamp.
    #[must_use]
    pub const fn fingerprint(&self) -> InputDigest {
        self.fingerprint
    }

    /// Semantic digest of selected package sources and configuration inputs.
    #[must_use]
    pub const fn input_digest(&self) -> InputDigest {
        self.input_digest
    }

    /// Immutable content-addressed cache directory for this exact build.
    ///
    /// The directory is selected by the build fingerprint and contains the
    /// cached Wasm artifacts and their stamps. The record and its clones retain
    /// this entry against pruning until their last drop. A copied path alone
    /// does not retain ownership. Treat the contents as read-only.
    #[must_use]
    pub fn exact_cache_path(&self) -> &Path {
        self.retention.path()
    }

    /// Exact, read-only Wasm paths retained until this record and its clones drop.
    ///
    /// Keep a record alive throughout post-link processing and reading. Configured
    /// materialization destinations remain mutable and are not retained.
    #[must_use]
    pub fn artifacts(&self) -> &[PathBuf] {
        &self.artifacts
    }

    /// Phase timings captured by the cacheable build operation.
    #[must_use]
    pub const fn timings(&self) -> WasmBuildTimings {
        self.timings
    }

    /// Cache maintenance attempted under the build lock, when configured.
    #[must_use]
    pub const fn maintenance(&self) -> Option<&ArtifactCacheMaintenance> {
        self.maintenance.as_ref()
    }

    /// Scheduled caller-owned shared-target maintenance, when configured.
    #[must_use]
    pub const fn shared_incremental_maintenance(
        &self,
    ) -> Option<&SharedIncrementalTargetMaintenanceOutcome> {
        self.shared_incremental_maintenance.as_ref()
    }
}

impl WasmBuildTimings {
    /// Time spent waiting for the output-directory process lock.
    #[must_use]
    pub const fn lock_wait(self) -> Duration {
        self.lock_wait
    }

    /// Time spent waiting for a shared incremental-target lock, when configured.
    #[must_use]
    pub const fn shared_incremental_lock_wait(self) -> Option<Duration> {
        self.shared_incremental_lock_wait
    }

    /// Detailed tool, metadata, discovery, and hashing timings.
    #[must_use]
    pub const fn input_resolution(self) -> WasmInputResolutionTimings {
        self.input_resolution
    }

    /// Time spent in `cargo build`, or `None` for a cache hit.
    #[must_use]
    pub const fn cargo_build(self) -> Option<Duration> {
        self.cargo_build
    }

    /// Time spent on configured best-effort cache maintenance.
    #[must_use]
    pub const fn cache_maintenance(self) -> Option<Duration> {
        self.cache_maintenance
    }

    /// Total operation duration, including lock coordination.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }

    pub(super) fn saturating_add(self, other: Self) -> Self {
        let mut input_resolution = self.input_resolution;
        input_resolution.include(other.input_resolution);
        Self {
            lock_wait: self.lock_wait.saturating_add(other.lock_wait),
            shared_incremental_lock_wait: [
                self.shared_incremental_lock_wait,
                other.shared_incremental_lock_wait,
            ]
            .into_iter()
            .flatten()
            .reduce(Duration::saturating_add),
            input_resolution,
            cargo_build: [self.cargo_build, other.cargo_build]
                .into_iter()
                .flatten()
                .reduce(Duration::saturating_add),
            cache_maintenance: [self.cache_maintenance, other.cache_maintenance]
                .into_iter()
                .flatten()
                .reduce(Duration::saturating_add),
            total: self.total.saturating_add(other.total),
        }
    }
}

impl WasmInputResolutionTimings {
    /// Time spent reading Cargo and rustc identities.
    #[must_use]
    pub const fn tool_identity(self) -> Duration {
        self.tool_identity
    }

    /// Time spent running and decoding `cargo metadata`.
    #[must_use]
    pub const fn cargo_metadata(self) -> Duration {
        self.cargo_metadata
    }

    /// Time spent resolving packages, configuration, and watched paths.
    #[must_use]
    pub const fn input_discovery(self) -> Duration {
        self.input_discovery
    }

    /// Time spent reading and hashing exact input contents.
    #[must_use]
    pub const fn content_hashing(self) -> Duration {
        self.content_hashing
    }

    /// Complete input-resolution duration.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }

    const fn include(&mut self, other: Self) {
        self.tool_identity = self.tool_identity.saturating_add(other.tool_identity);
        self.cargo_metadata = self.cargo_metadata.saturating_add(other.cargo_metadata);
        self.input_discovery = self.input_discovery.saturating_add(other.input_discovery);
        self.content_hashing = self.content_hashing.saturating_add(other.content_hashing);
        self.total = self.total.saturating_add(other.total);
    }
}

impl WasmBuildFailureDetails {
    pub(super) fn specification(total: Duration) -> Self {
        Self {
            phase: WasmBuildPhase::Specification,
            timings: WasmBuildFailureTimings {
                total,
                ..WasmBuildFailureTimings::default()
            },
        }
    }

    /// Primary acquisition phase that returned the failure.
    #[must_use]
    pub const fn phase(self) -> WasmBuildPhase {
        self.phase
    }

    /// Partial phase timings retained before the failure returned.
    #[must_use]
    pub const fn timings(self) -> WasmBuildFailureTimings {
        self.timings
    }
}

impl WasmBuildFailureTimings {
    /// Time spent coordinating the exact artifact cache before failure.
    #[must_use]
    pub const fn exact_cache_coordination(self) -> Duration {
        self.exact_cache_coordination
    }

    /// Time spent coordinating a shared incremental target, when reached.
    #[must_use]
    pub const fn shared_target_coordination(self) -> Option<Duration> {
        self.shared_target_coordination
    }

    /// Partial Cargo/rustc identity, metadata, discovery, and hashing timings.
    #[must_use]
    pub const fn input_resolution(self) -> WasmInputResolutionTimings {
        self.input_resolution
    }

    /// Time spent on shared-target maintenance, when reached.
    #[must_use]
    pub const fn shared_target_maintenance(self) -> Option<Duration> {
        self.shared_target_maintenance
    }

    /// Time spent executing Cargo, including an unsuccessful execution.
    #[must_use]
    pub const fn cargo_build(self) -> Option<Duration> {
        self.cargo_build
    }

    /// Time spent validating or publishing artifacts, when reached.
    #[must_use]
    pub const fn artifact_publication(self) -> Option<Duration> {
        self.artifact_publication
    }

    /// Time spent on exact-cache maintenance, when reached.
    #[must_use]
    pub const fn exact_cache_maintenance(self) -> Option<Duration> {
        self.exact_cache_maintenance
    }

    /// Explicit incomplete-entry cleanup time, when failure required it.
    #[must_use]
    pub const fn cleanup(self) -> Option<Duration> {
        self.cleanup
    }

    /// Complete failed acquisition wall time.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl CargoBuildInput {
    /// Stable checkout-independent label used while hashing this input.
    #[must_use]
    pub fn label(&self) -> &Path {
        &self.label
    }

    /// Resolved file or directory read by the Cargo build.
    ///
    /// Configuration inputs preserve their lookup paths so mutation guards
    /// observe replaced symlinks as well as changed file contents.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Caller-owned controls for Cargo identity, compiler identity and metadata probes.
///
/// Each probe runs in an owned process group using Host's existing communication
/// and cleanup machinery. Output limits remain fixed by IC Build. These controls
/// do not impose a deadline on filesystem discovery or hashing, execute a build,
/// or alter fingerprints. Existing resolution APIs retain their unbounded defaults.
pub struct CargoInputProbePolicy<'a> {
    /// Absolute deadline shared by every probe, including subsequent revalidation.
    /// Spawning syscalls and cleanup can exceed it; `None` imposes no deadline.
    pub deadline: Option<Instant>,
    /// Termination grace and reaping policy, separate from the probe deadline.
    pub cleanup: CleanupPolicy,
    /// Cancellation predicate, polled even while a probe is silent.
    /// It must return promptly; callback panics attempt cleanup before unwinding.
    pub cancelled: &'a mut (dyn FnMut() -> bool + Send),
}

impl ResolvedCargoBuildInputs {
    /// Borrow the recipe and tool observations used by this snapshot, without I/O.
    #[must_use]
    pub const fn facts(&self) -> &CargoBuildFacts {
        &self.facts
    }

    fn assemble(
        spec: &WasmBuildSpec,
        inputs: ResolvedLocalInputs,
        exclusions: Vec<PathBuf>,
        input_digest: InputDigest,
        validation_digest: InputDigest,
        facts: CargoBuildFacts,
        timings: impl FnOnce() -> WasmInputResolutionTimings,
    ) -> Self {
        Self {
            fingerprint: finish_build_fingerprint(spec, &facts, input_digest),
            facts,
            input_digest,
            validation_digest,
            inputs: cargo_build_inputs(inputs.validation_inputs),
            exclusions: exclusions.into(),
            wasm_outputs: inputs.wasm_targets.map(|targets| ResolvedWasmOutputs {
                artifact_paths: expected_artifacts(spec, &spec.target_dir, &targets).into(),
                targets: targets.into(),
            }),
            // Fresh-resolution totals include path conversion and assembly.
            timings: timings(),
        }
    }

    /// Borrow declared output paths after metadata admits the selected targets.
    ///
    /// These are paths under the specification's `target_dir`, not retained or
    /// captured outputs. External producers must execute the same recipe under
    /// source/target custody and validate before consuming or capturing bytes.
    /// Generic snapshots for non-Wasm packages remain useful independently.
    ///
    /// # Errors
    /// Returns missing/non-cdylib targets or colliding retained artifact names.
    pub fn artifact_paths(&self) -> Result<&[PathBuf], WasmBuildError> {
        self.wasm_outputs()
            .map(|outputs| outputs.artifact_paths.as_ref())
    }

    fn wasm_outputs(&self) -> Result<&ResolvedWasmOutputs, WasmBuildError> {
        self.wasm_outputs
            .as_ref()
            .map_err(|message| WasmBuildError::InvalidSpec {
                message: message.clone(),
            })
    }

    /// Exact build fingerprint including Cargo inputs, tools, arguments, and environment.
    #[must_use]
    pub const fn fingerprint(&self) -> InputDigest {
        self.fingerprint
    }

    /// Semantic digest of selected Cargo sources and workspace configuration.
    ///
    /// Unlike [`Self::validation_digest`], this may remain unchanged after an
    /// unrelated host-only workspace manifest or lockfile update.
    #[must_use]
    pub const fn input_digest(&self) -> InputDigest {
        self.input_digest
    }

    /// Conservative digest of every raw source and configuration input.
    ///
    /// This digest is used for mutation guards. It may change while
    /// [`Self::input_digest`] and [`Self::fingerprint`] remain unchanged.
    #[must_use]
    pub const fn validation_digest(&self) -> InputDigest {
        self.validation_digest
    }

    /// Stable logical labels and conservative resolved validation paths.
    #[must_use]
    pub fn inputs(&self) -> &[CargoBuildInput] {
        &self.inputs
    }

    /// Generated-state roots excluded while recursively hashing local inputs.
    ///
    /// These exclusions are derived by IC Build; callers cannot add
    /// arbitrary exclusions through this snapshot.
    #[must_use]
    pub fn exclusions(&self) -> &[PathBuf] {
        &self.exclusions
    }

    /// Timings for tool identity, metadata, discovery, and content hashing.
    #[must_use]
    pub const fn timings(&self) -> WasmInputResolutionTimings {
        self.timings
    }

    /// Resolve `spec` again and report whether its exact identity is unchanged.
    ///
    /// # Errors
    /// Returns specification, tool/metadata, filesystem or input-resolution errors.
    pub fn is_current(&self, spec: &WasmBuildSpec) -> Result<bool, WasmBuildError> {
        resolve_cargo_build_inputs(spec).map(|current| current.fingerprint == self.fingerprint)
    }

    /// Resolve again using the caller's probe deadline, cancellation and cleanup.
    ///
    /// Reusing the same policy preserves its absolute deadline across acquisitions.
    /// Filesystem discovery and hashing are outside the probe deadline.
    ///
    /// # Errors
    /// Returns the same errors as [`resolve_cargo_build_inputs_with_probe_policy`].
    pub fn is_current_with_probe_policy(
        &self,
        spec: &WasmBuildSpec,
        policy: &mut CargoInputProbePolicy<'_>,
    ) -> Result<bool, WasmBuildError> {
        resolve_cargo_build_inputs_with_probe_policy(spec, policy)
            .map(|current| current.fingerprint == self.fingerprint)
    }

    /// Rehash the already discovered Cargo source/configuration set.
    ///
    /// This is cheaper than rerunning Cargo metadata and is intended for
    /// before/after guards around external artifact transformations. Resolve a
    /// new snapshot to observe tool, argument, environment, or dependency-graph
    /// identity changes between separate acquisitions.
    ///
    /// # Errors
    /// Returns filesystem errors while rehashing the previously discovered inputs.
    pub fn is_content_current(&self) -> Result<bool, WasmBuildError> {
        self.current_validation_digest()
            .map(|current| current == self.validation_digest)
    }

    pub(super) fn current_validation_digest(&self) -> Result<InputDigest, WasmBuildError> {
        digest_labeled_paths_composable(
            "wasm-source-inputs-v1",
            self.inputs
                .iter()
                .map(|input| (input.label.as_path(), input.path.as_path())),
            &self.exclusions,
            &mut LabeledPathDigestCache::default(),
        )
        .map_err(|source| WasmBuildError::Io {
            operation: "rehash resolved Cargo build inputs",
            path: self
                .inputs
                .first()
                .map_or_else(PathBuf::new, |input| input.path.clone()),
            source,
        })
    }
}

impl WasmBuildInputSnapshotState {
    pub(super) fn prepare(specs: &[WasmBuildSpec]) -> Result<Self, WasmBuildError> {
        for spec in specs {
            validate_spec(spec)?;
        }
        let mut resolver = WasmBuildBatchInputResolver::new(specs, None)
            .map_err(|_| WasmBuildError::PreparedInputSnapshotInvalidated)?;
        let mut snapshots = Vec::with_capacity(specs.len());
        let mut preparation_timings = WasmInputResolutionTimings::default();
        let mut progress = ProgressReporter::silent();
        for (index, spec) in specs.iter().enumerate() {
            let mut prepared_input = resolver.resolve(index, &mut progress)?;
            preparation_timings.include(prepared_input.timings);
            prepared_input.timings = WasmInputResolutionTimings::default();
            snapshots.push((spec.clone(), prepared_input));
        }
        Ok(Self {
            snapshots,
            preparation_metrics: resolver.metrics(),
            preparation_timings,
            reader_reuses: AtomicUsize::new(0),
            invalidation: Arc::new(RwLock::new(false)),
        })
    }

    fn admit(&self, specs: &[&WasmBuildSpec]) -> Result<Vec<ResolvedCargoBuildInputs>, usize> {
        let matched = specs
            .iter()
            .enumerate()
            .map(|(index, spec)| {
                self.snapshots
                    .iter()
                    .find(|(candidate, _)| candidate == *spec)
                    .map(|(_, snapshot)| snapshot.clone())
                    .ok_or(index)
            })
            .collect::<Result<Vec<_>, _>>()?;
        #[allow(deprecated, reason = "try_update is unstable at the Rust 1.91 MSRV")]
        let _ = self
            .reader_reuses
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_add(matched.len()))
            });
        Ok(matched)
    }

    pub(super) const fn specification_count(&self) -> usize {
        self.snapshots.len()
    }

    pub(super) const fn preparation_metrics(&self) -> WasmBuildBatchInputMetrics {
        self.preparation_metrics
    }

    pub(super) const fn preparation_timings(&self) -> WasmInputResolutionTimings {
        self.preparation_timings
    }

    pub(super) fn reader_reuses(&self) -> usize {
        self.reader_reuses.load(Ordering::Relaxed)
    }

    pub(super) fn is_invalidated(&self) -> bool {
        *self
            .invalidation
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn invalidate(&self) {
        *self
            .invalidation
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
    }

    fn invalidation(&self) -> Arc<RwLock<bool>> {
        Arc::clone(&self.invalidation)
    }
}

impl<'a, 'session> WasmBuildBatchInputResolver<'a, 'session> {
    pub(super) fn new(
        specs: impl IntoIterator<Item = &'a WasmBuildSpec>,
        reuse: Option<&'session WasmBuildInputSnapshotState>,
    ) -> Result<Self, usize> {
        let specs = specs.into_iter().collect::<Vec<_>>();
        let prepared = match reuse.as_ref() {
            Some(snapshot) => Some(snapshot.admit(&specs)?),
            _ => None,
        };
        let mut groups_by_key = HashMap::new();
        let mut groups = Vec::<BatchResolutionGroup>::new();
        let mut group_by_index = Vec::with_capacity(specs.len());
        for (index, spec) in specs.iter().copied().enumerate() {
            let group = *groups_by_key
                .entry(BatchResolutionKey::for_spec(spec))
                .or_insert_with(|| {
                    groups.push(BatchResolutionGroup {
                        indexes: Vec::new(),
                    });
                    groups.len() - 1
                });
            groups[group].indexes.push(index);
            group_by_index.push(group);
        }
        let mut metrics = WasmBuildBatchInputMetrics::default();
        let resolved = prepared.map_or_else(
            || specs.iter().map(|_| None).collect(),
            |prepared| {
                metrics.prepared_reuses = prepared.len();
                prepared
                    .into_iter()
                    .map(|snapshot| Some(Ok(snapshot)))
                    .collect()
            },
        );
        Ok(Self {
            specs,
            groups,
            group_by_index,
            resolved,
            reuse,
            metrics,
        })
    }

    pub(super) const fn metrics(&self) -> WasmBuildBatchInputMetrics {
        self.metrics
    }

    pub(super) fn invalidate_source_lease(&self) {
        if let Some(snapshot) = self.reuse {
            snapshot.invalidate();
        }
    }

    pub(super) fn prepared_invalidation(&self) -> Option<Arc<RwLock<bool>>> {
        self.reuse.map(WasmBuildInputSnapshotState::invalidation)
    }

    fn resolve(
        &mut self,
        index: usize,
        progress: &mut ProgressReporter<'_>,
    ) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
        if self.resolved[index].is_none() {
            self.resolve_group(index, progress)?;
        }
        self.resolved[index]
            .take()
            .expect("resolved batch input must be populated")
            .map_err(|(phase, error)| {
                progress.begin_phase(phase);
                error
            })
    }

    fn resolve_group(
        &mut self,
        active_index: usize,
        progress: &mut ProgressReporter<'_>,
    ) -> Result<(), WasmBuildError> {
        let total_started = Instant::now();
        let group = self.group_by_index[active_index];
        let active = self.specs[active_index];

        let (cargo_identity, rustc_identity, tool_identity) =
            resolve_tool_identity(active, progress)?;

        let metadata_started = Instant::now();
        let metadata =
            progress.run_phase(WasmBuildPhase::CargoMetadata, || cargo_metadata(active))?;
        let cargo_metadata = metadata_started.elapsed();

        let (discovered, input_discovery) = self.discover_group_inputs(group, &metadata, progress);

        let hashing_started = Instant::now();
        let mut batch_digest_cache = LabeledPathDigestCache::default();
        let digest_cache = &mut batch_digest_cache;
        let workspace_root = &active.workspace_root;
        let resolved_inputs = progress.run_phase(WasmBuildPhase::ContentHashing, || {
            discovered
                .into_iter()
                .map(|(index, inputs, exclusions)| {
                    let result = digest_resolved_local_inputs(
                        &inputs,
                        &exclusions,
                        digest_cache,
                        workspace_root,
                        "hash batched Wasm build inputs",
                        "hash batched semantic Wasm build inputs",
                    )
                    .map(|(input_digest, validation_digest)| {
                        (inputs, exclusions, input_digest, validation_digest)
                    });
                    (index, result)
                })
                .collect::<Vec<_>>()
        });
        let content_hashing = hashing_started.elapsed();
        let timings = WasmInputResolutionTimings {
            tool_identity,
            cargo_metadata,
            input_discovery,
            content_hashing,
            total: total_started.elapsed(),
        };
        let resolved_count = resolved_inputs
            .iter()
            .filter(|(_, result)| result.is_ok())
            .count();
        self.metrics.runs += usize::from(resolved_count > 0);
        self.metrics.reuses += resolved_count.saturating_sub(1);
        let timing_index = resolved_inputs
            .iter()
            .find(|(index, result)| *index == active_index && result.is_ok())
            .or_else(|| resolved_inputs.iter().find(|(_, result)| result.is_ok()))
            .map(|(index, _)| *index);
        for (index, result) in resolved_inputs {
            let (inputs, exclusions, input_digest, validation_digest) = match result {
                Ok(resolved) => resolved,
                Err(error) => {
                    self.resolved[index] = Some(Err((WasmBuildPhase::ContentHashing, error)));
                    continue;
                }
            };
            let spec = self.specs[index];
            let resolved = ResolvedCargoBuildInputs::assemble(
                spec,
                inputs,
                exclusions,
                input_digest,
                validation_digest,
                CargoBuildFacts::observed(spec, cargo_identity.clone(), rustc_identity.clone()),
                || {
                    if Some(index) == timing_index {
                        timings
                    } else {
                        WasmInputResolutionTimings::default()
                    }
                },
            );
            self.resolved[index] = Some(Ok(resolved));
        }
        Ok(())
    }

    fn discover_group_inputs(
        &mut self,
        group: usize,
        metadata: &schema::Metadata,
        progress: &mut ProgressReporter<'_>,
    ) -> (Vec<(usize, ResolvedLocalInputs, Vec<PathBuf>)>, Duration) {
        let started = Instant::now();
        let indexes = &self.groups[group].indexes;
        let specs = &self.specs;
        let resolved = &mut self.resolved;
        let discovered = progress.run_phase(WasmBuildPhase::InputDiscovery, || {
            let parsed = ParsedCargoMetadata::parse(metadata);
            let mut discovered = Vec::new();
            for &index in indexes {
                if resolved[index].is_some() || validate_spec(specs[index]).is_err() {
                    continue;
                }
                let result = parsed
                    .as_ref()
                    .map_err(|message| invalid_metadata(message))
                    .and_then(|parsed| resolve_local_inputs(specs[index], parsed));
                match result {
                    Ok((inputs, exclusions)) => discovered.push((index, inputs, exclusions)),
                    Err(error) => {
                        resolved[index] = Some(Err((WasmBuildPhase::InputDiscovery, error)));
                    }
                }
            }
            discovered
        });
        (discovered, started.elapsed())
    }
}

fn resolve_tool_identity(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
) -> Result<(Vec<u8>, Vec<u8>, Duration), WasmBuildError> {
    resolve_tool_identity_with_policy(spec, progress, &mut None)
}

fn resolve_tool_identity_with_policy(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
    policy: &mut Option<&mut CargoInputProbePolicy<'_>>,
) -> Result<(Vec<u8>, Vec<u8>, Duration), WasmBuildError> {
    let started = Instant::now();
    let cargo_identity = progress.run_phase(WasmBuildPhase::CargoIdentity, || {
        command_identity_with_policy(
            spec,
            WasmBuildPhase::CargoIdentity,
            &spec.cargo_program,
            &["--version", "--verbose"],
            policy,
        )
    })?;
    let rustc_program = spec
        .environment
        .get(OsStr::new("RUSTC"))
        .unwrap_or(&spec.rustc_program);
    let rustc_identity = progress.run_phase(WasmBuildPhase::RustcIdentity, || {
        command_identity_with_policy(
            spec,
            WasmBuildPhase::RustcIdentity,
            rustc_program,
            &["-vV"],
            policy,
        )
    })?;
    Ok((cargo_identity, rustc_identity, started.elapsed()))
}

impl<'a> BatchResolutionKey<'a> {
    fn for_spec(spec: &'a WasmBuildSpec) -> Self {
        Self {
            workspace_root: &spec.workspace_root,
            cargo_program: &spec.cargo_program,
            rustc_program: spec
                .environment
                .get(OsStr::new("RUSTC"))
                .unwrap_or(&spec.rustc_program),
            metadata_arguments: spec.cargo_selection.metadata_arguments(),
            environment: spec.environment.clone(),
        }
    }
}

impl SharedIncrementalTargetInspection {
    /// Canonical shared Cargo target directory that was inspected.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    /// Logical bytes currently occupied by the complete shared target.
    #[must_use]
    pub const fn logical_size_bytes(&self) -> u64 {
        self.logical_size_bytes
    }

    /// Most recent build use recorded by IC Build, or the directory mtime for older targets.
    #[must_use]
    pub const fn last_used(&self) -> SystemTime {
        self.last_used
    }

    /// Time spent waiting for another process using the shared target.
    #[must_use]
    pub const fn lock_wait(&self) -> Duration {
        self.lock_wait
    }
}

impl SharedIncrementalTargetMaintenanceConfig {
    /// Schedule one strict retention pass at most once per interval.
    #[must_use]
    pub const fn new(policy: CacheRetentionLimits, minimum_interval: Duration) -> Self {
        Self {
            policy,
            minimum_interval,
            failure_mode: SharedIncrementalTargetMaintenanceFailureMode::Strict,
        }
    }

    /// Select whether an integrated maintenance failure fails the acquisition.
    #[must_use]
    pub const fn with_failure_mode(
        mut self,
        failure_mode: SharedIncrementalTargetMaintenanceFailureMode,
    ) -> Self {
        self.failure_mode = failure_mode;
        self
    }

    /// Configured whole-target retention policy.
    #[must_use]
    pub const fn policy(self) -> CacheRetentionLimits {
        self.policy
    }

    /// Minimum interval between successful matching maintenance passes.
    #[must_use]
    pub const fn minimum_interval(self) -> Duration {
        self.minimum_interval
    }

    /// Configured maintenance failure handling.
    #[must_use]
    pub const fn failure_mode(self) -> SharedIncrementalTargetMaintenanceFailureMode {
        self.failure_mode
    }
}

impl SharedIncrementalTargetMaintenance {
    /// Canonical shared Cargo target directory maintained under lock.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    /// Logical bytes observed before applying the policy.
    #[must_use]
    pub const fn logical_size_bytes_before(&self) -> u64 {
        self.logical_size_bytes_before
    }

    /// Logical bytes retained after applying the policy.
    #[must_use]
    pub const fn logical_size_bytes_after(&self) -> u64 {
        self.logical_size_bytes_after
    }

    /// Most recent build use observed before applying the policy.
    #[must_use]
    pub const fn last_used_before(&self) -> SystemTime {
        self.last_used_before
    }

    /// Whether a configured limit caused the mutable target contents to be cleared.
    #[must_use]
    pub const fn was_cleared(&self) -> bool {
        self.cleared
    }

    /// Time spent waiting for another process using the shared target.
    #[must_use]
    pub const fn lock_wait(&self) -> Duration {
        self.lock_wait
    }

    /// Time spent measuring and, when required, clearing the target.
    #[must_use]
    pub const fn maintenance(&self) -> Duration {
        self.maintenance
    }
}

impl std::fmt::Display for SharedIncrementalTargetMaintenance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "target={} action={} bytes={}=>{} lock={:?} maintenance={:?}",
            self.target_dir.display(),
            if self.cleared { "cleared" } else { "retained" },
            self.logical_size_bytes_before,
            self.logical_size_bytes_after,
            self.lock_wait,
            self.maintenance,
        )
    }
}

impl SharedIncrementalTargetMaintenanceOutcome {
    /// Configured or canonical target associated with this result.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        match self {
            Self::Missing { target_dir }
            | Self::Skipped { target_dir, .. }
            | Self::Failed { target_dir, .. } => target_dir,
            Self::Performed { maintenance, .. } => maintenance.target_dir(),
        }
    }

    /// Completed maintenance report, when retention was evaluated.
    #[must_use]
    pub const fn maintenance(&self) -> Option<&SharedIncrementalTargetMaintenance> {
        match self {
            Self::Performed { maintenance, .. } => Some(maintenance),
            Self::Missing { .. } | Self::Skipped { .. } | Self::Failed { .. } => None,
        }
    }

    /// Whether retention was evaluated during this call.
    #[must_use]
    pub const fn was_performed(&self) -> bool {
        matches!(self, Self::Performed { .. })
    }

    /// Time spent waiting for another process, when the target existed.
    #[must_use]
    pub const fn lock_wait(&self) -> Option<Duration> {
        match self {
            Self::Missing { .. } => None,
            Self::Skipped { lock_wait, .. } | Self::Failed { lock_wait, .. } => Some(*lock_wait),
            Self::Performed { maintenance, .. } => Some(maintenance.lock_wait()),
        }
    }

    /// Time spent checking the schedule marker, when the target existed.
    #[must_use]
    pub const fn schedule_check(&self) -> Option<Duration> {
        match self {
            Self::Missing { .. } | Self::Failed { .. } => None,
            Self::Skipped { schedule_check, .. } | Self::Performed { schedule_check, .. } => {
                Some(*schedule_check)
            }
        }
    }

    /// Rendered integrated maintenance failure, when best-effort handling preserved acquisition.
    #[must_use]
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Failed { message, .. } => Some(message),
            Self::Missing { .. } | Self::Skipped { .. } | Self::Performed { .. } => None,
        }
    }
}

impl std::fmt::Display for SharedIncrementalTargetMaintenanceOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { target_dir } => {
                write!(formatter, "target={} action=missing", target_dir.display())
            }
            Self::Skipped {
                target_dir,
                lock_wait,
                schedule_check,
            } => write!(
                formatter,
                "target={} action=skipped lock={lock_wait:?} schedule={schedule_check:?}",
                target_dir.display(),
            ),
            Self::Performed {
                maintenance,
                schedule_check,
            } => write!(formatter, "{maintenance} schedule={schedule_check:?}"),
            Self::Failed {
                target_dir,
                lock_wait,
                message,
            } => write!(
                formatter,
                "target={} action=failed lock={lock_wait:?} error={message}",
                target_dir.display(),
            ),
        }
    }
}

impl std::fmt::Display for WasmBuildTimings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "total={:?} lock={:?} shared_lock={:?} inputs={:?} cargo={:?} maintenance={:?}",
            self.total,
            self.lock_wait,
            self.shared_incremental_lock_wait,
            self.input_resolution.total,
            self.cargo_build,
            self.cache_maintenance,
        )
    }
}

impl std::fmt::Display for WasmBuildOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = if self.is_reused() { "reused" } else { "built" };
        write!(
            formatter,
            "{state} fingerprint={} artifacts={} {}",
            self.record().fingerprint,
            self.record().artifacts.len(),
            self.record().timings,
        )?;
        if let Some(maintenance) = self.record().shared_incremental_maintenance() {
            write!(formatter, " shared_maintenance=({maintenance})")?;
        }
        Ok(())
    }
}

/// Resolve the exact Cargo source, configuration, toolchain, argument, and environment identity.
///
/// This performs the same resolution used before and after cached Wasm builds
/// without running `cargo build`.
/// Input-only snapshots may describe ordinary libraries or other Cargo targets;
/// the `cdylib` name/output constraint is checked when acquiring Wasm artifacts.
/// Tool identities require complete output within 64 KiB per stream. Metadata
/// requires complete stdout within 16 MiB and stderr within 64 KiB. Probes have
/// no elapsed-time deadline; oversized output fails before parsing or hashing.
///
/// # Errors
/// Returns invalid specifications, tool/metadata probe failures, filesystem
/// errors or Cargo input discovery/resolution errors.
pub fn resolve_cargo_build_inputs(
    spec: &WasmBuildSpec,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    validate_spec(spec)?;
    build_fingerprint(spec)
}

/// Resolve Cargo inputs with caller-selected probe execution controls.
///
/// Uses the same commands, environment, complete-output limits, input discovery,
/// target admission and fingerprinting as [`resolve_cargo_build_inputs`]. Only
/// probe execution changes: Host owns each process group and retains original
/// failure, stdout/stderr and any cleanup failure in its [`ToolError`], stored
/// inside [`WasmBuildError::Io`]. Spawn errors retain [`WasmBuildError::CommandSpawn`].
/// Filesystem discovery and hashing are outside the probe deadline.
///
/// # Errors
/// Returns ordinary resolution errors or caller deadline/cancellation failures.
pub fn resolve_cargo_build_inputs_with_probe_policy(
    spec: &WasmBuildSpec,
    policy: &mut CargoInputProbePolicy<'_>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    validate_spec(spec)?;
    build_fingerprint_with_policy(spec, &mut ProgressReporter::silent(), &mut Some(policy))
}

/// Inspect one configured shared Cargo target under its build coordination lock.
///
/// Returns `None` without creating anything when the caller-owned target does
/// not exist. This operation never removes Cargo state.
///
/// # Errors
/// Returns target-boundary, coordination-lock, size-inspection or use-marker errors.
pub fn inspect_shared_incremental_target(
    spec: &WasmBuildSpec,
) -> Result<Option<SharedIncrementalTargetInspection>, WasmBuildError> {
    if !shared_incremental_target_exists(spec, "inspect shared incremental Cargo target")? {
        return Ok(None);
    }

    let (_lock, lock_wait, canonical) = lock_shared_incremental_target(spec)?;
    let logical_size_bytes =
        directory_logical_size(&canonical).map_err(|source| WasmBuildError::Io {
            operation: "measure shared incremental Cargo target",
            path: canonical.clone(),
            source,
        })?;
    let last_used = cache_entry_last_used(&canonical).map_err(|source| WasmBuildError::Io {
        operation: "read shared incremental Cargo target use time",
        path: canonical.clone(),
        source,
    })?;
    Ok(Some(SharedIncrementalTargetInspection {
        target_dir: canonical,
        logical_size_bytes,
        last_used,
        lock_wait,
    }))
}

/// Apply explicit whole-target retention to caller-owned shared Cargo state.
///
/// Returns `None` without creating anything when the target does not exist.
/// Policy evaluation and any clearing occur under the same cross-process lock
/// used by shared-incremental builds. The target root, `CACHEDIR.TAG`, and
/// `.ic-testkit` lock metadata are preserved, so another process cannot enter
/// through a replacement lock while maintenance is active.
/// Every other target child is removed when a limit is exceeded; unrelated
/// data that must survive must not be colocated there. Exact Cargo input
/// resolution first rejects targets overlapping source or configuration.
///
/// This function is never called automatically by exact Wasm acquisitions.
/// Consumers retain ownership of when mutable incremental state may be lost.
///
/// # Errors
/// Returns missing configuration, unsafe source/target overlap, input-resolution,
/// lock, inspection or target-removal errors.
pub fn maintain_shared_incremental_target(
    spec: &WasmBuildSpec,
    policy: CacheRetentionLimits,
) -> Result<Option<SharedIncrementalTargetMaintenance>, WasmBuildError> {
    if !shared_incremental_target_exists(
        spec,
        "inspect shared incremental Cargo target before maintenance",
    )? {
        return Ok(None);
    }

    // Reuse the exact build resolver so destructive maintenance cannot act on
    // a target that overlaps Cargo sources, configuration, or additional
    // inputs. The target itself is excluded as generated state during hashing.
    let _ = resolve_cargo_build_inputs(spec)?;
    let (_lock, lock_wait, canonical) = lock_shared_incremental_target(spec)?;
    maintain_shared_incremental_target_locked(&canonical, policy, lock_wait).map(Some)
}

/// Apply whole-target retention at most once per interval across processes.
///
/// The schedule marker is checked under the same lock used by shared Cargo
/// builds. A matching successful pass inside `minimum_interval` returns
/// [`SharedIncrementalTargetMaintenanceOutcome::Skipped`] without resolving
/// Cargo inputs or traversing the target. Missing targets are not created.
/// Changing the policy makes maintenance immediately due, and a zero interval
/// always evaluates retention.
///
/// Due maintenance performs exact Cargo input resolution before inspecting or
/// clearing the target. Failures are returned and are not recorded as a
/// successful pass, so an unsafe configuration cannot be hidden by the
/// schedule.
///
/// # Errors
/// Returns missing configuration, schedule, input-resolution, lock, inspection
/// or target-removal errors. Failed maintenance does not advance the schedule.
pub fn maintain_shared_incremental_target_at_most_every(
    spec: &WasmBuildSpec,
    policy: CacheRetentionLimits,
    minimum_interval: Duration,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    let target_dir =
        shared_incremental_target(spec).ok_or_else(|| WasmBuildError::InvalidSpec {
            message: "shared incremental target is not configured".to_owned(),
        })?;
    if !shared_incremental_target_exists(
        spec,
        "inspect shared incremental Cargo target before scheduled maintenance",
    )? {
        return Ok(SharedIncrementalTargetMaintenanceOutcome::Missing { target_dir });
    }

    let (_lock, lock_wait, canonical) = lock_shared_incremental_target(spec)?;
    let schedule = schedule_shared_incremental_target_maintenance(
        &canonical,
        policy,
        minimum_interval,
        lock_wait,
    )?;
    let schedule = match schedule {
        SharedIncrementalTargetMaintenanceSchedule::Skipped(outcome) => return Ok(outcome),
        SharedIncrementalTargetMaintenanceSchedule::Due(due) => due,
    };

    // Keep the schedule decision and maintenance in one critical section so
    // concurrent test binaries cannot all perform the same expensive scan.
    let _ = resolve_cargo_build_inputs(spec)?;
    perform_due_shared_incremental_target_maintenance(&canonical, policy, lock_wait, schedule)
}

enum SharedIncrementalTargetMaintenanceSchedule {
    Skipped(SharedIncrementalTargetMaintenanceOutcome),
    Due(DueSharedIncrementalTargetMaintenance),
}

struct DueSharedIncrementalTargetMaintenance {
    schedule_root: PathBuf,
    maintenance_identity: String,
    schedule_check: Duration,
}

fn schedule_shared_incremental_target_maintenance(
    canonical: &Path,
    policy: CacheRetentionLimits,
    minimum_interval: Duration,
    lock_wait: Duration,
) -> Result<SharedIncrementalTargetMaintenanceSchedule, WasmBuildError> {
    let schedule_root = canonical.join(".ic-testkit");
    let maintenance_identity = policy.maintenance_identity();
    let schedule_started = Instant::now();
    let due = cache_maintenance_due(&schedule_root, minimum_interval, &maintenance_identity)
        .map_err(wasm_cache_fs_error)?;
    let schedule_check = schedule_started.elapsed();
    if !due {
        return Ok(SharedIncrementalTargetMaintenanceSchedule::Skipped(
            SharedIncrementalTargetMaintenanceOutcome::Skipped {
                target_dir: canonical.to_owned(),
                lock_wait,
                schedule_check,
            },
        ));
    }
    Ok(SharedIncrementalTargetMaintenanceSchedule::Due(
        DueSharedIncrementalTargetMaintenance {
            schedule_root,
            maintenance_identity,
            schedule_check,
        },
    ))
}

fn perform_due_shared_incremental_target_maintenance(
    canonical: &Path,
    policy: CacheRetentionLimits,
    lock_wait: Duration,
    due: DueSharedIncrementalTargetMaintenance,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    let DueSharedIncrementalTargetMaintenance {
        schedule_root,
        maintenance_identity,
        schedule_check,
    } = due;
    let maintenance = maintain_shared_incremental_target_locked(canonical, policy, lock_wait)?;
    record_cache_maintenance(&schedule_root, &maintenance_identity).map_err(wasm_cache_fs_error)?;
    Ok(SharedIncrementalTargetMaintenanceOutcome::Performed {
        maintenance,
        schedule_check,
    })
}

fn maintain_shared_incremental_target_locked(
    canonical: &Path,
    policy: CacheRetentionLimits,
    lock_wait: Duration,
) -> Result<SharedIncrementalTargetMaintenance, WasmBuildError> {
    let started = Instant::now();
    let logical_size_bytes_before =
        directory_logical_size(canonical).map_err(|source| WasmBuildError::Io {
            operation: "measure shared incremental Cargo target before maintenance",
            path: canonical.to_owned(),
            source,
        })?;
    let last_used_before =
        cache_entry_last_used(canonical).map_err(|source| WasmBuildError::Io {
            operation: "read shared incremental Cargo target use time before maintenance",
            path: canonical.to_owned(),
            source,
        })?;
    let expired = policy.max_age().is_some_and(|max_age| {
        SystemTime::now()
            .duration_since(last_used_before)
            .is_ok_and(|age| age > max_age)
    });
    let oversized = policy
        .max_size_bytes()
        .is_some_and(|max_size_bytes| logical_size_bytes_before > max_size_bytes);
    let cleared = expired || oversized;
    if cleared {
        clear_shared_incremental_target_contents(canonical)?;
        record_cache_entry_use(canonical)?;
    }
    let logical_size_bytes_after = if cleared {
        directory_logical_size(canonical).map_err(|source| WasmBuildError::Io {
            operation: "measure shared incremental Cargo target after maintenance",
            path: canonical.to_owned(),
            source,
        })?
    } else {
        logical_size_bytes_before
    };
    Ok(SharedIncrementalTargetMaintenance {
        target_dir: canonical.to_owned(),
        logical_size_bytes_before,
        logical_size_bytes_after,
        last_used_before,
        cleared,
        lock_wait,
        maintenance: started.elapsed(),
    })
}

fn clear_shared_incremental_target_contents(target_dir: &Path) -> Result<(), WasmBuildError> {
    let entries = fs::read_dir(target_dir).map_err(|source| WasmBuildError::Io {
        operation: "read shared incremental Cargo target for maintenance",
        path: target_dir.to_owned(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| WasmBuildError::Io {
                operation: "read shared incremental Cargo target entry for maintenance",
                path: target_dir.to_owned(),
                source,
            })?
            .path();
        let preserved = path
            .file_name()
            .is_some_and(|name| name == ".ic-testkit" || name == "CACHEDIR.TAG");
        if !preserved {
            remove_path_if_present(&path).map_err(|source| WasmBuildError::Io {
                operation: "clear shared incremental Cargo target entry",
                path,
                source,
            })?;
        }
    }
    Ok(())
}

/// Build or reuse one exact set of Cargo Wasm artifacts.
///
/// The operation takes an exclusive process lock scoped to `target_dir`, then
/// fingerprints all declared inputs. A cache hit requires both a matching
/// atomic stamp and every expected nonempty Wasm output. Ordinary warm hits
/// revalidate inputs before returning and reject mutations during acquisition.
/// Explicit immutable-source prepared readers skip that warm
/// revalidation under their source-lease contract. Failed or interrupted
/// builds never publish a successful stamp.
///
/// `spawn` selects direct-child or owned-group custody and its cleanup policy,
/// independently of progress. Cargo starts with null stdin and piped stdout/
/// stderr; the spawner may override stdin. It must execute the supplied command
/// without changing its recipe. No signal handler or elapsed-time limit is added.
/// Probes retain their separate existing execution policy.
///
/// Use [`build_wasm_canisters_cached_with_progress`] for phase events and raw Cargo output.
///
/// A verified exact entry owns the bytes for its fingerprint. Warm acquisitions
/// repair public outputs and stamps to match it; matching independent writable
/// files owned by the effective user stay in place on Unix. If the exact entry
/// is missing or invalid, acquisition runs Cargo again instead of importing
/// mutable public outputs. A retained invalid entry cannot be replaced until
/// its last acquisition record releases it. Cold publication and non-Unix
/// materialization use atomic replacement. Callers must coordinate
/// other writers to mutable public destinations and treat retained paths as read-only.
///
/// # Errors
/// Returns invalid specification, input-resolution/change, coordination-lock,
/// Cargo execution, artifact verification or publication errors.
pub fn build_wasm_canisters_cached(
    spec: &WasmBuildSpec,
    mut spawn: impl FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    build_wasm_canisters_cached_internal(spec, &mut ProgressReporter::silent(), None, &mut spawn)
}

/// Build or reuse exact Wasm artifacts with caller-selected progress observation.
///
/// Execution, source checks and retention match [`build_wasm_canisters_cached`].
/// Observation never selects process custody or imposes a build deadline.
/// Observer panics clean up the active child with the caller's selected spawner.
///
/// # Errors
/// Returns invalid progress/specification, execution, verification or publication errors.
pub fn build_wasm_canisters_cached_with_progress<F>(
    spec: &WasmBuildSpec,
    config: WasmBuildProgressConfig,
    mut observer: F,
    mut spawn: impl FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<WasmBuildOutcome, WasmBuildError>
where
    F: FnMut(WasmBuildProgressEvent),
{
    build_wasm_canisters_cached_internal(
        spec,
        &mut ProgressReporter::new(Some((config, &mut observer)))?,
        None,
        &mut spawn,
    )
}

#[expect(
    clippy::result_large_err,
    reason = "Batch entries retain complete failure details inline without per-failure allocation."
)]
pub(super) fn build_wasm_canisters_cached_in_batch(
    spec: &WasmBuildSpec,
    index: usize,
    resolver: &mut WasmBuildBatchInputResolver<'_, '_>,
    observation: Option<(
        WasmBuildProgressConfig,
        &mut dyn FnMut(WasmBuildProgressEvent),
    )>,
    mut spawn: &mut dyn FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<WasmBuildOutcome, (WasmBuildError, WasmBuildFailureDetails)> {
    let mut progress = ProgressReporter::new(observation).map_err(|error| {
        (
            error,
            WasmBuildFailureDetails::specification(Duration::ZERO),
        )
    })?;
    let started = Instant::now();
    let result = build_wasm_canisters_cached_internal(
        spec,
        &mut progress,
        Some((resolver, index)),
        &mut spawn,
    );
    batch_result(result, &progress, resolver, started)
}

#[expect(
    clippy::result_large_err,
    reason = "Batch entries retain complete failure details inline without per-failure allocation."
)]
fn batch_result(
    result: Result<WasmBuildOutcome, WasmBuildError>,
    progress: &ProgressReporter<'_>,
    resolver: &WasmBuildBatchInputResolver<'_, '_>,
    started: Instant,
) -> Result<WasmBuildOutcome, (WasmBuildError, WasmBuildFailureDetails)> {
    if result
        .as_ref()
        .is_err_and(WasmBuildError::indicates_input_change)
    {
        resolver.invalidate_source_lease();
    }
    let total = started.elapsed();
    result.map_err(|error| {
        let details = progress.failure_details(&error, total);
        (error, details)
    })
}

fn batch_prepared_invalidation(
    batch_resolution: Option<&(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
) -> Option<Arc<RwLock<bool>>> {
    batch_resolution.and_then(|(resolver, _)| resolver.prepared_invalidation())
}

fn build_wasm_canisters_cached_internal(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
    mut batch_resolution: Option<(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
    mut spawn: &mut dyn FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let total_started = Instant::now();
    validate_spec(spec)?;
    let prepared_invalidation = batch_prepared_invalidation(batch_resolution.as_ref());
    progress.emit(WasmBuildProgressEvent::Started);
    if spec.shared_incremental_maintenance_config.is_some() {
        let outcome = build_wasm_canisters_cached_with_scheduled_shared_maintenance(
            spec,
            total_started,
            progress,
            batch_resolution.take(),
            &mut spawn,
        )?;
        emit_finished_progress(&outcome, progress);
        return Ok(outcome);
    }
    let (cache_lock, first_lock_wait) =
        lock_wasm_build_cache_with_progress(&spec.target_dir, progress)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    let resolved = resolve_initial_inputs(spec, batch_resolution.take(), progress)?;
    let isolated_acquisition = WasmAcquisitionContext::isolated(prepared_invalidation);
    if let Some(outcome) = try_reuse_wasm_artifacts(
        spec,
        &resolved,
        first_lock_wait,
        &isolated_acquisition,
        total_started,
        progress,
    )? {
        emit_finished_progress(&outcome, progress);
        return Ok(outcome);
    }
    progress.emit(WasmBuildProgressEvent::CacheMiss {
        fingerprint: resolved.fingerprint,
    });

    let outcome = match &spec.cache_mode {
        WasmBuildCacheMode::Isolated => {
            let cache_entry = cache_entry_directory(spec, resolved.fingerprint);
            build_wasm_cache_miss(
                spec,
                &resolved,
                first_lock_wait,
                isolated_acquisition,
                &cache_entry,
                total_started,
                progress,
                &mut spawn,
            )
        }
        WasmBuildCacheMode::SharedIncremental { .. } => {
            drop(cache_lock);
            build_wasm_with_shared_incremental(
                spec,
                resolved,
                first_lock_wait,
                isolated_acquisition,
                total_started,
                progress,
                &mut spawn,
            )
        }
    }?;
    emit_finished_progress(&outcome, progress);
    Ok(outcome)
}

fn build_wasm_with_shared_incremental(
    spec: &WasmBuildSpec,
    resolved: ResolvedCargoBuildInputs,
    first_lock_wait: Duration,
    source: WasmAcquisitionContext,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
    mut spawn: &mut dyn FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let (shared_lock, shared_lock_wait, shared_target) =
        lock_shared_incremental_target_with_progress(spec, progress)?;
    let (_cache_lock, second_lock_wait) =
        lock_wasm_build_cache_with_progress(&spec.target_dir, progress)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    let current = if source.prepared_invalidation.is_some() {
        resolved
    } else {
        let mut current = resolve_initial_inputs(spec, None, progress)?;
        current.timings.include(resolved.timings);
        current
    };
    let lock_wait = first_lock_wait.saturating_add(second_lock_wait);
    let shared_incremental =
        WasmAcquisitionContext::shared(shared_lock_wait, None, source.prepared_invalidation);
    if let Some(outcome) = try_reuse_wasm_artifacts(
        spec,
        &current,
        lock_wait,
        &shared_incremental,
        total_started,
        progress,
    )? {
        return Ok(outcome);
    }

    let outcome = build_wasm_cache_miss(
        spec,
        &current,
        lock_wait,
        shared_incremental,
        &shared_target,
        total_started,
        progress,
        &mut spawn,
    );
    drop(shared_lock);
    outcome
}

fn build_wasm_canisters_cached_with_scheduled_shared_maintenance(
    spec: &WasmBuildSpec,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
    batch_resolution: Option<(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
    mut spawn: &mut dyn FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let prepared_invalidation = batch_prepared_invalidation(batch_resolution.as_ref());
    let (_shared_lock, shared_lock_wait, shared_target) =
        lock_shared_incremental_target_with_progress(spec, progress)?;
    let (_cache_lock, lock_wait) = lock_wasm_build_cache_with_progress(&spec.target_dir, progress)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    // Resolution under both locks proves the target boundary once for the
    // scheduled retention pass and the following exact-cache acquisition.
    let resolved = resolve_initial_inputs(spec, batch_resolution, progress)?;
    let shared_maintenance = perform_configured_shared_incremental_target_maintenance(
        spec,
        &shared_target,
        shared_lock_wait,
        progress,
    )?;
    let shared_incremental = WasmAcquisitionContext::shared(
        shared_lock_wait,
        Some(shared_maintenance),
        prepared_invalidation,
    );
    if let Some(outcome) = try_reuse_wasm_artifacts(
        spec,
        &resolved,
        lock_wait,
        &shared_incremental,
        total_started,
        progress,
    )? {
        return Ok(outcome);
    }
    progress.emit(WasmBuildProgressEvent::CacheMiss {
        fingerprint: resolved.fingerprint,
    });
    build_wasm_cache_miss(
        spec,
        &resolved,
        lock_wait,
        shared_incremental,
        &shared_target,
        total_started,
        progress,
        &mut spawn,
    )
}

fn perform_configured_shared_incremental_target_maintenance(
    spec: &WasmBuildSpec,
    shared_target: &Path,
    lock_wait: Duration,
    progress: &mut ProgressReporter<'_>,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    let config = spec
        .shared_incremental_maintenance_config
        .expect("configured shared-target maintenance must have settings");
    progress.emit(WasmBuildProgressEvent::SharedTargetMaintenanceStarted {
        target_dir: shared_target.to_owned(),
    });
    let result = progress.run_phase(WasmBuildPhase::SharedTargetMaintenance, || {
        let schedule = schedule_shared_incremental_target_maintenance(
            shared_target,
            config.policy,
            config.minimum_interval,
            lock_wait,
        )?;
        match schedule {
            SharedIncrementalTargetMaintenanceSchedule::Skipped(outcome) => Ok(outcome),
            SharedIncrementalTargetMaintenanceSchedule::Due(due) => {
                perform_due_shared_incremental_target_maintenance(
                    shared_target,
                    config.policy,
                    lock_wait,
                    due,
                )
            }
        }
    });
    let outcome = integrated_shared_maintenance_result(config, shared_target, lock_wait, result)?;
    progress.emit(WasmBuildProgressEvent::SharedTargetMaintenanceFinished {
        outcome: outcome.clone(),
    });
    Ok(outcome)
}

fn integrated_shared_maintenance_result(
    config: SharedIncrementalTargetMaintenanceConfig,
    shared_target: &Path,
    lock_wait: Duration,
    result: Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError>,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    match result {
        Ok(outcome) => Ok(outcome),
        Err(error)
            if config.failure_mode == SharedIncrementalTargetMaintenanceFailureMode::BestEffort =>
        {
            Ok(SharedIncrementalTargetMaintenanceOutcome::Failed {
                target_dir: shared_target.to_owned(),
                lock_wait,
                message: error.to_string(),
            })
        }
        Err(error) => Err(error),
    }
}

fn resolve_initial_inputs(
    spec: &WasmBuildSpec,
    batch_resolution: Option<(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
    progress: &mut ProgressReporter<'_>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    let resolved = if let Some((resolver, index)) = batch_resolution {
        resolver.resolve(index, progress)?
    } else {
        build_fingerprint_with_progress(spec, progress)?
    };
    resolved.wasm_outputs().inspect_err(|_| {
        progress.begin_phase(WasmBuildPhase::InputDiscovery);
    })?;
    progress.emit(WasmBuildProgressEvent::InputsResolved {
        fingerprint: resolved.fingerprint,
        input_digest: resolved.input_digest,
        elapsed: resolved.timings.total,
    });
    Ok(resolved)
}

fn emit_finished_progress(outcome: &WasmBuildOutcome, progress: &mut ProgressReporter<'_>) {
    let state = if outcome.is_reused() {
        progress.emit(WasmBuildProgressEvent::CacheHit {
            fingerprint: outcome.record().fingerprint,
        });
        WasmBuildProgressOutcome::Reused
    } else {
        WasmBuildProgressOutcome::Built
    };
    progress.emit(WasmBuildProgressEvent::Finished {
        outcome: state,
        fingerprint: outcome.record().fingerprint,
        elapsed: outcome.record().timings.total,
    });
}

#[derive(Clone, Debug, Default)]
struct WasmAcquisitionContext {
    lock_wait: Option<Duration>,
    maintenance: Option<SharedIncrementalTargetMaintenanceOutcome>,
    prepared_invalidation: Option<Arc<RwLock<bool>>>,
}

impl WasmAcquisitionContext {
    fn isolated(prepared_invalidation: Option<Arc<RwLock<bool>>>) -> Self {
        Self {
            prepared_invalidation,
            ..Self::default()
        }
    }

    const fn shared(
        lock_wait: Duration,
        maintenance: Option<SharedIncrementalTargetMaintenanceOutcome>,
        prepared_invalidation: Option<Arc<RwLock<bool>>>,
    ) -> Self {
        Self {
            lock_wait: Some(lock_wait),
            maintenance,
            prepared_invalidation,
        }
    }

    fn lock_prepared_publication(
        &self,
    ) -> Result<Option<std::sync::RwLockReadGuard<'_, bool>>, WasmBuildError> {
        let guard = self.prepared_invalidation.as_deref().map(|invalidation| {
            invalidation
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        if guard.as_deref().is_some_and(|invalidated| *invalidated) {
            return Err(WasmBuildError::PreparedInputSnapshotInvalidated);
        }
        Ok(guard)
    }
}

fn try_reuse_wasm_artifacts(
    spec: &WasmBuildSpec,
    resolved: &ResolvedCargoBuildInputs,
    lock_wait: Duration,
    shared_incremental: &WasmAcquisitionContext,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
) -> Result<Option<WasmBuildOutcome>, WasmBuildError> {
    let targets = &resolved.wasm_outputs()?.targets;
    let _publication_guard = shared_incremental.lock_prepared_publication()?;
    let fingerprint = resolved.fingerprint;
    let cache_entry = cache_entry_directory(spec, fingerprint);
    let paths = artifact_publication_paths(spec, &cache_entry, targets);
    let Some(artifact_info) = progress.run_phase(WasmBuildPhase::ArtifactPublication, || {
        validated_artifact_set(paths, fingerprint)
    }) else {
        return Ok(None);
    };
    progress.run_phase(WasmBuildPhase::ArtifactPublication, || {
        materialize_artifacts(&artifact_info, fingerprint, true)?;
        record_cache_entry_use(&cache_entry)
    })?;
    let input_resolution = validate_reused_inputs(spec, resolved, shared_incremental, progress)?;
    Ok(Some(WasmBuildOutcome::Reused(complete_build_record(
        spec,
        BuildRecordInput {
            resolved,
            lock_wait,
            shared_incremental: shared_incremental.clone(),
            input_resolution,
            cargo_build: None,
            active_entry: &cache_entry,
            targets,
        },
        total_started,
        progress,
    )?)))
}

fn validate_reused_inputs(
    spec: &WasmBuildSpec,
    resolved: &ResolvedCargoBuildInputs,
    context: &WasmAcquisitionContext,
    progress: &mut ProgressReporter<'_>,
) -> Result<WasmInputResolutionTimings, WasmBuildError> {
    let mut timings = resolved.timings;
    if context.prepared_invalidation.is_none() {
        let verified = verify_resolved_inputs(spec, resolved, progress)?;
        timings.include(verified.timings);
    }
    Ok(timings)
}

fn verify_resolved_inputs(
    spec: &WasmBuildSpec,
    resolved: &ResolvedCargoBuildInputs,
    progress: &mut ProgressReporter<'_>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    let verified = resolve_initial_inputs(spec, None, progress)?;
    for (before, after) in [
        (resolved.validation_digest, verified.validation_digest),
        (resolved.fingerprint, verified.fingerprint),
    ] {
        if before != after {
            return Err(WasmBuildError::InputsChangedDuringAcquisition { before, after });
        }
    }
    Ok(verified)
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep the caller's spawner separate from recipe, acquisition and progress state"
)]
fn build_wasm_cache_miss(
    spec: &WasmBuildSpec,
    resolved: &ResolvedCargoBuildInputs,
    lock_wait: Duration,
    shared_incremental: WasmAcquisitionContext,
    cargo_target_dir: &Path,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
    mut spawn: &mut dyn FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let targets = &resolved.wasm_outputs()?.targets;
    let fingerprint = resolved.fingerprint;
    let mut input_resolution = resolved.timings;
    let cache_entry = cache_entry_directory(spec, fingerprint);
    let preparation_started = Instant::now();
    progress.begin_phase(WasmBuildPhase::ArtifactPublication);
    let preparation_result = prepare_exact_cache_entry(&cache_entry);
    progress.record_phase(
        WasmBuildPhase::ArtifactPublication,
        preparation_started.elapsed(),
    );
    let incomplete_directory = preparation_result?;
    let build_result = (|| {
        if matches!(
            spec.cache_mode,
            WasmBuildCacheMode::SharedIncremental { .. }
        ) {
            record_cache_entry_use(cargo_target_dir)?;
        }
        let build_started = Instant::now();
        progress.begin_phase(WasmBuildPhase::CargoBuild);
        let cargo_result = run_cargo_build(spec, cargo_target_dir, targets, progress, &mut spawn);
        let cargo_build = build_started.elapsed();
        progress.record_phase(WasmBuildPhase::CargoBuild, cargo_build);
        let built_artifacts = cargo_result?;
        let validation_started = Instant::now();
        progress.begin_phase(WasmBuildPhase::ArtifactPublication);
        let missing = missing_artifacts(&built_artifacts);
        progress.record_phase(
            WasmBuildPhase::ArtifactPublication,
            validation_started.elapsed(),
        );
        if !missing.is_empty() {
            return Err(WasmBuildError::MissingArtifacts { paths: missing });
        }

        let verified = verify_resolved_inputs(spec, resolved, progress)?;
        input_resolution.include(verified.timings);

        // Publication is the prepared snapshot's linearization boundary. A
        // reader that reaches it first may finish publishing; invalidation
        // takes the write side of this lock and therefore precedes every later
        // reader without racing a successful stamp into existence.
        let publication_guard = shared_incremental.lock_prepared_publication()?;

        let paths = artifact_publication_paths(spec, &cache_entry, targets);
        progress.run_phase(WasmBuildPhase::ArtifactPublication, || {
            copy_wasm_artifacts(&built_artifacts, &paths)?;
            let artifacts = publish_artifact_stamps(paths, fingerprint)?;
            materialize_artifacts(&artifacts, fingerprint, false)?;
            record_cache_entry_use(&cache_entry)
        })?;
        drop(publication_guard);

        Ok(WasmBuildOutcome::Built(complete_build_record(
            spec,
            BuildRecordInput {
                resolved,
                lock_wait,
                shared_incremental,
                input_resolution,
                cargo_build: Some(cargo_build),
                active_entry: &cache_entry,
                targets,
            },
            total_started,
            progress,
        )?))
    })();
    finish_fingerprint_build(build_result, incomplete_directory, progress)
}

/// Prune fingerprint-specific Cargo target directories under `target_dir`.
///
/// Pruning uses the same exclusive process lock as builds. Entries older than
/// the configured age are removed first, then least-recently-used entries are
/// removed until the configured logical byte limit is met. Only direct child
/// directories with SHA-256 fingerprint names are eligible; caller-facing
/// artifacts and unrelated target contents are never removed.
/// Entries owned by live build records are skipped until their last owner drops.
///
/// # Errors
/// Returns cache-lock, inspection or filesystem removal errors.
pub fn prune_wasm_build_cache(
    target_dir: &Path,
    policy: CacheRetentionLimits,
) -> Result<ArtifactCachePruneReport, WasmBuildError> {
    let (_lock_file, _) = lock_wasm_build_cache(target_dir)?;
    ensure_cache_directory_tag(target_dir)?;

    prune_wasm_build_cache_locked(target_dir, policy, None)
}

struct BuildRecordInput<'a> {
    resolved: &'a ResolvedCargoBuildInputs,
    lock_wait: Duration,
    shared_incremental: WasmAcquisitionContext,
    input_resolution: WasmInputResolutionTimings,
    cargo_build: Option<Duration>,
    active_entry: &'a Path,
    targets: &'a [WasmArtifactTarget],
}

fn complete_build_record(
    spec: &WasmBuildSpec,
    input: BuildRecordInput<'_>,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
) -> Result<WasmBuildRecord, WasmBuildError> {
    let retention = progress.run_phase(WasmBuildPhase::ArtifactPublication, || {
        RetainedCacheEntry::acquire(input.active_entry).map_err(wasm_cache_fs_error)
    })?;
    let (maintenance, cache_maintenance) =
        spec.pruning.map_or((None, None), |(policy, interval)| {
            progress.run_phase(WasmBuildPhase::ExactCacheMaintenance, || {
                let cache_root = spec.target_dir.join(".ic-testkit/wasm-targets");
                let identity = policy.maintenance_identity();
                perform_scheduled_cache_maintenance(&cache_root, interval, &identity, || {
                    prune_wasm_build_cache_locked(
                        &spec.target_dir,
                        policy,
                        Some(input.active_entry),
                    )
                    .map_err(|error| error.to_string())
                })
            })
        });
    Ok(WasmBuildRecord {
        facts: input.resolved.facts.clone(),
        fingerprint: input.resolved.fingerprint,
        input_digest: input.resolved.input_digest,
        artifacts: expected_artifacts(spec, input.active_entry, input.targets).into(),
        retention,
        timings: WasmBuildTimings {
            lock_wait: input.lock_wait,
            shared_incremental_lock_wait: input.shared_incremental.lock_wait,
            input_resolution: input.input_resolution,
            cargo_build: input.cargo_build,
            cache_maintenance,
            total: total_started.elapsed(),
        },
        maintenance,
        shared_incremental_maintenance: input.shared_incremental.maintenance,
    })
}

fn prune_wasm_build_cache_locked(
    target_dir: &Path,
    policy: CacheRetentionLimits,
    protected_entry: Option<&Path>,
) -> Result<ArtifactCachePruneReport, WasmBuildError> {
    let cache_root = target_dir.join(".ic-testkit/wasm-targets");
    prune_direct_child_directories(&cache_root, policy, protected_entry, is_sha256_directory)
        .map_err(wasm_cache_fs_error)
}

fn finish_fingerprint_build<T>(
    result: Result<T, WasmBuildError>,
    incomplete_directory: OwnedDirectory,
    progress: &mut ProgressReporter<'_>,
) -> Result<T, WasmBuildError> {
    match result {
        Ok(outcome) => {
            incomplete_directory.preserve();
            Ok(outcome)
        }
        Err(build_error) => Err(cleanup_failed_fingerprint_build(
            build_error,
            incomplete_directory,
            progress,
        )),
    }
}

fn cleanup_failed_fingerprint_build(
    build_error: WasmBuildError,
    incomplete_directory: OwnedDirectory,
    progress: &mut ProgressReporter<'_>,
) -> WasmBuildError {
    let path = incomplete_directory.path().to_owned();
    let primary_phase = progress.failure_phase;
    let cleanup_started = Instant::now();
    let cleanup = incomplete_directory.cleanup();
    progress.record_phase(WasmBuildPhase::Cleanup, cleanup_started.elapsed());
    match cleanup {
        Ok(()) => {
            progress.failure_phase = primary_phase;
            build_error
        }
        Err(source) => WasmBuildError::FailedBuildCleanup {
            build_error: Box::new(build_error),
            path,
            source,
        },
    }
}

fn lock_wasm_build_cache(target_dir: &Path) -> Result<(File, Duration), WasmBuildError> {
    create_dir_all(target_dir, "create Cargo target directory")?;
    let lock_path = target_dir.join(".ic-testkit/wasm-build.lock");
    lock_cache_file(&lock_path).map_err(wasm_cache_fs_error)
}

fn lock_wasm_build_cache_with_progress(
    target_dir: &Path,
    progress: &mut ProgressReporter<'_>,
) -> Result<(File, Duration), WasmBuildError> {
    progress.begin_phase(WasmBuildPhase::ExactCacheCoordination);
    create_dir_all(target_dir, "create Cargo target directory")?;
    let lock_path = target_dir.join(".ic-testkit/wasm-build.lock");
    lock_cache_file_with_progress(&lock_path, WasmBuildPhase::ExactCacheCoordination, progress)
}

fn lock_shared_incremental_target(
    spec: &WasmBuildSpec,
) -> Result<(File, Duration, PathBuf), WasmBuildError> {
    lock_shared_incremental_target_internal(spec, None)
}

fn lock_shared_incremental_target_with_progress(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
) -> Result<(File, Duration, PathBuf), WasmBuildError> {
    let target_dir = shared_incremental_target(spec)
        .expect("validated shared acquisition must have a shared Cargo target");
    progress.emit(WasmBuildProgressEvent::SharedTargetLockStarted { target_dir });
    let (lock, wait, canonical) = lock_shared_incremental_target_internal(spec, Some(progress))?;
    progress.emit(WasmBuildProgressEvent::SharedTargetLockAcquired {
        target_dir: canonical.clone(),
        wait,
    });
    Ok((lock, wait, canonical))
}

fn lock_shared_incremental_target_internal(
    spec: &WasmBuildSpec,
    mut progress: Option<&mut ProgressReporter<'_>>,
) -> Result<(File, Duration, PathBuf), WasmBuildError> {
    if let Some(progress) = progress.as_deref_mut() {
        progress.begin_phase(WasmBuildPhase::SharedTargetCoordination);
    }
    let target_dir =
        shared_incremental_target(spec).ok_or_else(|| WasmBuildError::InvalidSpec {
            message: "shared incremental target is not configured".to_owned(),
        })?;
    create_dir_all(
        &target_dir,
        "create shared incremental Cargo target directory",
    )?;
    ensure_cache_tag(&target_dir).map_err(wasm_cache_fs_error)?;
    let canonical = target_dir
        .canonicalize()
        .map_err(|source| WasmBuildError::Io {
            operation: "resolve shared incremental Cargo target directory",
            path: target_dir.clone(),
            source,
        })?;
    let lock_path = canonical.join(".ic-testkit/wasm-incremental.lock");
    let (lock, wait) = if let Some(progress) = progress {
        lock_cache_file_with_progress(
            &lock_path,
            WasmBuildPhase::SharedTargetCoordination,
            progress,
        )?
    } else {
        lock_cache_file(&lock_path).map_err(wasm_cache_fs_error)?
    };
    Ok((lock, wait, canonical))
}

fn lock_cache_file_with_progress(
    lock_path: &Path,
    phase: WasmBuildPhase,
    progress: &mut ProgressReporter<'_>,
) -> Result<(File, Duration), WasmBuildError> {
    let started = Instant::now();
    progress.begin_phase(phase);
    let result = if !progress.is_observed() || progress.config.heartbeat_interval.is_none() {
        lock_cache_file(lock_path).map_err(wasm_cache_fs_error)
    } else {
        let heartbeat_interval = progress
            .config
            .heartbeat_interval
            .expect("observed cache lock must have a heartbeat interval");
        lock_cache_file_with_wait_observer(lock_path, heartbeat_interval, |elapsed| {
            progress.emit_heartbeat_if_due(phase, elapsed);
        })
        .map_err(wasm_cache_fs_error)
    };
    progress.record_phase(phase, started.elapsed());
    result
}

fn ensure_cache_directory_tag(target_dir: &Path) -> Result<(), WasmBuildError> {
    ensure_cache_tag(target_dir).map_err(wasm_cache_fs_error)
}

fn record_cache_entry_use(path: &Path) -> Result<(), WasmBuildError> {
    record_entry_use(path).map_err(wasm_cache_fs_error)
}

fn wasm_cache_fs_error(error: CacheFsError) -> WasmBuildError {
    WasmBuildError::Io {
        operation: error.operation,
        path: error.path,
        source: error.source,
    }
}

fn validate_spec(spec: &WasmBuildSpec) -> Result<(), WasmBuildError> {
    for (key, value) in &spec.environment {
        let key = key.as_encoded_bytes();
        if key.is_empty()
            || key.iter().any(|byte| matches!(byte, b'=' | 0))
            || value.as_encoded_bytes().contains(&0)
        {
            return Err(WasmBuildError::InvalidSpec {
                message: "invalid child environment name or value".to_owned(),
            });
        }
    }
    if spec.cargo_message_bytes == 0 {
        return Err(WasmBuildError::InvalidSpec {
            message: "Cargo message allowance must be nonzero".to_owned(),
        });
    }
    if spec.packages.is_empty() {
        return Err(WasmBuildError::InvalidSpec {
            message: "at least one Cargo package is required".to_owned(),
        });
    }
    if spec.target.is_empty() {
        return Err(WasmBuildError::InvalidSpec {
            message: "Cargo compilation target must not be empty".to_owned(),
        });
    }
    if spec
        .environment
        .contains_key(OsStr::new("CARGO_TARGET_DIR"))
    {
        return Err(WasmBuildError::InvalidSpec {
            message: "Cargo target directories are owned by the build specification; use target_dir or with_shared_incremental_target instead of command overrides".to_owned(),
        });
    }
    validate_cargo_target_selection(spec)?;
    spec.cargo_selection
        .profile_target_dir()
        .map_err(|message| WasmBuildError::InvalidSpec {
            message: message.to_owned(),
        })?;
    if matches!(
        &spec.cache_mode,
        WasmBuildCacheMode::SharedIncremental { target_dir } if target_dir.as_os_str().is_empty()
    ) {
        return Err(WasmBuildError::InvalidSpec {
            message: "shared incremental Cargo target directory must not be empty".to_owned(),
        });
    }
    if spec.shared_incremental_maintenance_config.is_some()
        && !matches!(
            spec.cache_mode,
            WasmBuildCacheMode::SharedIncremental { .. }
        )
    {
        return Err(WasmBuildError::InvalidSpec {
            message:
                "scheduled shared-target maintenance requires a shared incremental Cargo target"
                    .to_owned(),
        });
    }
    Ok(())
}

fn validate_cargo_target_selection(spec: &WasmBuildSpec) -> Result<(), WasmBuildError> {
    if let Some(example) = &spec.example {
        let mut components = Path::new(example).components();
        if spec.packages.len() != 1
            || !matches!(
                (components.next(), components.next()),
                (Some(Component::Normal(_)), None)
            )
        {
            return Err(WasmBuildError::InvalidSpec {
                message: "a named Cargo example requires one package, a normal target name"
                    .to_owned(),
            });
        }
    }
    if spec.packages.iter().any(|package| {
        let mut components = Path::new(package).components();
        !matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(_)), None)
        )
    }) {
        return Err(WasmBuildError::InvalidSpec {
            message: "Cargo package names must be normal path components".to_owned(),
        });
    }
    Ok(())
}

fn build_fingerprint(spec: &WasmBuildSpec) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    build_fingerprint_with_progress(spec, &mut ProgressReporter::silent())
}

fn build_fingerprint_with_progress(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    build_fingerprint_with_policy(spec, progress, &mut None)
}

fn build_fingerprint_with_policy(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
    policy: &mut Option<&mut CargoInputProbePolicy<'_>>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    let total_started = Instant::now();
    let (cargo_identity, rustc_identity, tool_identity) =
        resolve_tool_identity_with_policy(spec, progress, policy)?;

    let metadata_started = Instant::now();
    let metadata = progress.run_phase(WasmBuildPhase::CargoMetadata, || {
        cargo_metadata_with_policy(spec, policy)
    })?;
    let cargo_metadata = metadata_started.elapsed();

    let discovery_started = Instant::now();
    let (inputs, exclusions) = progress.run_phase(WasmBuildPhase::InputDiscovery, || {
        let parsed =
            ParsedCargoMetadata::parse(&metadata).map_err(|message| invalid_metadata(&message))?;
        resolve_local_inputs(spec, &parsed)
    })?;
    let input_discovery = discovery_started.elapsed();

    let hashing_started = Instant::now();
    let (input_digest, validation_digest) =
        progress.run_phase(WasmBuildPhase::ContentHashing, || {
            let mut cache = LabeledPathDigestCache::default();
            digest_resolved_local_inputs(
                &inputs,
                &exclusions,
                &mut cache,
                &spec.workspace_root,
                "hash Wasm build inputs",
                "hash semantic Wasm build inputs",
            )
        })?;
    let content_hashing = hashing_started.elapsed();

    Ok(ResolvedCargoBuildInputs::assemble(
        spec,
        inputs,
        exclusions,
        input_digest,
        validation_digest,
        CargoBuildFacts::observed(spec, cargo_identity, rustc_identity),
        || WasmInputResolutionTimings {
            tool_identity,
            cargo_metadata,
            input_discovery,
            content_hashing,
            total: total_started.elapsed(),
        },
    ))
}

fn cargo_build_inputs(inputs: Vec<(PathBuf, PathBuf)>) -> Arc<[CargoBuildInput]> {
    inputs
        .into_iter()
        .map(|(label, path)| CargoBuildInput { label, path })
        .collect()
}

fn finish_build_fingerprint(
    spec: &WasmBuildSpec,
    facts: &CargoBuildFacts,
    input_digest: InputDigest,
) -> InputDigest {
    let mut hasher = InputHasher::new(BUILD_FINGERPRINT_DOMAIN);
    for package in &spec.packages {
        hasher.field("package", package.as_bytes());
    }
    hasher.field("target", facts.target.as_bytes());
    if let Some(example) = &spec.example {
        hasher.field("cargo-target-selection", b"example");
        hasher.field("cargo-example", example.as_bytes());
    } else {
        hasher.field("cargo-target-selection", b"lib");
    }
    hasher.field(
        "profile-target-dir",
        facts
            .selection
            .profile_target_dir()
            .expect("Cargo profile was validated before fingerprinting")
            .as_bytes(),
    );
    for argument in &facts.selection.build_arguments() {
        hasher.field("cargo-argument", &os_bytes(argument));
    }
    for (key, value) in &facts.environment {
        hasher.field("environment-key", &os_bytes(key));
        hasher.field("environment-value", &os_bytes(value));
    }
    hasher.field("cargo-identity", &facts.cargo_identity);
    hasher.field("rustc-identity", &facts.rustc_identity);
    hasher.field("source-input-digest", input_digest.as_bytes());
    hasher.finish()
}

#[cfg(test)]
fn command_identity(
    spec: &WasmBuildSpec,
    phase: WasmBuildPhase,
    program: &OsStr,
    arguments: &[&str],
) -> Result<Vec<u8>, WasmBuildError> {
    command_identity_with_policy(spec, phase, program, arguments, &mut None)
}

fn command_identity_with_policy(
    spec: &WasmBuildSpec,
    phase: WasmBuildPhase,
    program: &OsStr,
    arguments: &[&str],
    policy: &mut Option<&mut CargoInputProbePolicy<'_>>,
) -> Result<Vec<u8>, WasmBuildError> {
    let mut command = Command::new(program);
    command.current_dir(&spec.workspace_root).args(arguments);
    apply_command_environment(&mut command, spec);
    let output =
        capture_probe_output_with_policy(&mut command, phase, TOOL_IDENTITY_BYTES, policy)?;
    ensure_command_success(phase, output).map(|output| {
        let mut identity = output.stdout;
        identity.extend_from_slice(&output.stderr);
        identity
    })
}

fn cargo_metadata(spec: &WasmBuildSpec) -> Result<schema::Metadata, WasmBuildError> {
    cargo_metadata_with_policy(spec, &mut None)
}

fn cargo_metadata_with_policy(
    spec: &WasmBuildSpec,
    policy: &mut Option<&mut CargoInputProbePolicy<'_>>,
) -> Result<schema::Metadata, WasmBuildError> {
    let mut command = Command::new(&spec.cargo_program);
    command
        .current_dir(&spec.workspace_root)
        .args(["metadata", "--format-version", "1"]);
    for argument in spec.cargo_selection.metadata_arguments() {
        command.arg(argument);
    }
    apply_command_environment(&mut command, spec);
    let output = capture_probe_output_with_policy(
        &mut command,
        WasmBuildPhase::CargoMetadata,
        CARGO_METADATA_BYTES,
        policy,
    )?;
    let output = ensure_command_success(WasmBuildPhase::CargoMetadata, output)?;
    crate::cargo::observation::decode_metadata(&output.stdout).map_err(|error| {
        WasmBuildError::InvalidMetadata {
            message: format!("Cargo metadata did not match its schema: {error}"),
        }
    })
}

#[cfg(test)]
fn capture_probe_output(
    command: &mut Command,
    phase: WasmBuildPhase,
    stdout_bytes: usize,
) -> Result<CompletedExecution, WasmBuildError> {
    capture_probe_output_with_policy(command, phase, stdout_bytes, &mut None)
}

fn capture_probe_output_with_policy(
    command: &mut Command,
    phase: WasmBuildPhase,
    stdout_bytes: usize,
    policy: &mut Option<&mut CargoInputProbePolicy<'_>>,
) -> Result<CompletedExecution, WasmBuildError> {
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(stdout_bytes),
        stderr: OutputLimit::Terminate(TOOL_IDENTITY_BYTES),
        timeout: None,
    };
    let result = if let Some(policy) = policy {
        capture_controlled_probe(command, limits, policy)
    } else {
        capture_command(command, limits)
    }
    .and_then(CompletedExecution::require_complete);
    let evidence = match result {
        Ok(evidence) => evidence,
        Err(ToolError::Execution(error))
            if matches!(
                error.failure,
                ExecutionFailure::Io {
                    operation: ExecutionOperation::Spawn,
                    ..
                }
            ) && error.cleanup.is_none() =>
        {
            let ExecutionFailure::Io { source, .. } = error.failure else {
                unreachable!()
            };
            return Err(WasmBuildError::CommandSpawn {
                phase,
                program: command.get_program().to_owned(),
                source,
            });
        }
        Err(error) => {
            return Err(WasmBuildError::Io {
                operation: match phase {
                    WasmBuildPhase::CargoIdentity => "capture complete Cargo identity probe output",
                    WasmBuildPhase::RustcIdentity => "capture complete rustc identity probe output",
                    _ => "capture complete Cargo metadata probe output",
                },
                path: PathBuf::from(command.get_program()),
                source: io::Error::other(error),
            });
        }
    };
    Ok(evidence)
}

fn capture_controlled_probe(
    command: &mut Command,
    mut limits: OutputLimits,
    policy: &mut CargoInputProbePolicy<'_>,
) -> Result<CompletedExecution, ToolError> {
    let failure = if (policy.cancelled)() {
        Some(ExecutionFailure::Cancelled)
    } else if policy
        .deadline
        .is_some_and(|deadline| deadline <= Instant::now())
    {
        Some(ExecutionFailure::TimedOut)
    } else {
        None
    };
    if let Some(failure) = failure {
        return Err(ToolError::Execution(Box::new(ExecutionError {
            failure,
            evidence: ExecutionEvidence::default(),
            cleanup: None,
        })));
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = OwnedChild::spawn(command, policy.cleanup).map_err(|source| {
        ToolError::Execution(Box::new(ExecutionError {
            failure: ExecutionFailure::Io {
                operation: ExecutionOperation::Spawn,
                source,
            },
            evidence: ExecutionEvidence::default(),
            cleanup: None,
        }))
    })?;
    // The deadline is shared across probes, not restarted at each command.
    limits.timeout = policy.deadline.map(|deadline| {
        deadline
            .saturating_duration_since(Instant::now())
            .max(Duration::from_nanos(1))
    });
    communicate_child(
        &mut child,
        None,
        limits,
        SuccessfulExit::Cleanup,
        &mut *policy.cancelled,
    )
}

// Parsed once per Cargo resolution context. Each specification still selects
// its own closure and independently validates filesystem inputs.
struct ParsedCargoMetadata<'a> {
    packages: HashMap<&'a str, &'a schema::Package>,
    workspace_packages: HashMap<&'a str, Option<&'a str>>,
    nodes: HashMap<&'a str, &'a schema::Node>,
    workspace_root: &'a Path,
}

impl<'a> ParsedCargoMetadata<'a> {
    fn parse(metadata: &'a schema::Metadata) -> Result<Self, String> {
        let packages: HashMap<_, _> = metadata
            .packages
            .iter()
            .map(|package| (package.id.repr.as_str(), package))
            .collect();
        let workspace_members: HashSet<_> = metadata
            .workspace_members
            .iter()
            .map(|id| id.repr.as_str())
            .collect();
        let mut workspace_packages = HashMap::new();
        for package in packages
            .values()
            .filter(|package| workspace_members.contains(package.id.repr.as_str()))
        {
            workspace_packages
                .entry(package.name.as_str())
                .and_modify(|id| *id = None)
                .or_insert(Some(package.id.repr.as_str()));
        }
        let resolve = metadata
            .resolve
            .as_ref()
            .ok_or_else(|| "Cargo metadata has no resolved dependency nodes".to_owned())?;
        let nodes = resolve
            .nodes
            .iter()
            .map(|node| (node.id.repr.as_str(), node))
            .collect();
        Ok(Self {
            packages,
            workspace_packages,
            nodes,
            workspace_root: metadata.workspace_root.as_std_path(),
        })
    }
}

struct LockedPackageIdentity {
    name: String,
    version: String,
    source: String,
    checksum: Option<String>,
}

type LockedPackageIndex<'a> = HashMap<(&'a str, &'a str, &'a str), &'a LockedPackageIdentity>;

fn index_locked_packages(packages: &[LockedPackageIdentity]) -> LockedPackageIndex<'_> {
    let mut index = HashMap::with_capacity(packages.len());
    for package in packages {
        // Preserve the first matching row, including an absent checksum.
        index
            .entry((
                package.name.as_str(),
                package.version.as_str(),
                package.source.as_str(),
            ))
            .or_insert(package);
    }
    index
}

fn resolve_local_inputs(
    spec: &WasmBuildSpec,
    metadata: &ParsedCargoMetadata<'_>,
) -> Result<(ResolvedLocalInputs, Vec<PathBuf>), WasmBuildError> {
    let mut selected_ids = selected_package_ids(spec, metadata)?;
    // Cargo snapshots also serve generic transactions; defer this acquisition
    // constraint until the resolved snapshot is used to acquire Wasm artifacts.
    let mut artifact_names = BTreeSet::new();
    let wasm_targets = selected_ids
        .iter()
        .map(|id| {
            let package = metadata.packages[id];
            let mut matching = package.targets.iter().filter(|target| {
                target.crate_types.contains(&schema::CrateType::CDyLib)
                    && spec.example.as_ref().map_or_else(
                        || target.is_cdylib(),
                        |example| target.name == *example && target.is_example(),
                    )
            });
            let Some(target) = matching.next() else {
                return Err(format!(
                    "Cargo package `{}` must declare the selected cdylib target",
                    package.name
                ));
            };
            let mut components = Path::new(&target.name).components();
            if matching.next().is_some()
                || !matches!(
                    (components.next(), components.next()),
                    (Some(Component::Normal(_)), None)
                )
            {
                return Err(format!(
                    "Cargo package `{}` has an ambiguous or unsafe cdylib target",
                    package.name
                ));
            }
            let cache_name = target.name.replace('-', "_");
            if !artifact_names.insert(cache_name.clone()) {
                return Err(format!(
                    "selected Cargo packages share the Wasm artifact name `{cache_name}`"
                ));
            }
            Ok(WasmArtifactTarget {
                package_id: package.id.repr.clone(),
                name: target.name.clone(),
                kind: if spec.example.is_some() {
                    schema::TargetKind::Example
                } else {
                    schema::TargetKind::CDyLib
                },
            })
        })
        .collect();
    let mut closure = BTreeSet::new();
    while let Some(id) = selected_ids.pop_front() {
        if !closure.insert(id) {
            continue;
        }
        if let Some(node) = metadata.nodes.get(id) {
            selected_ids.extend(
                node.deps
                    .iter()
                    .map(|dependency| dependency.pkg.repr.as_str()),
            );
        }
    }

    let workspace_root = metadata.workspace_root;
    let workspace_projection = semantic_workspace_projection(metadata, &closure, workspace_root)?;
    let mut validation_inputs = workspace_configuration_inputs(spec, workspace_root)?;
    append_package_inputs(
        &mut validation_inputs,
        &metadata.packages,
        closure,
        workspace_root,
    )?;
    append_additional_inputs(&mut validation_inputs, spec, workspace_root);
    validate_shared_incremental_target_boundary(spec, &validation_inputs)?;
    let exclusions = source_exclusions(spec, &validation_inputs);
    Ok((
        ResolvedLocalInputs {
            validation_inputs,
            workspace_projection,
            wasm_targets,
        },
        exclusions,
    ))
}

fn selected_package_ids<'a>(
    spec: &WasmBuildSpec,
    metadata: &ParsedCargoMetadata<'a>,
) -> Result<VecDeque<&'a str>, WasmBuildError> {
    let mut selected_ids = VecDeque::new();
    for requested in &spec.packages {
        match metadata.workspace_packages.get(requested.as_str()) {
            Some(Some(id)) => selected_ids.push_back(*id),
            None => {
                return Err(WasmBuildError::InvalidSpec {
                    message: format!("Cargo workspace contains no package named `{requested}`"),
                });
            }
            Some(None) => {
                return Err(WasmBuildError::InvalidSpec {
                    message: format!("Cargo workspace package name `{requested}` is ambiguous"),
                });
            }
        }
    }
    Ok(selected_ids)
}

fn semantic_workspace_projection(
    metadata: &ParsedCargoMetadata<'_>,
    closure: &BTreeSet<&str>,
    workspace_root: &Path,
) -> Result<Option<InputDigest>, WasmBuildError> {
    // A workspace-root or external local package cannot be separated from the
    // broad root safely; `None` keeps the complete-input fingerprint.
    let locked_packages = locked_package_identities(workspace_root)?;
    let locked_index = index_locked_packages(&locked_packages);
    let mut identities = HashMap::new();
    for &id in closure {
        let package = metadata
            .packages
            .get(id)
            .ok_or_else(|| invalid_metadata(&format!("resolved package `{id}` is missing")))?;
        let Some(identity) = semantic_package_identity(package, workspace_root, &locked_index)
        else {
            return Ok(None);
        };
        identities.insert(id, identity);
    }

    let mut projected_packages = closure
        .iter()
        .map(|&id| {
            let package = metadata
                .packages
                .get(id)
                .expect("selected package closure was validated above");
            let identity = identities[id];
            let node = metadata.nodes.get(id).ok_or_else(|| {
                invalid_metadata(&format!("resolved package `{id}` has no dependency node"))
            })?;
            let projection = semantic_package_projection(package, node, &identities)?;
            Ok::<_, WasmBuildError>((identity, projection))
        })
        .collect::<Result<Vec<_>, _>>()?;
    projected_packages.sort_by_key(|(identity, _)| *identity);

    let root_manifest = workspace_root.join("Cargo.toml");
    let root_contents =
        fs::read_to_string(&root_manifest).map_err(|source| WasmBuildError::Io {
            operation: "read workspace manifest for semantic projection",
            path: root_manifest.clone(),
            source,
        })?;
    let root = toml::from_str::<TomlValue>(&root_contents).map_err(|error| {
        invalid_metadata(&format!(
            "workspace manifest could not be projected as TOML: {error}"
        ))
    })?;

    let mut hasher = InputHasher::new("wasm-semantic-workspace-projection-v1");
    for (identity, projection) in projected_packages {
        hasher.field("package-identity", identity.as_bytes());
        hasher.field("package-projection", projection.as_bytes());
    }
    hash_toml_setting(&mut hasher, "cargo-features", root.get("cargo-features"));
    hash_toml_setting(&mut hasher, "profile", root.get("profile"));
    let workspace = root.get("workspace").and_then(TomlValue::as_table);
    hash_toml_setting(
        &mut hasher,
        "workspace-resolver",
        workspace.and_then(|table| table.get("resolver")),
    );
    hash_toml_setting(
        &mut hasher,
        "workspace-lints",
        workspace.and_then(|table| table.get("lints")),
    );
    Ok(Some(hasher.finish()))
}

fn locked_package_identities(
    workspace_root: &Path,
) -> Result<Vec<LockedPackageIdentity>, WasmBuildError> {
    use crate::cargo::lockfile::{external_package, optional_string, required_string};
    let lockfile = workspace_root.join("Cargo.lock");
    let contents = match fs::read_to_string(&lockfile) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(WasmBuildError::Io {
                operation: "read Cargo lockfile for semantic projection",
                path: lockfile,
                source,
            });
        }
    };
    let lock = toml::from_str::<TomlValue>(&contents).map_err(|error| {
        invalid_metadata(&format!(
            "Cargo lockfile could not be projected as TOML: {error}"
        ))
    })?;
    let Some(packages) = lock.get("package").and_then(TomlValue::as_array) else {
        return Ok(Vec::new());
    };
    packages
        .iter()
        .filter_map(|package| {
            let table = match external_package(package) {
                Ok(Some(table)) => table,
                Ok(None) => return None,
                Err(message) => {
                    return Some(Err(invalid_metadata(&format!("Cargo lockfile {message}"))));
                }
            };
            let text = |field| {
                required_string(table, field)
                    .map(str::to_owned)
                    .map_err(|_| {
                        invalid_metadata(&format!(
                            "Cargo lockfile package `{field}` is missing or not a string"
                        ))
                    })
            };
            let source = required_string(table, "source").map(str::to_owned);
            Some(
                source
                    .map_err(|_| invalid_metadata("Cargo lockfile package source is not a string"))
                    .and_then(|source| {
                        Ok(LockedPackageIdentity {
                            name: text("name")?,
                            version: text("version")?,
                            source,
                            checksum: optional_string(table, "checksum")
                                .map_err(|_| {
                                    invalid_metadata(
                                        "Cargo lockfile package `checksum` is not a string",
                                    )
                                })?
                                .map(str::to_owned),
                        })
                    }),
            )
        })
        .collect()
}

fn semantic_package_identity(
    package: &schema::Package,
    workspace_root: &Path,
    locked_packages: &LockedPackageIndex<'_>,
) -> Option<InputDigest> {
    let version = package.version.to_string();
    let mut hasher = InputHasher::new("wasm-semantic-package-identity-v1");
    hasher.field("name", package.name.as_bytes());
    hasher.field("version", version.as_bytes());
    if package.source.is_none() {
        let manifest = package.manifest_path.strip_prefix(workspace_root).ok()?;
        let package_root = package.manifest_path.parent()?;
        if package_root == workspace_root {
            return None;
        }
        hasher.field("local-manifest", &os_bytes(manifest.as_os_str()));
    } else {
        let metadata_source = &package.source.as_ref()?.repr;
        let locked = locked_packages.get(&(
            package.name.as_str(),
            version.as_str(),
            metadata_source.as_str(),
        ))?;
        match locked.source.as_str() {
            source if source.starts_with("registry+") && locked.checksum.is_some() => {}
            source if source.starts_with("git+") && source.contains('#') => {}
            _ => return None,
        }
        hasher.field("external-package-id", package.id.repr.as_bytes());
        hasher.field("external-source", locked.source.as_bytes());
        hasher.field(
            "external-checksum",
            locked.checksum.as_deref().unwrap_or_default().as_bytes(),
        );
    }
    Some(hasher.finish())
}

fn semantic_package_projection(
    package: &schema::Package,
    node: &schema::Node,
    identities: &HashMap<&str, InputDigest>,
) -> Result<InputDigest, WasmBuildError> {
    // These are the effective package values Cargo can expose to compilation
    // through CARGO_PKG_* variables. Local manifests and external checksums
    // cover the remaining package definition.
    let mut hasher = InputHasher::new("wasm-semantic-package-projection-v2");
    // Borrow typed fields explicitly: whole-package serialization would include
    // checkout paths and inputs outside this semantic projection.
    macro_rules! fields {
        ($($field:ident),* $(,)?) => {$(
            hasher.field("package-field-name", stringify!($field).as_bytes());
            hasher.field("package-field-value", &serde_json::to_vec(&package.$field)
                .expect("serializing Cargo fields into a Vec cannot fail"));
        )*};
    }
    fields!(
        authors,
        default_run,
        description,
        documentation,
        edition,
        homepage,
        license,
        license_file,
        links,
        metadata,
        name,
        readme,
        repository,
        rust_version,
        version
    );

    let mut features = node
        .features
        .iter()
        .map(|feature| feature.as_str())
        .collect::<Vec<_>>();
    features.sort_unstable();
    for feature in features {
        hasher.field("enabled-feature", feature.as_bytes());
    }

    let mut dependencies = node
        .deps
        .iter()
        .map(|dependency| {
            if dependency.dep_kinds.iter().any(|kind| {
                !matches!(
                    kind.kind,
                    schema::DependencyKind::Normal
                        | schema::DependencyKind::Development
                        | schema::DependencyKind::Build
                )
            }) {
                return Err(invalid_metadata("unsupported Cargo dependency kind"));
            }
            let name = dependency.name.as_str();
            let package_id = dependency.pkg.repr.as_str();
            let identity = identities.get(package_id).copied().ok_or_else(|| {
                invalid_metadata(&format!(
                    "dependency `{package_id}` is outside the selected package closure"
                ))
            })?;
            let kinds = serde_json::to_vec(&dependency.dep_kinds)
                .expect("serializing Cargo dependency kinds into a Vec cannot fail");
            Ok::<_, WasmBuildError>((name, identity, kinds))
        })
        .collect::<Result<Vec<_>, _>>()?;
    dependencies.sort();
    for (name, identity, kinds) in dependencies {
        hasher.field("dependency-name", name.as_bytes());
        hasher.field("dependency-identity", identity.as_bytes());
        hasher.field("dependency-kinds", &kinds);
    }
    Ok(hasher.finish())
}

fn hash_toml_setting(hasher: &mut InputHasher, label: &str, value: Option<&TomlValue>) {
    hasher.field("workspace-setting-name", label.as_bytes());
    match value {
        Some(value) => hasher.field("workspace-setting-value", value.to_string().as_bytes()),
        None => hasher.field("workspace-setting-missing", b""),
    }
}

fn is_broad_workspace_input(label: &Path) -> bool {
    label == Path::new("workspace/Cargo.toml") || label == Path::new("workspace/Cargo.lock")
}

fn digest_resolved_local_inputs(
    inputs: &ResolvedLocalInputs,
    exclusions: &[PathBuf],
    cache: &mut LabeledPathDigestCache,
    error_path: &Path,
    validation_operation: &'static str,
    semantic_operation: &'static str,
) -> Result<(InputDigest, InputDigest), WasmBuildError> {
    let validation_digest = digest_labeled_paths_composable(
        "wasm-source-inputs-v1",
        inputs
            .validation_inputs
            .iter()
            .map(|(label, path)| (label.as_path(), path.as_path())),
        exclusions,
        cache,
    )
    .map_err(|source| WasmBuildError::Io {
        operation: validation_operation,
        path: error_path.to_owned(),
        source,
    })?;
    let input_digest = semantic_input_digest(inputs, validation_digest, exclusions, cache)
        .map_err(|source| WasmBuildError::Io {
            operation: semantic_operation,
            path: error_path.to_owned(),
            source,
        })?;
    Ok((input_digest, validation_digest))
}

fn semantic_input_digest(
    inputs: &ResolvedLocalInputs,
    validation_digest: InputDigest,
    exclusions: &[PathBuf],
    cache: &mut LabeledPathDigestCache,
) -> io::Result<InputDigest> {
    let Some(workspace) = inputs.workspace_projection else {
        return Ok(validation_digest);
    };
    let path_digest = digest_labeled_paths_composable(
        "wasm-source-inputs-v1",
        inputs
            .validation_inputs
            .iter()
            .filter(|(label, _)| !is_broad_workspace_input(label))
            .map(|(label, path)| (label.as_path(), path.as_path())),
        exclusions,
        cache,
    )?;
    let mut hasher = InputHasher::new("wasm-semantic-source-inputs-v1");
    hasher.field("path-input-digest", path_digest.as_bytes());
    hasher.field("workspace-projection", workspace.as_bytes());
    Ok(hasher.finish())
}

fn workspace_configuration_inputs(
    spec: &WasmBuildSpec,
    workspace_root: &Path,
) -> Result<Vec<(PathBuf, PathBuf)>, WasmBuildError> {
    let mut inputs = Vec::new();
    add_if_present(
        &mut inputs,
        "workspace/Cargo.toml",
        workspace_root.join("Cargo.toml"),
    );
    add_if_present(
        &mut inputs,
        "workspace/Cargo.lock",
        workspace_root.join("Cargo.lock"),
    );
    add_if_present(
        &mut inputs,
        "workspace/rust-toolchain.toml",
        workspace_root.join("rust-toolchain.toml"),
    );
    add_if_present(
        &mut inputs,
        "workspace/rust-toolchain",
        workspace_root.join("rust-toolchain"),
    );
    append_cargo_configuration_inputs(&mut inputs, spec, workspace_root)?;
    Ok(inputs)
}

fn append_cargo_configuration_inputs(
    inputs: &mut Vec<(PathBuf, PathBuf)>,
    spec: &WasmBuildSpec,
    workspace_root: &Path,
) -> Result<(), WasmBuildError> {
    let invocation_root =
        spec.workspace_root
            .canonicalize()
            .map_err(|source| WasmBuildError::Io {
                operation: "resolve Cargo invocation directory",
                path: spec.workspace_root.clone(),
                source,
            })?;
    let canonical_workspace =
        workspace_root
            .canonicalize()
            .map_err(|source| WasmBuildError::Io {
                operation: "resolve Cargo workspace directory",
                path: workspace_root.to_owned(),
                source,
            })?;

    let cargo_home = command_environment_value(spec, "CARGO_HOME");
    let default_home = default_home_directory(spec);
    let directories = crate::cargo::environment::configuration_directories(
        &invocation_root,
        cargo_home.as_deref(),
        default_home.as_deref().map(Path::as_os_str),
    )
    .map_err(|source| WasmBuildError::Io {
        operation: "resolve Cargo configuration directories",
        path: invocation_root.clone(),
        source,
    })?;
    let roots: Vec<_> = directories
        .iter()
        .filter_map(|directory| crate::cargo::environment::preferred_configuration(directory))
        .collect();
    let observed =
        crate::cargo::environment::observe_configurations(&roots).map_err(|error| match error {
            crate::cargo::environment::ConfigurationError::Io {
                operation,
                path,
                source,
            } => WasmBuildError::Io {
                operation,
                path,
                source,
            },
            crate::cargo::environment::ConfigurationError::Invalid { path, message } => {
                WasmBuildError::InvalidCargoConfiguration { path, message }
            }
        })?;
    for config in observed {
        if !inputs.iter().any(|(_, path)| path == &config.path) {
            inputs.push((
                cargo_configuration_label(&config.location, &canonical_workspace),
                config.path,
            ));
        }
    }
    Ok(())
}

#[cfg(windows)]
fn default_home_directory(spec: &WasmBuildSpec) -> Option<PathBuf> {
    command_environment_value(spec, "USERPROFILE")
        .or_else(|| command_environment_value(spec, "HOME"))
        .map(PathBuf::from)
}

#[cfg(not(windows))]
fn default_home_directory(spec: &WasmBuildSpec) -> Option<PathBuf> {
    command_environment_value(spec, "HOME").map(PathBuf::from)
}

fn command_environment_value(spec: &WasmBuildSpec, name: &str) -> Option<OsString> {
    spec.environment.get(OsStr::new(name)).cloned()
}

fn cargo_configuration_label(config: &Path, workspace_root: &Path) -> PathBuf {
    if let Ok(relative) = config.strip_prefix(workspace_root) {
        return PathBuf::from("cargo-config/workspace").join(relative);
    }
    let location = digest_bytes("cargo-config-location-v1", &os_bytes(config.as_os_str()));
    PathBuf::from("cargo-config/external").join(location.to_hex())
}

fn append_package_inputs(
    inputs: &mut Vec<(PathBuf, PathBuf)>,
    packages: &HashMap<&str, &schema::Package>,
    closure: BTreeSet<&str>,
    workspace_root: &Path,
) -> Result<(), WasmBuildError> {
    for id in closure {
        let Some(package) = packages.get(id) else {
            return Err(invalid_metadata(&format!(
                "resolved package `{id}` is missing"
            )));
        };
        if package.source.is_some() {
            continue;
        }
        let root = package.manifest_path.parent().ok_or_else(|| {
            invalid_metadata(&format!(
                "package `{}` manifest has no parent",
                package.name
            ))
        })?;
        let relative_manifest = package
            .manifest_path
            .as_std_path()
            .strip_prefix(workspace_root)
            .unwrap_or(package.manifest_path.as_std_path());
        let label = PathBuf::from(format!("package/{}@{}", package.name, package.version))
            .join(relative_manifest.parent().unwrap_or_else(|| Path::new(".")));
        inputs.push((label, root.as_std_path().to_owned()));
    }
    Ok(())
}

fn append_additional_inputs(
    inputs: &mut Vec<(PathBuf, PathBuf)>,
    spec: &WasmBuildSpec,
    workspace_root: &Path,
) {
    for additional in &spec.additional_inputs {
        let path = if additional.is_absolute() {
            additional.clone()
        } else {
            workspace_root.join(additional)
        };
        inputs.push((PathBuf::from("additional").join(additional), path));
    }
}

fn source_exclusions(spec: &WasmBuildSpec, inputs: &[(PathBuf, PathBuf)]) -> Vec<PathBuf> {
    let mut exclusions = vec![
        spec.target_dir.clone(),
        spec.workspace_root.join("target"),
        spec.workspace_root.join(".git"),
    ];
    if let Some(shared_target) = shared_incremental_target(spec) {
        exclusions.push(shared_target);
    }
    for (_, path) in inputs {
        if path.is_dir() {
            exclusions.push(path.join("target"));
            exclusions.push(path.join(".git"));
        }
    }
    exclusions
}

fn validate_shared_incremental_target_boundary(
    spec: &WasmBuildSpec,
    inputs: &[(PathBuf, PathBuf)],
) -> Result<(), WasmBuildError> {
    let Some(shared_target) = shared_incremental_target(spec) else {
        return Ok(());
    };
    let shared_target =
        canonicalize_allow_missing(&shared_target).map_err(|source| WasmBuildError::Io {
            operation: "resolve shared incremental Cargo target boundary",
            path: shared_target.clone(),
            source,
        })?;
    let exact_entries = spec.target_dir.join(".ic-testkit/wasm-targets");
    let exact_entries =
        canonicalize_allow_missing(&exact_entries).map_err(|source| WasmBuildError::Io {
            operation: "resolve exact Wasm cache boundary",
            path: exact_entries,
            source,
        })?;
    // Maintenance preserves only the shared target's direct metadata child.
    // An exact cache elsewhere beneath that target would lose retained entries.
    if shared_target.starts_with(&exact_entries)
        || (exact_entries.starts_with(&shared_target)
            && !exact_entries.starts_with(shared_target.join(".ic-testkit")))
    {
        return Err(WasmBuildError::InvalidSpec {
            message: "shared incremental target must not overlap removable exact Wasm cache state"
                .to_owned(),
        });
    }
    let mut boundaries = super::inputs::boundaries::CargoInputBoundaries::default();
    let resolved_inputs = inputs
        .iter()
        .map(|(_, input)| {
            boundaries
                .input(
                    input,
                    "resolve Cargo input boundary",
                    "inspect Cargo input boundary",
                )
                .cloned()
                .map_err(wasm_cache_fs_error)
        })
        .collect::<Result<Vec<_>, WasmBuildError>>()?;
    let has_safe_generated_root = std::iter::once(spec.target_dir.clone())
        .chain(std::iter::once(spec.workspace_root.join("target")))
        .chain(
            inputs
                .iter()
                .zip(&resolved_inputs)
                .filter(|(_, (_, is_directory))| *is_directory)
                .map(|((_, path), _)| path.join("target")),
        )
        .filter_map(|path| canonicalize_allow_missing(&path).ok())
        .filter(|root| {
            !resolved_inputs
                .iter()
                .any(|(input, _is_directory)| input.starts_with(root))
        })
        // Keep observing every candidate after finding a safe generated root.
        .fold(false, |safe, root| safe | shared_target.starts_with(root));
    if has_safe_generated_root {
        return Ok(());
    }

    for (input, is_directory) in resolved_inputs {
        if shared_target == input
            || (is_directory && shared_target.starts_with(&input))
            || input.starts_with(&shared_target)
        {
            return Err(WasmBuildError::InvalidSpec {
                message: format!(
                    "shared incremental target {} must not overlap exact Cargo inputs unless it is inside a generated target directory",
                    shared_target.display()
                ),
            });
        }
    }
    Ok(())
}

pub(super) fn shared_incremental_target(spec: &WasmBuildSpec) -> Option<PathBuf> {
    let WasmBuildCacheMode::SharedIncremental { target_dir } = &spec.cache_mode else {
        return None;
    };
    Some(if target_dir.is_absolute() {
        target_dir.clone()
    } else {
        spec.workspace_root.join(target_dir)
    })
}

fn shared_incremental_target_exists(
    spec: &WasmBuildSpec,
    operation: &'static str,
) -> Result<bool, WasmBuildError> {
    let target_dir =
        shared_incremental_target(spec).ok_or_else(|| WasmBuildError::InvalidSpec {
            message: "shared incremental target is not configured".to_owned(),
        })?;
    match fs::symlink_metadata(&target_dir) {
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(_) => Err(WasmBuildError::InvalidSpec {
            message: format!(
                "shared incremental Cargo target {} must be a directory",
                target_dir.display()
            ),
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(WasmBuildError::Io {
            operation,
            path: target_dir,
            source,
        }),
    }
}

fn apply_command_environment(command: &mut Command, spec: &WasmBuildSpec) {
    command.env_clear().envs(&spec.environment);
}

fn run_cargo_build(
    spec: &WasmBuildSpec,
    build_target_dir: &Path,
    targets: &[WasmArtifactTarget],
    progress: &mut ProgressReporter<'_>,
    mut spawn: &mut dyn FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<Vec<PathBuf>, WasmBuildError> {
    let command = cargo_build_command(spec, build_target_dir)?;
    communicate_cargo_build(
        spec,
        build_target_dir,
        targets,
        command,
        progress,
        &mut spawn,
    )
}

fn cargo_build_command(
    spec: &WasmBuildSpec,
    build_target_dir: &Path,
) -> Result<Command, WasmBuildError> {
    let absolute_target_dir =
        canonicalize_allow_missing(build_target_dir).map_err(|source| WasmBuildError::Io {
            operation: "resolve Cargo build target directory",
            path: build_target_dir.to_owned(),
            source,
        })?;
    let mut command = Command::new(&spec.cargo_program);
    apply_command_environment(&mut command, spec);
    command
        .current_dir(&spec.workspace_root)
        .env("CARGO_TARGET_DIR", absolute_target_dir)
        .args(["build", "--message-format=json"]);
    if let Some(example) = &spec.example {
        command.args(["--example", example]);
    } else {
        command.arg("--lib");
    }
    command
        .args(["--target", &spec.target])
        .args(spec.cargo_selection.build_arguments());
    for package in &spec.packages {
        command.args(["-p", package]);
    }

    Ok(command)
}

fn communicate_cargo_build(
    spec: &WasmBuildSpec,
    build_target_dir: &Path,
    targets: &[WasmArtifactTarget],
    mut command: Command,
    progress: &mut ProgressReporter<'_>,
    spawn: &mut dyn FnMut(&mut Command) -> io::Result<OwnedChild>,
) -> Result<Vec<PathBuf>, WasmBuildError> {
    // The caller's spawner may override stdin and selects child/group custody.
    // Progress reporting never changes either selection.
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let started = Instant::now();
    let mut child = spawn(&mut command).map_err(|source| WasmBuildError::CommandSpawn {
        phase: WasmBuildPhase::CargoBuild,
        program: spec.cargo_program.clone(),
        source,
    })?;
    progress.emit(WasmBuildProgressEvent::CargoStarted {
        target_dir: build_target_dir.to_owned(),
    });
    let progress = RefCell::new(progress);
    let mut artifacts = vec![None; targets.len()];
    let mut collect = |message| collect_wasm_artifact(message, targets, &mut artifacts);
    let observed = communicate_cargo(
        &mut child,
        OutputLimits {
            // Retention is bounded; all chunks still reach the observer.
            // Output volume never terminates a build, and builds have no deadline.
            stdout: OutputLimit::Truncate(CARGO_DIAGNOSTIC_BYTES),
            stderr: OutputLimit::Truncate(CARGO_DIAGNOSTIC_BYTES),
            timeout: None,
        },
        spec.cargo_message_bytes,
        || {
            progress
                .borrow_mut()
                .emit_heartbeat_if_due(WasmBuildPhase::CargoBuild, started.elapsed());
            false
        },
        |stream, bytes| {
            let mut progress = progress.borrow_mut();
            if progress.config.emit_cargo_output {
                progress.emit(WasmBuildProgressEvent::CargoOutput {
                    stream,
                    bytes: bytes.to_vec(),
                });
            }
            Ok(())
        },
        &mut collect,
    );
    let result = observed.execution;
    let status = match &result {
        Ok(evidence) => Some(evidence.status),
        Err(error) => error.evidence().and_then(|evidence| evidence.status),
    };
    if let Some(status) = status {
        progress
            .borrow_mut()
            .emit(WasmBuildProgressEvent::CargoFinished {
                success: status.success(),
                code: status.code(),
                elapsed: started.elapsed(),
            });
    }
    let evidence = result.map_err(|error| WasmBuildError::Io {
        operation: "communicate with cargo build",
        path: PathBuf::from(&spec.cargo_program),
        // Retain typed Host evidence and secondary cleanup failures.
        source: io::Error::other(error),
    })?;
    let status = evidence.status;
    if status.success() {
        observed.messages.map_err(|source| WasmBuildError::Io {
            operation: "decode Cargo build artifacts",
            path: PathBuf::from(&spec.cargo_program),
            source,
        })?;
        return artifacts
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| WasmBuildError::MissingArtifacts {
                paths: expected_artifacts(spec, build_target_dir, targets),
            });
    }
    Err(WasmBuildError::CommandFailed {
        phase: WasmBuildPhase::CargoBuild,
        status,
        stdout: cargo_diagnostic(&evidence.stdout, evidence.stdout_truncated),
        stderr: cargo_diagnostic(&evidence.stderr, evidence.stderr_truncated),
    })
}

fn collect_wasm_artifact(
    message: schema::Message,
    targets: &[WasmArtifactTarget],
    artifacts: &mut [Option<PathBuf>],
) -> io::Result<()> {
    let schema::Message::CompilerArtifact(artifact) = message else {
        return Ok(());
    };
    // Acquisition admits unique targets before dispatching Cargo.
    let Some((_, selected)) = targets.iter().zip(artifacts).find(|(target, _)| {
        artifact_matches(&artifact, &target.package_id, &target.name, &target.kind)
    }) else {
        return Ok(());
    };
    if !artifact
        .target
        .crate_types
        .contains(&schema::CrateType::CDyLib)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Cargo reported a non-cdylib Wasm target",
        ));
    }
    let mut matching = wasm_artifact_paths(&artifact);
    let path = matching
        .next()
        .filter(|path| path.is_absolute())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Cargo omitted absolute Wasm artifact path",
            )
        })?;
    if matching.next().is_some() || selected.replace(path.to_path_buf()).is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Cargo reported ambiguous Wasm artifacts",
        ));
    }
    Ok(())
}

fn cargo_diagnostic(bytes: &[u8], truncated: bool) -> String {
    let mut text = String::from_utf8_lossy(bytes).into_owned();
    if truncated {
        text.push_str("\n[diagnostic truncated after 1 MiB; enable Cargo output events for the complete stream]\n");
    }
    text
}

fn ensure_command_success(
    phase: WasmBuildPhase,
    output: CompletedExecution,
) -> Result<CompletedExecution, WasmBuildError> {
    if output.status.success() {
        return Ok(output);
    }
    Err(WasmBuildError::CommandFailed {
        phase,
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn artifact_relative_path(spec: &WasmBuildSpec, target: &WasmArtifactTarget) -> PathBuf {
    let profile = Path::new(&spec.target).join(
        spec.cargo_selection
            .profile_target_dir()
            .expect("Cargo profile was validated before artifact discovery"),
    );
    let directory = if spec.example.is_some() {
        profile.join("examples")
    } else {
        profile
    };
    directory.join(format!("{}.wasm", target.name.replace('-', "_")))
}

fn expected_artifacts(
    spec: &WasmBuildSpec,
    target_dir: &Path,
    targets: &[WasmArtifactTarget],
) -> Vec<PathBuf> {
    targets
        .iter()
        .map(|target| target_dir.join(artifact_relative_path(spec, target)))
        .collect()
}

fn artifact_publication_paths(
    spec: &WasmBuildSpec,
    cache_entry: &Path,
    targets: &[WasmArtifactTarget],
) -> Vec<(PathBuf, PathBuf)> {
    targets
        .iter()
        .map(|target| {
            let relative = artifact_relative_path(spec, target);
            (cache_entry.join(&relative), spec.target_dir.join(relative))
        })
        .collect()
}

#[derive(Debug, Eq, PartialEq)]
struct VerifiedWasmArtifact {
    cached_path: PathBuf,
    public_path: PathBuf,
    info: FileDigest,
}

fn cache_entry_directory(spec: &WasmBuildSpec, fingerprint: InputDigest) -> PathBuf {
    spec.target_dir
        .join(".ic-testkit/wasm-targets")
        .join(fingerprint.to_hex())
}

fn validated_artifact_set(
    artifacts: Vec<(PathBuf, PathBuf)>,
    fingerprint: InputDigest,
) -> Option<Vec<VerifiedWasmArtifact>> {
    let header = artifact_stamp_header(fingerprint);
    // The artifact digest has a fixed hex width, followed by one newline.
    let stamp_len =
        header.len() + ARTIFACT_STAMP_DIGEST_LABEL.len() + fingerprint.as_bytes().len() * 2 + 1;
    artifacts
        .into_iter()
        .map(|(artifact, public_path)| {
            let metadata = fs::metadata(&artifact).ok()?;
            if !metadata.is_file() || metadata.len() == 0 {
                return None;
            }
            let stamp =
                read_stamp_with_limit(&artifact_stamp_path(&artifact), stamp_len).ok()??;
            // An incomplete stamp or different build cannot be a hit. Reject it
            // before reading the Wasm bytes; then verify the complete exact stamp.
            if stamp.len() != stamp_len || !stamp.starts_with(&header) {
                return None;
            }
            let info = digest_file("wasm-artifact-v1", &artifact).ok()?;
            (stamp == artifact_stamp_contents(fingerprint, info.digest)).then_some(
                VerifiedWasmArtifact {
                    cached_path: artifact,
                    public_path,
                    info,
                },
            )
        })
        .collect()
}

fn missing_artifacts(artifacts: &[PathBuf]) -> Vec<PathBuf> {
    artifacts
        .iter()
        .filter(|path| {
            fs::metadata(path).map_or(true, |metadata| !metadata.is_file() || metadata.len() == 0)
        })
        .cloned()
        .collect()
}

fn artifact_stamp_path(artifact: &Path) -> PathBuf {
    let mut name = artifact
        .file_name()
        .map_or_else(|| OsString::from("artifact"), OsString::from);
    name.push(".ic-testkit-build");
    artifact.with_file_name(name)
}

fn artifact_stamp_header(fingerprint: InputDigest) -> String {
    format!("{CACHE_FORMAT_VERSION}\nbuild-sha256:{fingerprint}\n")
}

fn artifact_stamp_contents(fingerprint: InputDigest, artifact_digest: InputDigest) -> String {
    format!(
        "{}{ARTIFACT_STAMP_DIGEST_LABEL}{artifact_digest}\n",
        artifact_stamp_header(fingerprint),
    )
}

fn write_artifact_stamp(
    artifact: &Path,
    fingerprint: InputDigest,
    info: &FileDigest,
    preserve_matching: bool,
) -> Result<(), WasmBuildError> {
    let stamp_path = artifact_stamp_path(artifact);
    let stamp = artifact_stamp_contents(fingerprint, info.digest);
    if preserve_matching && destination_matches_bytes(&stamp_path, stamp.as_bytes()) {
        return Ok(());
    }
    ic_host_fs::durable::write_bytes(
        &stamp_path,
        ic_host_fs::durable::WriteOptions {
            mode: ic_host_fs::durable::PublicationMode::Replace,
            permissions: 0o666,
        },
        stamp.as_bytes(),
    )
    .map_err(|source| WasmBuildError::Io {
        operation: "publish Wasm build stamp",
        path: stamp_path,
        source: io::Error::other(source),
    })
}

fn publish_artifact_stamps(
    artifacts: Vec<(PathBuf, PathBuf)>,
    fingerprint: InputDigest,
) -> Result<Vec<VerifiedWasmArtifact>, WasmBuildError> {
    artifacts
        .into_iter()
        .map(|(cached_path, public_path)| {
            let info = digest_file("wasm-artifact-v1", &cached_path).map_err(|source| {
                WasmBuildError::Io {
                    operation: "hash built Wasm artifact",
                    path: cached_path.clone(),
                    source,
                }
            })?;
            write_artifact_stamp(&cached_path, fingerprint, &info, false)?;
            Ok(VerifiedWasmArtifact {
                cached_path,
                public_path,
                info,
            })
        })
        .collect()
}

fn materialize_artifacts(
    artifacts: &[VerifiedWasmArtifact],
    fingerprint: InputDigest,
    preserve_matching: bool,
) -> Result<(), WasmBuildError> {
    for artifact in artifacts {
        materialize_verified_output(
            "wasm-artifact-v1",
            &artifact.cached_path,
            &artifact.public_path,
            preserve_matching.then_some(&artifact.info),
        )
        .map_err(|source| WasmBuildError::Io {
            operation: "publish Wasm artifact",
            path: artifact.public_path.clone(),
            source,
        })?;
    }
    // Complete every copy before stamping any public output.
    for artifact in artifacts {
        write_artifact_stamp(
            &artifact.public_path,
            fingerprint,
            &artifact.info,
            preserve_matching,
        )?;
    }
    Ok(())
}

fn copy_wasm_artifacts(
    source_artifacts: &[PathBuf],
    destinations: &[(PathBuf, PathBuf)],
) -> Result<(), WasmBuildError> {
    // Cargo paths are external evidence. Refuse a mismatched inventory before any copy.
    if source_artifacts.len() != destinations.len() {
        return Err(invalid_metadata(
            "Cargo artifact count does not match admitted outputs",
        ));
    }
    for (index, (cached, _)) in destinations.iter().enumerate() {
        let source = &source_artifacts[index];
        if source != cached {
            copy_file_atomic(source, cached).map_err(|source_error| WasmBuildError::Io {
                operation: "cache shared-incremental Wasm artifact",
                path: cached.clone(),
                source: source_error,
            })?;
        }
    }
    Ok(())
}

fn create_dir_all(path: &Path, operation: &'static str) -> Result<(), WasmBuildError> {
    fs::create_dir_all(path).map_err(|source| WasmBuildError::Io {
        operation,
        path: path.to_owned(),
        source,
    })
}

fn add_if_present(inputs: &mut Vec<(PathBuf, PathBuf)>, label: &str, path: PathBuf) {
    if path.exists() {
        inputs.push((PathBuf::from(label), path));
    }
}

fn invalid_metadata(message: &str) -> WasmBuildError {
    WasmBuildError::InvalidMetadata {
        message: message.to_owned(),
    }
}

impl WasmBuildError {
    fn indicates_input_change(&self) -> bool {
        match self {
            Self::InputsChangedDuringAcquisition { .. } => true,
            Self::FailedBuildCleanup { build_error, .. } => build_error.indicates_input_change(),
            _ => false,
        }
    }
}

impl std::fmt::Display for WasmBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSpec { message } => {
                write!(formatter, "invalid Wasm build spec: {message}")
            }
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::CommandSpawn {
                phase,
                program,
                source,
            } => write!(
                formatter,
                "failed to launch {phase} using `{}`: {source}",
                program.to_string_lossy(),
            ),
            Self::CommandFailed {
                phase,
                status,
                stdout,
                stderr,
            } => write!(
                formatter,
                "{phase} failed with {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            ),
            Self::InvalidMetadata { message } => {
                write!(formatter, "invalid Cargo metadata: {message}")
            }
            Self::InvalidCargoConfiguration { path, message } => write!(
                formatter,
                "invalid Cargo configuration at {}: {message}",
                path.display(),
            ),
            Self::MissingArtifacts { paths } => {
                formatter.write_str("cargo build succeeded without producing: ")?;
                for (index, path) in paths.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{}", path.display())?;
                }
                Ok(())
            }
            Self::InputsChangedDuringAcquisition { before, after } => write!(
                formatter,
                "Wasm inputs changed during artifact acquisition: {before} -> {after}",
            ),
            Self::PreparedInputSnapshotInvalidated => formatter.write_str(
                "the prepared Wasm input snapshot was invalidated before artifact publication",
            ),
            Self::FailedBuildCleanup {
                build_error,
                path,
                source,
            } => write!(
                formatter,
                "Wasm build failed ({build_error}) and its incomplete target directory at {} could not be removed: {source}",
                path.display(),
            ),
        }
    }
}

impl std::error::Error for WasmBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. }
            | Self::CommandSpawn { source, .. }
            | Self::FailedBuildCleanup { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
