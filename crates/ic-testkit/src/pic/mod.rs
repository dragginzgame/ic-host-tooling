//! PocketIC host-test harness extensions.
//!
//! Import the client and DTOs directly from `ic_platform::pocket_ic`.
//! This module owns Testkit's extension traits and policy types.
//!
//! Construct and own [`ic_platform::pocket_ic::PocketIc`] instances normally, then import individual
//! extension traits or [`prelude`] for installation, diagnostics and snapshots.
//! Import the call functions, startup configuration and polling helpers explicitly.
//! Native simulator operations remain upstream inherent methods.
//!
//! [`PocketIcManagedServer`] can explicitly own one exact caller-selected
//! server child for a serial suite. No type serializes independent instances or
//! downloads a server implicitly. On Unix, `server_assets` owns explicit
//! authenticated provisioning and admission; the published
//! `ic-testkit-server setup` / `check` commands delegate to that implementation.

#[cfg(feature = "reply-fixtures")]
pub mod fixtures;

mod server;
pub use server::{POCKET_IC_SERVER_VERSION, supports_pocket_ic_server};
#[cfg(unix)]
pub mod server_assets;

mod baseline;
mod baseline_pool;
mod bounded_pool;
mod diagnostics;
mod errors;
mod lifecycle;
mod snapshot;
mod standalone;
mod standalone_recipe;
mod startup;
mod time;
mod transport;
#[cfg(feature = "wire")]
mod wire;

pub use baseline::CachedPocketIcBaseline;
pub use baseline_pool::{
    BaselinePoolContractError, BaselinePoolDisposition, BaselinePoolError, BaselinePoolOutcome,
    BaselinePoolPreparationError, BaselinePoolTimings, BaselinePreparationStage,
    CachedPocketIcBaselinePool, CachedPocketIcBaselinePoolGuard, ExtraCanisterPolicy,
    FailureDisposition, PocketIcBaselineRecipe, PreparedBaseline, RebuildReason, ResetDomainKind,
    ResetPolicies, ResetReceipt, ResetRequirements, StateResetPolicy, TimeResetPolicy,
};
pub use diagnostics::{
    CanisterDiagnosticLogRecord, CanisterDiagnosticLogs, CanisterDiagnosticsBatchEntry,
    CanisterDiagnosticsBatchReport, CanisterDiagnosticsReport, CanisterDiagnosticsRequest,
    CanisterLogRenderLimits, DEFAULT_CANISTER_LOG_BYTE_LIMIT, DEFAULT_CANISTER_LOG_RECORD_LIMIT,
    LabeledCanisterDiagnosticsRequest, PocketIcDiagnosticsExt,
};
pub use errors::{CanisterInstallError, StandaloneCanisterInstallError};
pub use lifecycle::{CanisterInstallExt, InstallSpec, RetryPolicy};
pub use snapshot::{
    CanisterSnapshotTarget, ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt,
    SnapshotCleanupFailure, SnapshotRestoreFunding,
};
pub use standalone::StandaloneCanisterFixture;
pub use standalone_recipe::StandaloneCanisterRecipe;
pub use startup::{
    PocketIcInstance, PocketIcManagedServer, PocketIcManagedServerOutput, PocketIcStartupConfig,
    PocketIcStartupError, PocketIcStartupFailure,
};
pub use time::{TickUntilError, advance_and_tick, tick_until};
pub use transport::{
    PocketIcCallFailure, PocketIcOperationError, is_dead_pocket_ic_transport_error,
    try_pocket_ic_operation,
};
#[cfg(feature = "wire")]
pub use wire::{WireCallContext, WireCallError, query_wire, update_wire};

/// All PocketIC extension traits, and no data types.
///
/// Importing this module keeps policy/data types explicit while avoiding
/// repeated trait lists across a large integration-test crate.
pub mod prelude {
    pub use super::{CanisterInstallExt, PocketIcDiagnosticsExt, PocketIcSnapshotExt};
}
