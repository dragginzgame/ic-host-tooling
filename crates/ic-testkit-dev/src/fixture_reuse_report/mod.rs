//! Shared workload report fields; execution, admission and summaries remain caller-owned.

use serde::{Deserialize, Serialize};

use crate::fixture_reuse_protocol::Mode;

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_field_names,
    reason = "serialized field names explicitly carry their millisecond units"
)]
pub struct Phases {
    pub(super) wait_ms: f64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) build_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) restore_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) reset_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) readiness_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) validation_ms: Option<f64>,
    pub(super) acquisition_ms: f64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub(super) iteration: usize,
    pub(super) phases: Phases,
    pub(super) body_ms: f64,
    pub(super) release_ms: f64,
    pub(super) total_ms: f64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerReport {
    pub(super) mode: Mode,
    pub(super) capacity: usize,
    pub(super) iterations: usize,
    pub(super) workers: usize,
    pub(super) state_bytes_per_canister: u32,
    pub(super) wall_ms: f64,
    pub(super) fixture_wall_ms: f64,
    pub(super) samples: Vec<Sample>,
    pub(super) server_startup_ms: f64,
    pub(super) cold_preparation: Vec<Phases>,
    pub(super) preparation_wall_ms: f64,
    pub(super) pool_teardown_ms: f64,
    pub(super) server_teardown_ms: f64,
}
