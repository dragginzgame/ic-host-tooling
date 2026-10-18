//! Complete two-canister recipe for `CachedPocketIcBaselinePool`.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ic-testkit --example multi_canister_baseline_pool
//! ```

use std::{error::Error, fmt, num::NonZeroUsize, time::Duration};

use ic_platform::Principal;
use ic_testkit::pic::{
    BaselinePoolDisposition, BaselinePreparationStage, CachedPocketIcBaseline,
    CachedPocketIcBaselinePool, CanisterSnapshotTarget, ControllerSnapshotError,
    FailureDisposition, PocketIcBaselineRecipe, PocketIcBuilder, PocketIcStartupConfig,
    PocketIcStartupError, PreparedBaseline, RebuildReason, ResetPolicies, ResetReceipt,
    ResetRequirements, SnapshotRestoreFunding, is_dead_pocket_ic_transport_error,
};

const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

struct TwoCanisterRecipe {
    requirements: ResetRequirements,
}

struct TopologyMetadata {
    canister_ids: [Principal; 2],
}

#[derive(Debug)]
enum RecipeError {
    Startup(PocketIcStartupError),
    Snapshot(ControllerSnapshotError),
    Validation(&'static str),
}

impl TwoCanisterRecipe {
    fn new() -> Self {
        Self {
            requirements: ResetRequirements {
                funding: SnapshotRestoreFunding::Preserve,
                policies: ResetPolicies::default(),
            },
        }
    }
}

impl PocketIcBaselineRecipe for TwoCanisterRecipe {
    type Metadata = TopologyMetadata;
    type Error = RecipeError;

    fn reset_requirements(&self) -> &ResetRequirements {
        &self.requirements
    }

    fn build(&self) -> Result<CachedPocketIcBaseline<Self::Metadata>, Self::Error> {
        let pocket_ic = PocketIcStartupConfig::from_env(Duration::from_secs(30))?
            .build_instance(PocketIcBuilder::new().with_application_subnet())?;
        let canister_ids = [
            pocket_ic.instance.create_canister(),
            pocket_ic.instance.create_canister(),
        ];
        for canister_id in canister_ids {
            pocket_ic
                .instance
                .install_canister(canister_id, EMPTY_WASM.to_vec(), vec![], None);
        }

        CachedPocketIcBaseline::capture(
            pocket_ic,
            canister_ids.map(|id| CanisterSnapshotTarget {
                canister_id: id,
                sender: None,
            }),
            TopologyMetadata { canister_ids },
        )
        .map_err(Into::into)
    }

    fn reset_non_snapshot_state(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ResetReceipt, Self::Error> {
        // This recipe is suitable only for tests whose relevant mutations are
        // fully contained in the two captured canisters.
        Ok(ResetReceipt::default())
    }

    fn drive_to_readiness(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
        _preparation: &PreparedBaseline,
    ) -> Result<(), Self::Error> {
        for canister_id in baseline.metadata().canister_ids {
            let status = baseline
                .pocket_ic()
                .canister_status(canister_id, None)
                .map_err(|_| RecipeError::Validation("canister status failed"))?;
            if status.module_hash.is_none() {
                return Err(RecipeError::Validation(
                    "captured canister has no installed module",
                ));
            }
        }

        Ok(())
    }

    fn classify_failure(
        &self,
        stage: BaselinePreparationStage,
        error: &(dyn std::error::Error + 'static),
    ) -> FailureDisposition {
        if is_dead_pocket_ic_transport_error(error) {
            FailureDisposition::Rebuild(RebuildReason::DeadPocketIcTransport)
        } else {
            FailureDisposition::Rebuild(stage.default_rebuild_reason())
        }
    }
}

impl From<PocketIcStartupError> for RecipeError {
    fn from(error: PocketIcStartupError) -> Self {
        Self::Startup(error)
    }
}

impl From<ControllerSnapshotError> for RecipeError {
    fn from(error: ControllerSnapshotError) -> Self {
        Self::Snapshot(error)
    }
}

impl fmt::Display for RecipeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Startup(error) => error.fmt(formatter),
            Self::Snapshot(error) => error.fmt(formatter),
            Self::Validation(message) => formatter.write_str(message),
        }
    }
}

impl Error for RecipeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Startup(error) => Some(error),
            Self::Snapshot(error) => Some(error),
            Self::Validation(_) => None,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).expect("one is nonzero"),
        TwoCanisterRecipe::new(),
    );

    let (baseline, first) = pool.acquire()?;
    assert!(matches!(first.disposition, BaselinePoolDisposition::Built));
    println!("first acquisition: {first:?}");
    drop(baseline);

    let (baseline, second) = pool.acquire()?;
    assert!(matches!(
        second.disposition,
        BaselinePoolDisposition::Restored
    ));
    println!("second acquisition: {second:?}");
    drop(baseline);

    Ok(())
}
