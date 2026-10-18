use crate::batch::{BatchLabelError, validate_labels};

use std::time::{Duration, Instant};

use super::transaction::{
    ArtifactBuildTransaction, ArtifactCacheError, ArtifactCacheOutcome, ArtifactCachePreparation,
    ArtifactCacheSpec, ArtifactCacheTimings, prepare_artifact_cache,
};

/// Caller-labeled specification for one generic artifact batch entry.
///
/// The label is report and callback identity, not artifact-cache identity. The
/// caller must keep it stable anywhere reports are composed across stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabeledArtifactCacheSpec {
    label: String,
    spec: ArtifactCacheSpec,
}

/// Ordered labeled entries from a collect-all artifact transaction batch.
#[derive(Debug)]
pub struct ArtifactCacheBatchReport<E> {
    entries: Vec<ArtifactCacheBatchEntry<E>>,
    total: Duration,
}

/// One ordered labeled result from a generic artifact batch.
#[derive(Debug)]
pub struct ArtifactCacheBatchEntry<E> {
    index: usize,
    label: String,
    result: Result<ArtifactCacheOutcome, ArtifactCacheBatchFailure<E>>,
    entry_elapsed: Duration,
}

/// Partial phase timings retained for one failed artifact batch entry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCacheBatchFailureTimings {
    preparation: Duration,
    callback: Option<Duration>,
    cleanup: Option<Duration>,
    commit: Option<Duration>,
    total: Duration,
}

/// Failure from one entry in a collect-all artifact transaction batch.
#[derive(Debug)]
pub enum ArtifactCacheBatchFailure<E> {
    /// Cache preparation failed before the population callback ran.
    Preparation {
        /// Cache preparation failure.
        source: Box<ArtifactCacheError>,
        /// Phase timings completed before the failure returned.
        timings: ArtifactCacheBatchFailureTimings,
    },
    /// Transaction commit failed after successful population.
    Commit {
        /// Cache commit failure.
        source: Box<ArtifactCacheError>,
        /// Phase timings completed before the failure returned.
        timings: ArtifactCacheBatchFailureTimings,
    },
    /// The caller's population callback failed.
    Callback {
        /// Caller population failure.
        source: Box<E>,
        /// Failure from synchronously aborting the active transaction.
        cleanup_error: Option<Box<ArtifactCacheError>>,
        /// Preparation, callback, and explicit cleanup timings.
        timings: ArtifactCacheBatchFailureTimings,
    },
}

/// Aggregate counters and successful-acquisition timings for an artifact batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCacheBatchMetrics {
    entries: usize,
    built: usize,
    reused: usize,
    successful_timings: ArtifactCacheTimings,
    total: Duration,
}

impl LabeledArtifactCacheSpec {
    /// Attach a caller-owned stable label to one artifact specification.
    #[must_use]
    pub fn new(label: impl Into<String>, spec: ArtifactCacheSpec) -> Self {
        Self {
            label: label.into(),
            spec,
        }
    }

    /// Caller-owned report and callback label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Underlying exact artifact specification.
    #[must_use]
    pub const fn spec(&self) -> &ArtifactCacheSpec {
        &self.spec
    }

    /// Consume the entry into its label and artifact specification.
    #[must_use]
    pub fn into_parts(self) -> (String, ArtifactCacheSpec) {
        (self.label, self.spec)
    }
}

impl<E> ArtifactCacheBatchReport<E> {
    /// Ordered labeled entries.
    #[must_use]
    pub fn entries(&self) -> &[ArtifactCacheBatchEntry<E>] {
        &self.entries
    }

    /// Consume the report into its ordered labeled entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<ArtifactCacheBatchEntry<E>> {
        self.entries
    }

    /// Successful entries, retaining the canonical identity, result and timings.
    pub fn outcomes(&self) -> impl Iterator<Item = &ArtifactCacheBatchEntry<E>> {
        self.entries.iter().filter(|entry| entry.is_success())
    }

    /// Failed entries; inspect `failure()` on each entry.
    pub fn failures(&self) -> impl Iterator<Item = &ArtifactCacheBatchEntry<E>> {
        self.entries.iter().filter(|entry| !entry.is_success())
    }

    /// Complete wall-clock time for the sequential collect-all batch.
    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }

    /// Whether every specification completed successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.entries.iter().all(ArtifactCacheBatchEntry::is_success)
    }

    /// Aggregate outcome counters and successful-acquisition timings.
    #[must_use]
    pub fn metrics(&self) -> ArtifactCacheBatchMetrics {
        let mut metrics = ArtifactCacheBatchMetrics {
            entries: self.entries.len(),
            total: self.total,
            ..ArtifactCacheBatchMetrics::default()
        };
        for outcome in self
            .entries
            .iter()
            .filter_map(ArtifactCacheBatchEntry::outcome)
        {
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

impl<E> ArtifactCacheBatchEntry<E> {
    /// Zero-based position in the supplied specification slice.
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
    /// Returns this entry's recorded preparation, population or publication failure.
    pub const fn result(&self) -> Result<&ArtifactCacheOutcome, &ArtifactCacheBatchFailure<E>> {
        self.result.as_ref()
    }

    /// Successful artifact outcome, when this entry succeeded.
    #[must_use]
    pub fn outcome(&self) -> Option<&ArtifactCacheOutcome> {
        self.result.as_ref().ok()
    }

    /// Structured batch failure, when this entry failed.
    #[must_use]
    pub fn failure(&self) -> Option<&ArtifactCacheBatchFailure<E>> {
        self.result.as_ref().err()
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

    /// Consume the entry into its ordered identity, result, and wall time.
    pub fn into_parts(
        self,
    ) -> (
        usize,
        String,
        Result<ArtifactCacheOutcome, ArtifactCacheBatchFailure<E>>,
        Duration,
    ) {
        (self.index, self.label, self.result, self.entry_elapsed)
    }
}

impl<E> ArtifactCacheBatchFailure<E> {
    /// Partial phase timings completed before the failure returned.
    #[must_use]
    pub const fn timings(&self) -> ArtifactCacheBatchFailureTimings {
        match self {
            Self::Preparation { timings, .. }
            | Self::Commit { timings, .. }
            | Self::Callback { timings, .. } => *timings,
        }
    }

    /// Cleanup failure after a caller population error, when one occurred.
    #[must_use]
    pub fn cleanup_error(&self) -> Option<&ArtifactCacheError> {
        match self {
            Self::Callback { cleanup_error, .. } => cleanup_error.as_deref(),
            Self::Preparation { .. } | Self::Commit { .. } => None,
        }
    }
}

impl ArtifactCacheBatchFailureTimings {
    /// Time spent in cache preparation before the failure path continued.
    #[must_use]
    pub const fn preparation(self) -> Duration {
        self.preparation
    }

    /// Time spent in the population callback, when it ran.
    #[must_use]
    pub const fn callback(self) -> Option<Duration> {
        self.callback
    }

    /// Time spent explicitly aborting after a callback failure, when attempted.
    #[must_use]
    pub const fn cleanup(self) -> Option<Duration> {
        self.cleanup
    }

    /// Time spent committing, including commit-owned failure cleanup, when attempted.
    #[must_use]
    pub const fn commit(self) -> Option<Duration> {
        self.commit
    }

    /// Complete wall-clock time retained for the failed entry.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl ArtifactCacheBatchMetrics {
    /// Number of supplied specifications.
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
    }

    /// Number of successful specifications.
    #[must_use]
    pub const fn succeeded(self) -> usize {
        self.built + self.reused
    }

    /// Number of failed specifications.
    #[must_use]
    pub const fn failed(self) -> usize {
        self.entries - self.succeeded()
    }

    /// Number of newly built artifact sets.
    #[must_use]
    pub const fn built(self) -> usize {
        self.built
    }

    /// Number of artifact sets reused from the exact cache.
    #[must_use]
    pub const fn reused(self) -> usize {
        self.reused
    }

    /// Sum of timings from successful acquisitions.
    #[must_use]
    pub const fn successful_timings(self) -> ArtifactCacheTimings {
        self.successful_timings
    }

    /// Complete wall-clock time for the sequential batch.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl<E> std::fmt::Display for ArtifactCacheBatchReport<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let metrics = self.metrics();
        write!(
            formatter,
            "entries={} succeeded={} failed={} built={} reused={} successful_timings=({}) total={:?}",
            metrics.entries(),
            metrics.succeeded(),
            metrics.failed(),
            metrics.built(),
            metrics.reused(),
            metrics.successful_timings(),
            metrics.total(),
        )
    }
}

/// Build every independent caller-labeled artifact specification.
///
/// Labels must be nonempty and unique; the complete batch is rejected before
/// work begins otherwise. The callback runs only for misses and receives the
/// stable label plus one live transaction at a time. Sequential acquisition
/// avoids self-deadlock when entries share a coordination scope. Every
/// independent result is retained, and a callback error aborts and
/// synchronously cleans that transaction before the next entry begins.
///
/// This operation is not atomic across specifications. A caller requiring one
/// all-or-nothing multi-output publication should declare those outputs on one
/// [`ArtifactCacheSpec`] instead.
///
/// # Errors
/// Returns a contract error for an empty or duplicate label before any work.
/// Per-entry acquisition, callback and commit errors are retained in the report.
pub fn build_artifact_caches_batch<E, F>(
    specs: &[LabeledArtifactCacheSpec],
    mut populate: F,
) -> Result<ArtifactCacheBatchReport<E>, BatchLabelError>
where
    F: FnMut(&str, &ArtifactBuildTransaction) -> Result<(), E>,
{
    validate_labels(specs.iter().map(LabeledArtifactCacheSpec::label))?;
    let started = Instant::now();
    let mut entries = Vec::with_capacity(specs.len());
    for (index, labeled) in specs.iter().enumerate() {
        entries.push(build_artifact_cache_entry(index, labeled, &mut populate));
    }
    Ok(ArtifactCacheBatchReport {
        entries,
        total: started.elapsed(),
    })
}

fn build_artifact_cache_entry<E>(
    index: usize,
    labeled: &LabeledArtifactCacheSpec,
    populate: &mut impl FnMut(&str, &ArtifactBuildTransaction) -> Result<(), E>,
) -> ArtifactCacheBatchEntry<E> {
    let entry_started = Instant::now();
    let mut timings = ArtifactCacheBatchFailureTimings::default();
    let mut result = (|| {
        let preparation_started = Instant::now();
        let preparation = prepare_artifact_cache(&labeled.spec);
        timings.preparation = preparation_started.elapsed();
        let transaction =
            match preparation.map_err(|source| ArtifactCacheBatchFailure::Preparation {
                source: Box::new(source),
                timings,
            })? {
                ArtifactCachePreparation::Reused(record) => {
                    return Ok(ArtifactCacheOutcome::Reused(record));
                }
                ArtifactCachePreparation::Build(transaction) => transaction,
            };

        let callback_started = Instant::now();
        let callback_result = populate(&labeled.label, &transaction);
        timings.callback = Some(callback_started.elapsed());
        if let Err(source) = callback_result {
            let cleanup_started = Instant::now();
            let cleanup_error = transaction.abort().err().map(Box::new);
            timings.cleanup = Some(cleanup_started.elapsed());
            return Err(ArtifactCacheBatchFailure::Callback {
                source: Box::new(source),
                cleanup_error,
                timings,
            });
        }

        let commit_started = Instant::now();
        let committed = transaction.commit();
        timings.commit = Some(commit_started.elapsed());
        committed.map_err(|source| ArtifactCacheBatchFailure::Commit {
            source: Box::new(source),
            timings,
        })
    })();
    let entry_elapsed = entry_started.elapsed();
    if let Err(
        ArtifactCacheBatchFailure::Preparation { timings, .. }
        | ArtifactCacheBatchFailure::Commit { timings, .. }
        | ArtifactCacheBatchFailure::Callback { timings, .. },
    ) = &mut result
    {
        timings.total = entry_elapsed;
    }
    ArtifactCacheBatchEntry {
        index,
        label: labeled.label.clone(),
        result,
        entry_elapsed,
    }
}

impl std::fmt::Display for ArtifactCacheBatchFailureTimings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "total={:?} preparation={:?} callback={:?} cleanup={:?} commit={:?}",
            self.total, self.preparation, self.callback, self.cleanup, self.commit,
        )
    }
}

impl<E: std::fmt::Display> std::fmt::Display for ArtifactCacheBatchFailure<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preparation { source, timings } => write!(
                formatter,
                "artifact cache preparation failed: {source}; timings=({timings})",
            ),
            Self::Commit { source, timings } => write!(
                formatter,
                "artifact cache commit failed: {source}; timings=({timings})",
            ),
            Self::Callback {
                source,
                cleanup_error,
                timings,
            } => {
                write!(
                    formatter,
                    "artifact callback failed: {source}; timings=({timings})"
                )?;
                if let Some(cleanup_error) = cleanup_error {
                    write!(formatter, "; cleanup also failed: {cleanup_error}")?;
                }
                Ok(())
            }
        }
    }
}

impl<E> std::error::Error for ArtifactCacheBatchFailure<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Preparation { source, .. } | Self::Commit { source, .. } => Some(source.as_ref()),
            Self::Callback { source, .. } => Some(source.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests;
