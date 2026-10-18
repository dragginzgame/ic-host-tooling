use super::cache_fs::{CacheRetentionLimits, canonicalize_allow_missing};
use ic_host_artifacts::batch::{BatchLabelError, validate_labels};

use std::{
    collections::HashSet,
    marker::PhantomData,
    path::PathBuf,
    time::{Duration, Instant},
};

use super::wasm_cache::{
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceOutcome,
    WasmBuildBatchInputMetrics, WasmBuildBatchInputResolver, WasmBuildError,
    WasmBuildFailureDetails, WasmBuildInputSnapshotState, WasmBuildOutcome,
    WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildSpec, WasmBuildTimings,
    WasmInputResolutionTimings, build_wasm_canisters_cached_in_batch, shared_incremental_target,
};

/// Orchestration shared by every entry in one independent Wasm build batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildBatchConfig {
    shared_incremental_maintenance: Option<SharedIncrementalTargetMaintenanceConfig>,
}

/// Caller-labeled specification for one exact Wasm batch entry.
///
/// The label is report and progress identity only; it does not alter the
/// underlying exact Wasm fingerprint or cache key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabeledWasmBuildSpec {
    label: String,
    spec: WasmBuildSpec,
}

/// Ordered outcomes and failures from a collect-all Wasm build batch.
#[derive(Debug)]
pub struct WasmBuildBatchReport {
    entries: Vec<WasmBuildBatchEntry>,
    input_resolution: WasmBuildBatchInputMetrics,
    total: Duration,
}

/// Immutable prepared Cargo input resolution shared by concurrent readers.
///
/// Preparation resolves the complete declared specification set while the
/// caller holds a genuine source write-exclusion guard. Reader batches may run
/// concurrently through `&self`, but cannot introduce specifications that
/// were not declared during preparation.
pub struct WasmBuildInputSnapshot<'guard> {
    state: WasmBuildInputSnapshotState,
    _source_guard: PhantomData<&'guard ()>,
}

/// Preparation and reader-reuse counters for one immutable input snapshot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildInputSnapshotMetrics {
    specifications: usize,
    input_resolution_runs: usize,
    input_resolution_reuses: usize,
    input_resolution_timings: WasmInputResolutionTimings,
    reader_reuses: usize,
    invalidated: bool,
}

/// One ordered caller-labeled result from a Wasm build batch.
#[derive(Debug)]
pub struct WasmBuildBatchEntry {
    index: usize,
    label: String,
    result: Result<WasmBuildOutcome, (WasmBuildError, WasmBuildFailureDetails)>,
    entry_elapsed: Duration,
}

/// Structural error that prevents a labeled Wasm batch from starting.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildBatchContractError {
    /// Invalid input labels, rejected before source validation or command effects.
    Labels(BatchLabelError),
    /// A source mutation invalidated the caller's immutable-source lease.
    SourceLeaseInvalidated,
    /// A prepared snapshot reader requested a specification absent at preparation.
    SpecificationNotPrepared {
        /// Zero-based position of the undeclared entry.
        index: usize,
        /// Caller-owned label of the undeclared entry.
        label: String,
    },
}

impl LabeledWasmBuildSpec {
    /// Attach a caller-owned stable label to one Wasm build specification.
    #[must_use]
    pub fn new(label: impl Into<String>, spec: WasmBuildSpec) -> Self {
        Self {
            label: label.into(),
            spec,
        }
    }

    /// Caller-owned report and progress label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Underlying exact Wasm build specification.
    #[must_use]
    pub const fn spec(&self) -> &WasmBuildSpec {
        &self.spec
    }

    /// Consume the entry into its label and Wasm build specification.
    #[must_use]
    pub fn into_parts(self) -> (String, WasmBuildSpec) {
        (self.label, self.spec)
    }
}

impl<'guard> WasmBuildInputSnapshot<'guard> {
    /// Resolve and freeze the complete specification set under a source lease.
    ///
    /// The guard must prevent mutation of every Cargo/rustc executable,
    /// manifest, configuration file, discovered source, declared additional
    /// input, and relevant environment value used by the supplied
    /// specifications. The type system cannot verify guard provenance.
    ///
    /// # Errors
    /// Returns specification, tool/metadata, filesystem or input-resolution errors.
    pub fn prepare_assuming_sources_immutable<Guard: ?Sized>(
        _source_write_guard: &'guard Guard,
        specs: &[WasmBuildSpec],
    ) -> Result<Self, WasmBuildError> {
        Ok(Self {
            state: WasmBuildInputSnapshotState::prepare(specs)?,
            _source_guard: PhantomData,
        })
    }

    /// Build a collect-all batch with explicit orchestration and optional progress.
    ///
    /// Source lease, feature isolation and retention match independent batches.
    /// `spawn` selects Cargo child/group custody and cleanup, independently of progress.
    /// Prepared snapshots reject specifications absent during preparation.
    ///
    /// # Errors
    /// Returns invalid labels, an invalidated lease or an unprepared specification
    /// error before work. Individual build failures remain in the report.
    pub fn build_batch(
        &self,
        specs: &[LabeledWasmBuildSpec],
        config: WasmBuildBatchConfig,
        progress: Option<(
            WasmBuildProgressConfig,
            &mut dyn FnMut(WasmBuildBatchProgressEvent),
        )>,
        mut spawn: impl FnMut(
            &mut std::process::Command,
        ) -> std::io::Result<ic_host_process::child::OwnedChild>,
    ) -> Result<WasmBuildBatchReport, WasmBuildBatchContractError> {
        run_wasm_batch(specs, config, Some(&self.state), progress, &mut spawn)
    }

    /// Current preparation, reader-reuse, and invalidation metrics.
    #[must_use]
    pub fn metrics(&self) -> WasmBuildInputSnapshotMetrics {
        let preparation = self.state.preparation_metrics();
        WasmBuildInputSnapshotMetrics {
            specifications: self.state.specification_count(),
            input_resolution_runs: preparation.runs,
            input_resolution_reuses: preparation.reuses,
            input_resolution_timings: self.state.preparation_timings(),
            reader_reuses: self.state.reader_reuses(),
            invalidated: self.state.is_invalidated(),
        }
    }
}

impl WasmBuildInputSnapshotMetrics {
    /// Number of exact specifications captured during preparation.
    #[must_use]
    pub const fn specifications(self) -> usize {
        self.specifications
    }

    /// Number of workspace/toolchain resolution snapshots prepared.
    #[must_use]
    pub const fn input_resolution_runs(self) -> usize {
        self.input_resolution_runs
    }

    /// Number of prepared specifications sharing another resolution run.
    #[must_use]
    pub const fn input_resolution_reuses(self) -> usize {
        self.input_resolution_reuses
    }

    /// Complete tool, metadata, discovery, and hashing preparation timings.
    #[must_use]
    pub const fn input_resolution_timings(self) -> WasmInputResolutionTimings {
        self.input_resolution_timings
    }

    /// Cumulative exact specification resolutions served to readers.
    #[must_use]
    pub const fn reader_reuses(self) -> usize {
        self.reader_reuses
    }

    /// Whether any reader detected a violation of the source lease.
    #[must_use]
    pub const fn is_invalidated(self) -> bool {
        self.invalidated
    }
}

impl WasmBuildBatchEntry {
    /// Zero-based position in the supplied labeled specification slice.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Caller-owned stable label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Structured success or failure for this entry.
    ///
    /// # Errors
    /// Returns this entry's recorded build error when the entry failed.
    pub const fn result(&self) -> Result<&WasmBuildOutcome, &WasmBuildError> {
        match &self.result {
            Ok(outcome) => Ok(outcome),
            Err((error, _)) => Err(error),
        }
    }

    /// Successful Wasm outcome, when this entry succeeded.
    #[must_use]
    pub fn outcome(&self) -> Option<&WasmBuildOutcome> {
        self.result.as_ref().ok()
    }

    /// Structured build failure, when this entry failed.
    #[must_use]
    pub fn error(&self) -> Option<&WasmBuildError> {
        self.result().err()
    }

    /// Structured phase and partial timings when this entry failed.
    #[must_use]
    pub const fn failure_details(&self) -> Option<WasmBuildFailureDetails> {
        match &self.result {
            Ok(_) => None,
            Err((_, details)) => Some(*details),
        }
    }

    /// Complete wall-clock time retained for this entry.
    #[must_use]
    pub const fn entry_elapsed(&self) -> Duration {
        self.entry_elapsed
    }

    /// Whether this entry completed successfully.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    /// Consume the entry into its identity, result, optional failure details, and wall time.
    pub fn into_parts(
        self,
    ) -> (
        usize,
        String,
        Result<WasmBuildOutcome, WasmBuildError>,
        Option<WasmBuildFailureDetails>,
        Duration,
    ) {
        let (result, failure) = match self.result {
            Ok(outcome) => (Ok(outcome), None),
            Err((error, details)) => (Err(error), Some(details)),
        };
        (self.index, self.label, result, failure, self.entry_elapsed)
    }
}

/// Aggregate counters and successful-acquisition timings for a Wasm build batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildBatchMetrics {
    specifications: usize,
    built: usize,
    reused: usize,
    input_resolution_runs: usize,
    input_resolution_reuses: usize,
    input_resolution_prepared_reuses: usize,
    successful_timings: WasmBuildTimings,
    total: Duration,
}

/// Structured progress for an independent sequence of exact Wasm builds.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildBatchProgressEvent {
    /// One independently resolved build specification is about to start.
    BuildStarted {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Caller-owned stable label.
        label: String,
        /// Total number of supplied specifications.
        total: usize,
    },
    /// Progress forwarded from one independent build.
    BuildProgress {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Caller-owned stable label.
        label: String,
        /// Event emitted by that build.
        event: WasmBuildProgressEvent,
    },
    /// One independent build completed successfully.
    BuildFinished {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Caller-owned stable label.
        label: String,
    },
    /// One independent build failed.
    BuildFailed {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Caller-owned stable label.
        label: String,
    },
}

impl WasmBuildBatchReport {
    /// Ordered labeled entries.
    #[must_use]
    pub fn entries(&self) -> &[WasmBuildBatchEntry] {
        &self.entries
    }

    /// Consume the report into its ordered labeled entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<WasmBuildBatchEntry> {
        self.entries
    }

    /// Successful entries, retaining the canonical identity, result and timings.
    pub fn outcomes(&self) -> impl Iterator<Item = &WasmBuildBatchEntry> {
        self.entries.iter().filter(|entry| entry.is_success())
    }

    /// Failed entries; inspect `error()` and `failure_details()` on each entry.
    pub fn failures(&self) -> impl Iterator<Item = &WasmBuildBatchEntry> {
        self.entries.iter().filter(|entry| !entry.is_success())
    }

    /// Canonical entries paired with their integrated shared-target maintenance.
    ///
    /// Batch-owned maintenance contributes at most one outcome for each
    /// distinct configured shared-target path.
    pub fn shared_incremental_maintenance_outcomes(
        &self,
    ) -> impl Iterator<
        Item = (
            &WasmBuildBatchEntry,
            &SharedIncrementalTargetMaintenanceOutcome,
        ),
    > {
        self.outcomes().filter_map(|entry| {
            entry
                .outcome()?
                .record()
                .shared_incremental_maintenance()
                .map(|outcome| (entry, outcome))
        })
    }

    /// Complete wall-clock time for the sequential collect-all batch.
    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }

    /// Whether every specification completed successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.entries.iter().all(WasmBuildBatchEntry::is_success)
    }

    /// Aggregate outcome, input-resolution reuse, and timing counters.
    #[must_use]
    pub fn metrics(&self) -> WasmBuildBatchMetrics {
        let mut metrics = WasmBuildBatchMetrics {
            specifications: self.entries.len(),
            input_resolution_runs: self.input_resolution.runs,
            input_resolution_reuses: self.input_resolution.reuses,
            input_resolution_prepared_reuses: self.input_resolution.prepared_reuses,
            total: self.total,
            ..WasmBuildBatchMetrics::default()
        };
        for outcome in self.entries.iter().filter_map(WasmBuildBatchEntry::outcome) {
            if outcome.is_reused() {
                metrics.reused += 1;
            } else {
                metrics.built += 1;
            }
            metrics.successful_timings = metrics
                .successful_timings
                .saturating_add(outcome.record().timings());
        }
        metrics
    }
}

impl WasmBuildBatchMetrics {
    /// Number of supplied specifications.
    #[must_use]
    pub const fn specifications(self) -> usize {
        self.specifications
    }

    /// Number of successful specifications.
    #[must_use]
    pub const fn succeeded(self) -> usize {
        self.built + self.reused
    }

    /// Number of failed specifications.
    #[must_use]
    pub const fn failed(self) -> usize {
        self.specifications - self.succeeded()
    }

    /// Number of newly built Wasm artifact sets.
    #[must_use]
    pub const fn built(self) -> usize {
        self.built
    }

    /// Number of Wasm artifact sets reused from the exact cache.
    #[must_use]
    pub const fn reused(self) -> usize {
        self.reused
    }

    /// Number of workspace/toolchain input-resolution snapshots performed.
    #[must_use]
    pub const fn input_resolution_runs(self) -> usize {
        self.input_resolution_runs
    }

    /// Number of specifications resolved by reusing another batch snapshot.
    #[must_use]
    pub const fn input_resolution_reuses(self) -> usize {
        self.input_resolution_reuses
    }

    /// Number of specifications resolved from a prepared concurrent snapshot.
    #[must_use]
    pub const fn input_resolution_prepared_reuses(self) -> usize {
        self.input_resolution_prepared_reuses
    }

    /// Sum of timings from successful acquisitions.
    #[must_use]
    pub const fn successful_timings(self) -> WasmBuildTimings {
        self.successful_timings
    }

    /// Complete wall-clock time for the sequential batch.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl WasmBuildBatchConfig {
    /// Create batch orchestration without batch-owned target maintenance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            shared_incremental_maintenance: None,
        }
    }

    /// Maintain each distinct resolved shared target once through its first batch entry.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance(
        mut self,
        config: SharedIncrementalTargetMaintenanceConfig,
    ) -> Self {
        self.shared_incremental_maintenance = Some(config);
        self
    }

    /// Strictly maintain each distinct shared target at most once per interval.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance_at_most_every(
        self,
        policy: CacheRetentionLimits,
        minimum_interval: Duration,
    ) -> Self {
        self.with_shared_incremental_target_maintenance(
            SharedIncrementalTargetMaintenanceConfig::new(policy, minimum_interval),
        )
    }

    /// Batch-owned shared-target maintenance, when configured.
    #[must_use]
    pub const fn shared_incremental_target_maintenance(
        self,
    ) -> Option<SharedIncrementalTargetMaintenanceConfig> {
        self.shared_incremental_maintenance
    }
}

impl std::fmt::Display for WasmBuildBatchReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let metrics = self.metrics();
        write!(
            formatter,
            "builds={} succeeded={} failed={} built={} reused={} input_resolution_runs={} input_resolution_reuses={} input_resolution_prepared_reuses={} successful_timings=({}) total={:?}",
            metrics.specifications(),
            metrics.succeeded(),
            metrics.failed(),
            metrics.built(),
            metrics.reused(),
            metrics.input_resolution_runs(),
            metrics.input_resolution_reuses(),
            metrics.input_resolution_prepared_reuses(),
            metrics.successful_timings(),
            metrics.total(),
        )
    }
}

/// Build every Wasm specification as an independent Cargo invocation.
///
/// Pass explicit batch configuration and optional progress reporting.
/// `spawn` selects Cargo child/group custody and cleanup for each cache miss,
/// independently of progress; it may override the default null stdin.
/// Specifications run sequentially and every result is retained. Each entry
/// keeps its own package set, profile arguments, feature resolution,
/// fingerprint, locks, and cache policy. Packages are never combined into one
/// Cargo command because doing so can unify shared dependency features.
///
/// # Errors
/// Returns a contract error for an empty or duplicate label before any work.
/// Individual build failures are retained in the returned report.
pub fn build_wasm_canisters_cached_batch(
    specs: &[LabeledWasmBuildSpec],
    config: WasmBuildBatchConfig,
    progress: Option<(
        WasmBuildProgressConfig,
        &mut dyn FnMut(WasmBuildBatchProgressEvent),
    )>,
    mut spawn: impl FnMut(
        &mut std::process::Command,
    ) -> std::io::Result<ic_host_process::child::OwnedChild>,
) -> Result<WasmBuildBatchReport, WasmBuildBatchContractError> {
    run_wasm_batch(specs, config, None, progress, &mut spawn)
}

#[expect(
    clippy::result_large_err,
    reason = "Batch entries retain complete failure details inline without per-failure allocation."
)]
fn run_wasm_batch(
    specs: &[LabeledWasmBuildSpec],
    batch_config: WasmBuildBatchConfig,
    reuse: Option<&WasmBuildInputSnapshotState>,
    mut observation: Option<(
        WasmBuildProgressConfig,
        &mut dyn FnMut(WasmBuildBatchProgressEvent),
    )>,
    mut spawn: impl FnMut(
        &mut std::process::Command,
    ) -> std::io::Result<ic_host_process::child::OwnedChild>,
) -> Result<WasmBuildBatchReport, WasmBuildBatchContractError> {
    validate_labels(specs.iter().map(LabeledWasmBuildSpec::label))
        .map_err(WasmBuildBatchContractError::Labels)?;
    validate_input_reuse(reuse)?;
    let count = specs.len();
    let mut resolver =
        WasmBuildBatchInputResolver::new(specs.iter().map(LabeledWasmBuildSpec::spec), reuse)
            .map_err(
                |index| WasmBuildBatchContractError::SpecificationNotPrepared {
                    index,
                    label: specs[index].label.clone(),
                },
            )?;
    let mut report = build_wasm_batch(specs, batch_config, |spec, index| {
        let Some((progress_config, observer)) = observation.as_mut() else {
            return build_wasm_canisters_cached_in_batch(
                spec,
                index,
                &mut resolver,
                None,
                &mut spawn,
            );
        };
        let label = specs[index].label.clone();
        observer(WasmBuildBatchProgressEvent::BuildStarted {
            index,
            label: label.clone(),
            total: count,
        });
        let mut forward = |event| {
            observer(WasmBuildBatchProgressEvent::BuildProgress {
                index,
                label: label.clone(),
                event,
            });
        };
        let attempt = build_wasm_canisters_cached_in_batch(
            spec,
            index,
            &mut resolver,
            Some((*progress_config, &mut forward)),
            &mut spawn,
        );
        observer(match &attempt {
            Ok(_) => WasmBuildBatchProgressEvent::BuildFinished { index, label },
            Err(_) => WasmBuildBatchProgressEvent::BuildFailed { index, label },
        });
        attempt
    });
    report.input_resolution = resolver.metrics();
    Ok(report)
}

fn build_wasm_batch<F>(
    specs: &[LabeledWasmBuildSpec],
    config: WasmBuildBatchConfig,
    mut build: F,
) -> WasmBuildBatchReport
where
    F: FnMut(
        &WasmBuildSpec,
        usize,
    ) -> Result<WasmBuildOutcome, (WasmBuildError, WasmBuildFailureDetails)>,
{
    let started = Instant::now();
    let mut entries = Vec::with_capacity(specs.len());
    let mut maintenance = BatchMaintenanceTracker::new(config.shared_incremental_maintenance);
    for (index, labeled) in specs.iter().enumerate() {
        let entry_started = Instant::now();
        let spec = &labeled.spec;
        let result = if config.shared_incremental_maintenance.is_some()
            && spec.shared_incremental_target_maintenance().is_some()
        {
            Err((
                batch_maintenance_ownership_error(),
                WasmBuildFailureDetails::specification(entry_started.elapsed()),
            ))
        } else {
            match maintenance.prepare_spec(spec) {
                Ok(configured) => build(configured.as_ref().unwrap_or(spec), index),
                Err(error) => Err((
                    error,
                    WasmBuildFailureDetails::specification(entry_started.elapsed()),
                )),
            }
        };
        entries.push(WasmBuildBatchEntry {
            index,
            label: labeled.label.clone(),
            result,
            entry_elapsed: entry_started.elapsed(),
        });
    }
    WasmBuildBatchReport {
        entries,
        input_resolution: WasmBuildBatchInputMetrics::default(),
        total: started.elapsed(),
    }
}

fn validate_input_reuse(
    reuse: Option<&WasmBuildInputSnapshotState>,
) -> Result<(), WasmBuildBatchContractError> {
    if reuse.is_some_and(WasmBuildInputSnapshotState::is_invalidated) {
        Err(WasmBuildBatchContractError::SourceLeaseInvalidated)
    } else {
        Ok(())
    }
}

struct BatchMaintenanceTracker {
    config: Option<SharedIncrementalTargetMaintenanceConfig>,
    configured_targets: HashSet<PathBuf>,
}

impl BatchMaintenanceTracker {
    fn new(config: Option<SharedIncrementalTargetMaintenanceConfig>) -> Self {
        Self {
            config,
            configured_targets: HashSet::new(),
        }
    }

    fn prepare_spec(
        &mut self,
        spec: &WasmBuildSpec,
    ) -> Result<Option<WasmBuildSpec>, WasmBuildError> {
        let Some(config) = self.config else {
            return Ok(None);
        };
        debug_assert!(spec.shared_incremental_target_maintenance().is_none());
        let Some(target_dir) = shared_incremental_target(spec) else {
            return Ok(None);
        };
        let canonical =
            canonicalize_allow_missing(&target_dir).map_err(|source| WasmBuildError::Io {
                operation: "resolve batch shared incremental target",
                path: target_dir,
                source,
            })?;
        if !self.configured_targets.insert(canonical) {
            return Ok(None);
        }
        Ok(Some(
            spec.clone()
                .with_shared_incremental_target_maintenance(config),
        ))
    }
}

fn batch_maintenance_ownership_error() -> WasmBuildError {
    WasmBuildError::InvalidSpec {
        message:
            "batch-owned shared-target maintenance cannot be combined with per-spec maintenance"
                .to_owned(),
    }
}

impl std::fmt::Display for WasmBuildBatchContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Labels(source) => write!(formatter, "Wasm batch {source}"),
            Self::SourceLeaseInvalidated => formatter
                .write_str("Wasm build source lease was invalidated by a detected input mutation"),
            Self::SpecificationNotPrepared { index, label } => write!(
                formatter,
                "Wasm batch entry {label:?} at index {index} was not declared when the input snapshot was prepared",
            ),
        }
    }
}

impl std::error::Error for WasmBuildBatchContractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Labels(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
