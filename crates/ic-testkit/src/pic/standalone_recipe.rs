use std::io;

use candid::Principal;

use super::{
    CachedPocketIcBaseline, CanisterSnapshotTarget, FailureDisposition, PocketIcBaselineRecipe,
    PreparedBaseline, RebuildReason, ResetPolicies, ResetReceipt, ResetRequirements,
    SnapshotRestoreFunding, StandaloneCanisterFixture, is_dead_pocket_ic_transport_error,
};

/// Snapshot-only recipe for one caller-built, installed canister.
///
/// Use with [`super::CachedPocketIcBaselinePool`]. The baseline metadata is the
/// installed canister ID; calls go through the baseline's borrowed PocketIC.
/// The builder owns initialization and readiness before capture and must produce
/// the same topology and seeded state for every slot of this recipe. It returns
/// `io::Result<StandaloneCanisterFixture>`; build errors retain their original
/// cause and are reported at the pool's build stage, without invoking capture.
///
/// Only the captured canister is restored. Time, other canisters, pending work
/// and external resources are not reset. Use a custom recipe or a fresh instance
/// when those mutations matter. Funding is applied immediately before restore.
/// Non-transport restore failures are fatal: the pool discards the affected
/// baseline before returning the error. Dead transport permits one recovery build.
pub struct StandaloneCanisterRecipe<B> {
    requirements: ResetRequirements,
    build: B,
}

impl<B> StandaloneCanisterRecipe<B> {
    /// Describe a single-canister recipe without invoking its builder.
    ///
    /// Static suites can initialize the recipe and pool with [`std::sync::LazyLock`].
    #[must_use]
    pub fn new(funding: SnapshotRestoreFunding, build: B) -> Self {
        Self {
            requirements: ResetRequirements {
                funding,
                policies: ResetPolicies::default(),
            },
            build,
        }
    }
}

impl<B> PocketIcBaselineRecipe for StandaloneCanisterRecipe<B>
where
    B: Fn() -> io::Result<StandaloneCanisterFixture> + Send + Sync + 'static,
{
    type Metadata = Principal;
    // Retain the typed snapshot/contract cause without another error vocabulary.
    type Error = io::Error;

    fn reset_requirements(&self) -> &ResetRequirements {
        &self.requirements
    }

    fn build(&self) -> Result<CachedPocketIcBaseline<Principal>, io::Error> {
        let (pocket_ic, canister_id) = (self.build)()?.into_parts();
        CachedPocketIcBaseline::capture(
            pocket_ic,
            [CanisterSnapshotTarget {
                canister_id,
                sender: None,
            }],
            canister_id,
        )
        .map_err(io::Error::other)
    }

    fn reset_non_snapshot_state(
        &self,
        _baseline: &CachedPocketIcBaseline<Principal>,
    ) -> Result<ResetReceipt, io::Error> {
        Ok(ResetReceipt::default())
    }

    fn drive_to_readiness(
        &self,
        _baseline: &CachedPocketIcBaseline<Principal>,
    ) -> Result<(), io::Error> {
        Ok(())
    }

    fn validate(
        &self,
        _baseline: &CachedPocketIcBaseline<Principal>,
        _preparation: &PreparedBaseline,
    ) -> Result<(), io::Error> {
        Ok(())
    }

    fn classify_failure(
        &self,
        _stage: super::BaselinePreparationStage,
        error: &(dyn std::error::Error + 'static),
    ) -> FailureDisposition {
        if is_dead_pocket_ic_transport_error(error) {
            FailureDisposition::Rebuild(RebuildReason::DeadPocketIcTransport)
        } else {
            FailureDisposition::Fatal
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;
    use crate::pic::CachedPocketIcBaselinePool;

    fn build_fixture() -> io::Result<StandaloneCanisterFixture> {
        panic!("constructing an empty pool must not invoke its builder")
    }

    #[test]
    fn runtime_capacity_preserves_lazy_fixture_construction() {
        for capacity in [1, 2, 4, 8].map(|value| NonZeroUsize::new(value).unwrap()) {
            let recipe = StandaloneCanisterRecipe::new(
                SnapshotRestoreFunding::TopUpTo { minimum_cycles: 1 },
                build_fixture,
            );
            let pool = CachedPocketIcBaselinePool::new(capacity, recipe);
            assert_eq!(pool.capacity(), capacity);
        }
    }

    #[test]
    fn builder_errors_preserve_the_cause_and_release_the_slot_for_another_attempt() {
        let recipe = StandaloneCanisterRecipe::new(SnapshotRestoreFunding::Preserve, || {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                std::fmt::Error,
            ))
        });
        let pool = CachedPocketIcBaselinePool::new(NonZeroUsize::new(1).unwrap(), recipe);
        for _ in 0..2 {
            let Err(super::super::BaselinePoolError::Preparation {
                error:
                    super::super::BaselinePoolPreparationError::Recipe {
                        stage: super::super::BaselinePreparationStage::Build,
                        source,
                    },
                timings,
            }) = pool.acquire()
            else {
                panic!("builder failure must return at the build stage");
            };
            assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
            assert!(source.get_ref().unwrap().is::<std::fmt::Error>());
            assert!(timings.build().is_some());
            assert!(timings.restore().is_none());
            assert!(timings.total() >= timings.build().unwrap());
        }
    }
}
