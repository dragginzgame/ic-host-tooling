use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    time::{Duration, Instant},
};

use super::{
    CachedPocketIcBaseline, ControllerSnapshotError, SnapshotRestoreFunding,
    bounded_pool::{BoundedSlotLease, BoundedSlotPool},
};

/// Reset domain whose handling is declared by a pooled baseline recipe.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResetDomainKind {
    /// PocketIC simulated time.
    PocketIcTime,
    /// Canisters outside the captured baseline set.
    ExtraCanisters,
    /// Pending ingress, timers, or cross-canister messages.
    PendingMessages,
    /// Subnet metrics, routing, allocation, or other subnet-global state.
    SubnetState,
    /// Files, processes, services, or other caller-owned resources.
    ExternalResources,
}

/// PocketIC time handling required or achieved by one reset.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeResetPolicy {
    /// Preserve the current simulator time rather than claiming it was reset.
    PreserveCurrent,
    /// Restore the exact time recorded by the recipe baseline.
    RestoreBaseline,
    /// Treat any relevant time mutation as requiring slot reconstruction.
    RebuildOnMutation,
}

/// Extra-canister handling required or achieved by one reset.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtraCanisterPolicy {
    /// Validate that the baseline canister set is unchanged.
    RequireBaselineSet,
    /// Remove canisters explicitly tracked by the recipe.
    RemoveTracked,
    /// Treat any extra-canister change as requiring slot reconstruction.
    RebuildOnChange,
}

/// Generic handling for reset domains without a more specific policy.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateResetPolicy {
    /// Reset the domain through caller-owned recipe logic.
    ResetByRecipe,
    /// Validate that the domain remained unchanged.
    ValidateUnchanged,
    /// Explicitly declare the domain irrelevant to this recipe's guarantees.
    IrrelevantByRecipeContract,
    /// Treat any relevant change as requiring slot reconstruction.
    RebuildOnChange,
}

/// Policy for one non-snapshot reset domain.
///
/// [`ResetRequirements`] declares policies that a recipe requires;
/// [`ResetReceipt`] reports policies achieved by its reset operation. The pool
/// checks every required domain against the reported policy before reuse.
///
/// ```
/// use ic_testkit::pic::{
///     SnapshotRestoreFunding, ResetDomainKind, ResetDomainPolicy, ResetReceipt,
///     ResetRequirements, TimeResetPolicy,
/// };
///
/// let requirements = ResetRequirements::try_new(
///     SnapshotRestoreFunding::Preserve,
///     [ResetDomainPolicy::PocketIcTime(TimeResetPolicy::PreserveCurrent)],
/// )?;
/// // The reset operation deliberately preserves simulated time.
/// let receipt = ResetReceipt::try_new([
///     ResetDomainPolicy::PocketIcTime(TimeResetPolicy::PreserveCurrent),
/// ])?;
/// assert_eq!(
///     requirements.get(ResetDomainKind::PocketIcTime),
///     receipt.get(ResetDomainKind::PocketIcTime),
/// );
/// # Ok::<(), ic_testkit::pic::BaselinePoolContractError>(())
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResetDomainPolicy {
    /// PocketIC time policy.
    PocketIcTime(TimeResetPolicy),
    /// Extra-canister policy.
    ExtraCanisters(ExtraCanisterPolicy),
    /// Pending-message policy.
    PendingMessages(StateResetPolicy),
    /// Subnet-state policy.
    SubnetState(StateResetPolicy),
    /// External-resource policy.
    ExternalResources(StateResetPolicy),
}

/// Typed reset guarantees required by one fixture recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetRequirements {
    funding: SnapshotRestoreFunding,
    domains: BTreeMap<ResetDomainKind, ResetDomainPolicy>,
}

/// Typed reset guarantees achieved by one preparation pass.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResetReceipt(BTreeMap<ResetDomainKind, ResetDomainPolicy>);

/// Contract failure while constructing or verifying recipe reset evidence.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaselinePoolContractError {
    /// A reset domain was declared more than once.
    DuplicateResetDomain { domain: ResetDomainKind },
    /// A required reset domain had no matching achievement.
    MissingResetDomain { domain: ResetDomainKind },
    /// A reset achievement did not satisfy the required policy.
    ResetPolicyMismatch {
        requirement: ResetDomainPolicy,
        achievement: ResetDomainPolicy,
    },
}

/// Whether validation is observing a newly built or restored baseline.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreparedBaseline {
    /// The recipe just built this baseline.
    Built,
    /// The pool restored an existing baseline and the recipe reset its other state.
    Restored {
        /// Typed non-snapshot reset receipt.
        reset: ResetReceipt,
    },
}

/// Recipe stage associated with a structured preparation failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselinePreparationStage {
    /// Constructing a new baseline.
    Build,
    /// Restoring captured canisters.
    RestoreCanisters,
    /// Resetting state outside the snapshots.
    ResetNonSnapshotState,
    /// Driving the restored topology to readiness.
    DriveToReadiness,
    /// Validating a newly built baseline.
    ValidateBuilt,
    /// Validating a restored baseline.
    ValidateRestored,
}

/// Why an invalid or failed slot was reconstructed.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebuildReason {
    /// PocketIC transport was no longer reachable.
    DeadPocketIcTransport,
    /// Captured snapshot restoration failed.
    SnapshotRestoreFailure,
    /// Non-snapshot reset failed.
    ResetFailure,
    /// Readiness or quiescence could not be established.
    ReadinessFailure,
    /// Required and achieved reset domains did not match.
    ResetCoverageMismatch,
    /// Final invariant validation failed.
    InvariantValidationFailure,
    /// A caller explicitly invalidated its lease.
    ExplicitLeaseInvalidation,
    /// A lease was dropped while its thread was unwinding.
    UnwindWhileLeased,
    /// Recipe-specific structured reason.
    RecipeClassified { code: String },
}

/// Recipe decision for a failed restored-slot preparation stage.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FailureDisposition {
    /// Return the failure without rebuilding during this acquisition.
    Fatal,
    /// Invalidate and rebuild the slot once.
    Rebuild(RebuildReason),
}

/// Timings for one baseline-pool acquisition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BaselinePoolTimings {
    wait: Duration,
    build: Option<Duration>,
    restore: Option<Duration>,
    reset: Option<Duration>,
    readiness: Option<Duration>,
    validation: Option<Duration>,
    stale_teardown: Option<Duration>,
    total: Duration,
}

/// One successful acquisition's slot, timings and preparation disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselinePoolOutcome {
    /// Diagnostic slot index.
    pub slot: usize,
    /// Acquisition phase timings.
    pub timings: BaselinePoolTimings,
    /// Whether this acquisition built, restored or rebuilt the baseline.
    pub disposition: BaselinePoolDisposition,
}

/// Whether a baseline-pool lease was built, restored, or rebuilt.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaselinePoolDisposition {
    /// An empty slot was constructed and validated.
    Built,
    /// An existing slot was restored, reset, and validated.
    Restored,
    /// An invalid or failed slot was reconstructed and validated.
    Rebuilt(RebuildReason),
}

/// One failed recipe or contract stage while preparing a baseline slot.
#[non_exhaustive]
#[derive(Debug)]
pub enum BaselinePoolPreparationError<E> {
    /// Caller-owned recipe logic returned an error.
    Recipe {
        /// Failed lifecycle stage.
        stage: BaselinePreparationStage,
        /// Caller-owned structured source error.
        source: E,
    },
    /// Canonical restoration of the captured snapshot set failed.
    Restore(ControllerSnapshotError),
    /// Typed non-snapshot reset evidence violated the pool contract.
    Contract(BaselinePoolContractError),
}

/// Failure to acquire a validated baseline-pool lease.
#[non_exhaustive]
#[derive(Debug)]
pub enum BaselinePoolError<E> {
    /// Initial construction or a non-rebuilt preparation failed.
    Preparation {
        /// Recipe or contract failure that stopped acquisition.
        error: BaselinePoolPreparationError<E>,
        /// Phase timings recorded before acquisition failed.
        timings: Box<BaselinePoolTimings>,
    },
    /// Reused-slot preparation failed and its one rebuild attempt also failed.
    RecoveryFailed {
        /// Original restore/reset/readiness/validation failure.
        original: Box<BaselinePoolPreparationError<E>>,
        /// Failure while rebuilding or validating the replacement.
        rebuild: Box<BaselinePoolPreparationError<E>>,
        /// Combined timings for preparation, stale teardown, and rebuilding.
        timings: Box<BaselinePoolTimings>,
    },
}

/// Caller-owned build, preflight and non-snapshot reset policy for one pooled baseline.
pub trait PocketIcBaselineRecipe: Send + Sync + 'static {
    /// Metadata retained beside every baseline owned by this recipe.
    type Metadata: Send + 'static;
    /// Structured caller error shared by recipe lifecycle stages.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Reset guarantees required before an existing slot may be reused.
    fn reset_requirements(&self) -> &ResetRequirements;

    /// Construct and capture one complete baseline.
    fn build(&self) -> Result<CachedPocketIcBaseline<Self::Metadata>, Self::Error>;

    /// Admit recipe-specific preconditions before the pool restores snapshots.
    ///
    /// Most recipes need no preflight. Checks such as exact current cycle-balance
    /// guards run here, before restoration can charge cycles. This cannot replace
    /// the pool's canonical restore or manufacture its completion evidence.
    fn prepare_restore(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Reset state not covered by canister snapshots.
    fn reset_non_snapshot_state(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ResetReceipt, Self::Error>;

    /// Drive the topology to the recipe's readiness boundary.
    /// Successful completion permits final invariant validation.
    fn drive_to_readiness(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<(), Self::Error>;

    /// Validate the same baseline invariants after build and restore.
    /// Success admits this pool-owned recipe's baseline for leasing.
    fn validate(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
        preparation: &PreparedBaseline,
    ) -> Result<(), Self::Error>;

    /// Classify a restored-slot recipe failure as fatal or rebuildable.
    fn classify_failure(
        &self,
        stage: BaselinePreparationStage,
        _error: &(dyn std::error::Error + 'static),
    ) -> FailureDisposition {
        FailureDisposition::Rebuild(stage.default_rebuild_reason())
    }
}

/// Caller-owned runtime-capacity pool of independently restorable PocketIC baselines.
///
/// One pool structurally owns one [`PocketIcBaselineRecipe`]. A warm
/// acquisition restores the complete captured canister set, applies the
/// recipe's non-snapshot reset, reaches its readiness boundary, checks typed
/// reset coverage, and validates final invariants before exposing a lease.
/// Each capacity slot owns an independent PocketIC instance.
///
/// Snapshot reuse is not a complete PocketIC rollback. The recipe must account
/// for time, extra canisters, pending messages, subnet state, cycles, and
/// external resources when those domains matter to its tests.
pub struct CachedPocketIcBaselinePool<R>
where
    R: PocketIcBaselineRecipe,
{
    recipe: R,
    slots: BoundedSlotPool<CachedPocketIcBaseline<R::Metadata>, RebuildReason>,
}

/// Exclusive lease of one validated pooled PocketIC baseline.
pub struct CachedPocketIcBaselinePoolGuard<'a, R>
where
    R: PocketIcBaselineRecipe,
{
    slot: BoundedSlotLease<'a, CachedPocketIcBaseline<R::Metadata>, RebuildReason>,
}

impl ResetDomainPolicy {
    /// Domain governed by this policy.
    #[must_use]
    pub const fn domain(&self) -> ResetDomainKind {
        match self {
            Self::PocketIcTime(_) => ResetDomainKind::PocketIcTime,
            Self::ExtraCanisters(_) => ResetDomainKind::ExtraCanisters,
            Self::PendingMessages(_) => ResetDomainKind::PendingMessages,
            Self::SubnetState(_) => ResetDomainKind::SubnetState,
            Self::ExternalResources(_) => ResetDomainKind::ExternalResources,
        }
    }
}

fn collect_reset_domains(
    policies: impl IntoIterator<Item = ResetDomainPolicy>,
) -> Result<BTreeMap<ResetDomainKind, ResetDomainPolicy>, BaselinePoolContractError> {
    let mut domains = BTreeMap::new();
    for policy in policies {
        let domain = policy.domain();
        if domains.insert(domain, policy).is_some() {
            return Err(BaselinePoolContractError::DuplicateResetDomain { domain });
        }
    }
    Ok(domains)
}

impl ResetRequirements {
    /// Construct a duplicate-checked reset requirement set.
    ///
    /// The pool restores the complete captured set using `funding`.
    /// `requirements` describe only non-snapshot domains.
    pub fn try_new<I>(
        funding: SnapshotRestoreFunding,
        requirements: I,
    ) -> Result<Self, BaselinePoolContractError>
    where
        I: IntoIterator<Item = ResetDomainPolicy>,
    {
        Ok(Self {
            funding,
            domains: collect_reset_domains(requirements)?,
        })
    }

    /// Funding selected once for the pool's canonical snapshot restore.
    #[must_use]
    pub const fn funding(&self) -> SnapshotRestoreFunding {
        self.funding
    }

    /// Read the requirement for one domain.
    #[must_use]
    pub fn get(&self, domain: ResetDomainKind) -> Option<&ResetDomainPolicy> {
        self.domains.get(&domain)
    }

    /// Iterate over requirements in deterministic domain order.
    pub fn iter(&self) -> impl Iterator<Item = &ResetDomainPolicy> {
        self.domains.values()
    }

    fn verify(&self, receipt: &ResetReceipt) -> Result<(), BaselinePoolContractError> {
        for (domain, requirement) in &self.domains {
            let Some(achievement) = receipt.0.get(domain) else {
                return Err(BaselinePoolContractError::MissingResetDomain { domain: *domain });
            };
            if achievement != requirement {
                return Err(BaselinePoolContractError::ResetPolicyMismatch {
                    requirement: requirement.clone(),
                    achievement: achievement.clone(),
                });
            }
        }
        Ok(())
    }
}

impl ResetReceipt {
    /// Construct a duplicate-checked non-snapshot reset achievement set.
    ///
    /// Snapshot restoration is performed and recorded by the pool itself.
    pub fn try_new<I>(achievements: I) -> Result<Self, BaselinePoolContractError>
    where
        I: IntoIterator<Item = ResetDomainPolicy>,
    {
        Ok(Self(collect_reset_domains(achievements)?))
    }

    /// Create an empty receipt for recipes with no non-snapshot reset achievements.
    #[must_use]
    pub const fn empty() -> Self {
        Self(BTreeMap::new())
    }

    /// Read the achievement for one domain.
    #[must_use]
    pub fn get(&self, domain: ResetDomainKind) -> Option<&ResetDomainPolicy> {
        self.0.get(&domain)
    }

    /// Iterate over achievements in deterministic domain order.
    pub fn iter(&self) -> impl Iterator<Item = &ResetDomainPolicy> {
        self.0.values()
    }
}

impl BaselinePreparationStage {
    /// Default rebuild reason used by [`PocketIcBaselineRecipe::classify_failure`].
    ///
    /// Recipes that override classification can use this for their fallback
    /// after handling a more specific error such as dead PocketIC transport.
    #[must_use]
    pub fn default_rebuild_reason(self) -> RebuildReason {
        match self {
            Self::RestoreCanisters => RebuildReason::SnapshotRestoreFailure,
            Self::ResetNonSnapshotState => RebuildReason::ResetFailure,
            Self::DriveToReadiness => RebuildReason::ReadinessFailure,
            Self::ValidateRestored | Self::ValidateBuilt => {
                RebuildReason::InvariantValidationFailure
            }
            Self::Build => RebuildReason::RecipeClassified {
                code: "build".to_owned(),
            },
        }
    }
}

impl BaselinePoolTimings {
    /// Time spent waiting for a capacity slot.
    #[must_use]
    pub const fn wait(self) -> Duration {
        self.wait
    }

    /// Time spent constructing a new baseline.
    #[must_use]
    pub const fn build(self) -> Option<Duration> {
        self.build
    }

    /// Time spent restoring captured canisters.
    #[must_use]
    pub const fn restore(self) -> Option<Duration> {
        self.restore
    }

    /// Time spent resetting non-snapshot state.
    #[must_use]
    pub const fn reset(self) -> Option<Duration> {
        self.reset
    }

    /// Time spent driving the topology to readiness.
    #[must_use]
    pub const fn readiness(self) -> Option<Duration> {
        self.readiness
    }

    /// Time spent validating final baseline invariants.
    #[must_use]
    pub const fn validation(self) -> Option<Duration> {
        self.validation
    }

    /// Time spent dropping an invalid baseline before rebuilding.
    #[must_use]
    pub const fn stale_teardown(self) -> Option<Duration> {
        self.stale_teardown
    }

    /// Complete acquisition duration.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl std::fmt::Display for BaselinePoolTimings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "total={:?} wait={:?} build={:?} restore={:?} reset={:?} readiness={:?} validation={:?} stale_teardown={:?}",
            self.total,
            self.wait,
            self.build,
            self.restore,
            self.reset,
            self.readiness,
            self.validation,
            self.stale_teardown,
        )
    }
}

impl std::fmt::Display for BaselinePoolOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.disposition {
            BaselinePoolDisposition::Built => formatter.write_str("built")?,
            BaselinePoolDisposition::Restored => formatter.write_str("restored")?,
            BaselinePoolDisposition::Rebuilt(reason) => {
                write!(
                    formatter,
                    "rebuilt slot={} reason={reason:?} {}",
                    self.slot, self.timings
                )?;
                return Ok(());
            }
        }
        write!(formatter, " slot={} {}", self.slot, self.timings)
    }
}

impl<E> BaselinePoolError<E> {
    /// Timings recorded before this acquisition failed.
    #[must_use]
    pub const fn timings(&self) -> BaselinePoolTimings {
        match self {
            Self::Preparation { timings, .. } | Self::RecoveryFailed { timings, .. } => **timings,
        }
    }
}

impl<R> CachedPocketIcBaselinePool<R>
where
    R: PocketIcBaselineRecipe,
{
    /// Create a runtime-capacity pool that structurally owns one recipe.
    ///
    /// Construction is const and lazy: slots are allocated on first acquisition,
    /// and the recipe builds a baseline only when a slot needs one.
    #[must_use]
    pub const fn new(capacity: NonZeroUsize, recipe: R) -> Self {
        Self {
            recipe,
            slots: BoundedSlotPool::new(capacity),
        }
    }

    /// Maximum number of simultaneously leased PocketIC baselines.
    #[must_use]
    pub const fn capacity(&self) -> NonZeroUsize {
        self.slots.capacity()
    }

    /// Acquire one fully built or restored and validated baseline lease.
    ///
    /// A rebuildable warm-preparation failure discards the stale slot and
    /// performs at most one build attempt. Recipe and caller panics are never
    /// converted into cache misses; the lease is invalidated while the panic
    /// continues unwinding.
    ///
    /// # Errors
    ///
    /// Returns a stage-specific recipe or contract error. If warm preparation
    /// and its one recovery build both fail, the error preserves both causes.
    pub fn acquire(
        &self,
    ) -> Result<
        (CachedPocketIcBaselinePoolGuard<'_, R>, BaselinePoolOutcome),
        BaselinePoolError<R::Error>,
    > {
        let total_started = Instant::now();
        let mut slot = self.slots.acquire();
        let mut timings = BaselinePoolTimings {
            wait: slot.wait(),
            ..BaselinePoolTimings::default()
        };
        let slot_index = slot.slot_index();

        let (rebuild_reason, original) = if slot.is_reusable() {
            match self.prepare_reused(&slot, &mut timings) {
                Ok(()) => {
                    timings.total = total_started.elapsed();
                    return Ok((
                        CachedPocketIcBaselinePoolGuard { slot },
                        BaselinePoolOutcome {
                            slot: slot_index,
                            timings,
                            disposition: BaselinePoolDisposition::Restored,
                        },
                    ));
                }
                Err(original) => match self.failure_disposition(&original) {
                    FailureDisposition::Fatal => {
                        // A failed restore may have partially changed the instance.
                        // Discard it without treating a fatal failure as a rebuild.
                        Self::discard_stale_slot(&mut slot, &mut timings);
                        timings.total = total_started.elapsed();
                        return Err(BaselinePoolError::Preparation {
                            error: original,
                            timings: Box::new(timings),
                        });
                    }
                    FailureDisposition::Rebuild(reason) => (Some(reason), Some(original)),
                },
            }
        } else {
            let reason = if slot.invalidated_by_unwind() {
                Some(RebuildReason::UnwindWhileLeased)
            } else {
                slot.invalidation_reason().cloned()
            };
            (reason, None)
        };

        if slot.is_populated() {
            Self::discard_stale_slot(&mut slot, &mut timings);
        }
        let build = self.build_slot(&mut slot, &mut timings);
        timings.total = total_started.elapsed();
        if let Err(error) = build {
            return Err(match original {
                Some(original) => BaselinePoolError::RecoveryFailed {
                    original: Box::new(original),
                    rebuild: Box::new(error),
                    timings: Box::new(timings),
                },
                None => BaselinePoolError::Preparation {
                    error,
                    timings: Box::new(timings),
                },
            });
        }

        let outcome = BaselinePoolOutcome {
            slot: slot_index,
            timings,
            disposition: rebuild_reason.map_or(
                BaselinePoolDisposition::Built,
                BaselinePoolDisposition::Rebuilt,
            ),
        };
        Ok((CachedPocketIcBaselinePoolGuard { slot }, outcome))
    }

    fn prepare_reused(
        &self,
        slot: &BoundedSlotLease<'_, CachedPocketIcBaseline<R::Metadata>, RebuildReason>,
        timings: &mut BaselinePoolTimings,
    ) -> Result<(), BaselinePoolPreparationError<R::Error>> {
        let baseline = slot
            .get()
            .expect("reusable baseline slot must be populated");

        let started = Instant::now();
        let restore = self
            .recipe
            .prepare_restore(baseline)
            .map_err(|source| BaselinePoolPreparationError::Recipe {
                stage: BaselinePreparationStage::RestoreCanisters,
                source,
            })
            .and_then(|()| {
                baseline
                    .restore(self.recipe.reset_requirements().funding())
                    .map_err(BaselinePoolPreparationError::Restore)
            });
        add_timing(&mut timings.restore, started.elapsed());
        restore?;

        let started = Instant::now();
        let reset_result = self.recipe.reset_non_snapshot_state(baseline);
        add_timing(&mut timings.reset, started.elapsed());
        let reset = reset_result.map_err(|source| BaselinePoolPreparationError::Recipe {
            stage: BaselinePreparationStage::ResetNonSnapshotState,
            source,
        })?;

        let started = Instant::now();
        let readiness_result = self.recipe.drive_to_readiness(baseline);
        add_timing(&mut timings.readiness, started.elapsed());
        readiness_result.map_err(|source| BaselinePoolPreparationError::Recipe {
            stage: BaselinePreparationStage::DriveToReadiness,
            source,
        })?;
        self.recipe
            .reset_requirements()
            .verify(&reset)
            .map_err(BaselinePoolPreparationError::Contract)?;

        let preparation = PreparedBaseline::Restored { reset };
        self.validate_baseline(
            baseline,
            &preparation,
            BaselinePreparationStage::ValidateRestored,
            timings,
        )?;
        Ok(())
    }

    fn build_slot(
        &self,
        slot: &mut BoundedSlotLease<'_, CachedPocketIcBaseline<R::Metadata>, RebuildReason>,
        timings: &mut BaselinePoolTimings,
    ) -> Result<(), BaselinePoolPreparationError<R::Error>> {
        let started = Instant::now();
        let build = self.recipe.build();
        add_timing(&mut timings.build, started.elapsed());
        let baseline = build.map_err(|source| BaselinePoolPreparationError::Recipe {
            stage: BaselinePreparationStage::Build,
            source,
        })?;

        if let Err(error) = self.validate_baseline(
            &baseline,
            &PreparedBaseline::Built,
            BaselinePreparationStage::ValidateBuilt,
            timings,
        ) {
            drop_baseline_safely(baseline);
            return Err(error);
        }
        let replaced = slot.replace(baseline);
        debug_assert!(replaced.is_none());
        Ok(())
    }

    fn validate_baseline(
        &self,
        baseline: &CachedPocketIcBaseline<R::Metadata>,
        preparation: &PreparedBaseline,
        stage: BaselinePreparationStage,
        timings: &mut BaselinePoolTimings,
    ) -> Result<(), BaselinePoolPreparationError<R::Error>> {
        let started = Instant::now();
        let validation = self.recipe.validate(baseline, preparation);
        add_timing(&mut timings.validation, started.elapsed());
        validation.map_err(|source| BaselinePoolPreparationError::Recipe { stage, source })
    }

    fn failure_disposition(
        &self,
        error: &BaselinePoolPreparationError<R::Error>,
    ) -> FailureDisposition {
        match error {
            BaselinePoolPreparationError::Recipe { stage, source } => {
                self.recipe.classify_failure(*stage, source)
            }
            BaselinePoolPreparationError::Restore(source) => self
                .recipe
                .classify_failure(BaselinePreparationStage::RestoreCanisters, source),
            BaselinePoolPreparationError::Contract(_) => {
                FailureDisposition::Rebuild(RebuildReason::ResetCoverageMismatch)
            }
        }
    }

    fn discard_stale_slot(
        slot: &mut BoundedSlotLease<'_, CachedPocketIcBaseline<R::Metadata>, RebuildReason>,
        timings: &mut BaselinePoolTimings,
    ) {
        let started = Instant::now();
        if let Some(stale) = slot.take() {
            drop_baseline_safely(stale);
        }
        timings.stale_teardown = Some(started.elapsed());
    }
}

impl<R> CachedPocketIcBaselinePoolGuard<'_, R>
where
    R: PocketIcBaselineRecipe,
{
    /// Diagnostic slot index held by this lease.
    #[must_use]
    pub const fn slot(&self) -> usize {
        self.slot.slot_index()
    }

    /// Mark this slot non-reusable with a structured rebuild reason.
    pub fn invalidate(&mut self, reason: RebuildReason) {
        self.slot.invalidate(reason);
    }
}

impl<R> Deref for CachedPocketIcBaselinePoolGuard<'_, R>
where
    R: PocketIcBaselineRecipe,
{
    type Target = CachedPocketIcBaseline<R::Metadata>;

    fn deref(&self) -> &Self::Target {
        self.slot
            .get()
            .expect("leased baseline pool slot must be populated")
    }
}

const fn add_timing(total: &mut Option<Duration>, elapsed: Duration) {
    *total = Some(match *total {
        None => elapsed,
        Some(previous) => previous.saturating_add(elapsed),
    });
}

fn drop_baseline_safely<M>(baseline: CachedPocketIcBaseline<M>) {
    let _ = catch_unwind(AssertUnwindSafe(|| drop(baseline)));
}

impl std::fmt::Display for BaselinePreparationStage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Build => "build",
            Self::RestoreCanisters => "canister restore",
            Self::ResetNonSnapshotState => "non-snapshot reset",
            Self::DriveToReadiness => "readiness",
            Self::ValidateBuilt => "built-baseline validation",
            Self::ValidateRestored => "restored-baseline validation",
        })
    }
}

impl std::fmt::Display for BaselinePoolContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateResetDomain { domain } => {
                write!(
                    formatter,
                    "reset domain {domain:?} was reported more than once"
                )
            }
            Self::MissingResetDomain { domain } => {
                write!(
                    formatter,
                    "required reset domain {domain:?} was not achieved"
                )
            }
            Self::ResetPolicyMismatch {
                requirement,
                achievement,
            } => write!(
                formatter,
                "reset achievement {achievement:?} does not satisfy {requirement:?}",
            ),
        }
    }
}

impl std::error::Error for BaselinePoolContractError {}

impl<E> std::fmt::Display for BaselinePoolPreparationError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recipe { stage, source } => {
                write!(formatter, "baseline {stage} failed: {source}")
            }
            Self::Restore(source) => {
                write!(formatter, "baseline canister restore failed: {source}")
            }
            Self::Contract(error) => write!(formatter, "baseline pool contract failed: {error}"),
        }
    }
}

impl<E> std::error::Error for BaselinePoolPreparationError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Recipe { source, .. } => Some(source),
            Self::Restore(source) => Some(source),
            Self::Contract(error) => Some(error),
        }
    }
}

impl<E> std::fmt::Display for BaselinePoolError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preparation { error, .. } => error.fmt(formatter),
            Self::RecoveryFailed {
                original, rebuild, ..
            } => write!(
                formatter,
                "baseline preparation failed ({original}); rebuilding the slot also failed: {rebuild}",
            ),
        }
    }
}

impl<E> std::error::Error for BaselinePoolError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Preparation { error, .. } => Some(error),
            Self::RecoveryFailed { original, .. } => Some(original.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BaselinePoolContractError, BaselinePoolTimings, ExtraCanisterPolicy, ResetDomainKind,
        ResetDomainPolicy, ResetReceipt, ResetRequirements, SnapshotRestoreFunding,
        StateResetPolicy, TimeResetPolicy, add_timing,
    };
    use std::time::Duration;

    #[test]
    fn phase_timing_preserves_absence_zero_and_saturation() {
        let mut timings = BaselinePoolTimings::default();
        assert_eq!(timings.restore(), None);
        add_timing(&mut timings.restore, Duration::ZERO);
        assert_eq!(timings.restore(), Some(Duration::ZERO));
        add_timing(&mut timings.restore, Duration::from_secs(2));
        assert_eq!(timings.restore(), Some(Duration::from_secs(2)));
        add_timing(&mut timings.restore, Duration::MAX);
        assert_eq!(timings.restore(), Some(Duration::MAX));
        assert_eq!(timings.build(), None);
    }

    #[test]
    fn reset_declarations_and_receipts_reject_duplicate_domains() {
        let policies = [
            ResetDomainPolicy::PocketIcTime(TimeResetPolicy::PreserveCurrent),
            ResetDomainPolicy::PocketIcTime(TimeResetPolicy::RebuildOnMutation),
        ];
        let error = BaselinePoolContractError::DuplicateResetDomain {
            domain: ResetDomainKind::PocketIcTime,
        };
        assert_eq!(
            ResetRequirements::try_new(SnapshotRestoreFunding::Preserve, policies.clone()),
            Err(error.clone()),
        );
        assert_eq!(ResetReceipt::try_new(policies), Err(error));
    }

    #[test]
    fn required_policy_must_match_achieved_policy() {
        for (requirement, achievement) in [
            (
                ResetDomainPolicy::PocketIcTime(TimeResetPolicy::PreserveCurrent),
                ResetDomainPolicy::PocketIcTime(TimeResetPolicy::RebuildOnMutation),
            ),
            (
                ResetDomainPolicy::ExtraCanisters(ExtraCanisterPolicy::RequireBaselineSet),
                ResetDomainPolicy::ExtraCanisters(ExtraCanisterPolicy::RemoveTracked),
            ),
            (
                ResetDomainPolicy::PendingMessages(StateResetPolicy::ValidateUnchanged),
                ResetDomainPolicy::PendingMessages(StateResetPolicy::IrrelevantByRecipeContract),
            ),
            (
                ResetDomainPolicy::SubnetState(StateResetPolicy::ResetByRecipe),
                ResetDomainPolicy::SubnetState(StateResetPolicy::RebuildOnChange),
            ),
            (
                ResetDomainPolicy::ExternalResources(StateResetPolicy::ValidateUnchanged),
                ResetDomainPolicy::ExternalResources(StateResetPolicy::ResetByRecipe),
            ),
        ] {
            let requirements =
                ResetRequirements::try_new(SnapshotRestoreFunding::Preserve, [requirement.clone()])
                    .unwrap();
            let receipt = ResetReceipt::try_new([requirement.clone()]).unwrap();
            assert_eq!(requirements.verify(&receipt), Ok(()));
            assert_eq!(
                requirements.verify(&ResetReceipt::empty()),
                Err(BaselinePoolContractError::MissingResetDomain {
                    domain: requirement.domain(),
                }),
            );
            let receipt = ResetReceipt::try_new([achievement.clone()]).unwrap();
            assert_eq!(
                requirements.verify(&receipt),
                Err(BaselinePoolContractError::ResetPolicyMismatch {
                    requirement,
                    achievement,
                }),
            );
        }
    }
}
