//! Private method contracts shared by the probe and its native callers.
//! These bounds cover control values; seeded heap bytes never cross the wire.

use ic_wire::Limits;
use serde::{Deserialize, Serialize};

/// Maximum frame size admitted before the CDK copies an argument.
pub const FRAME_BYTES: usize = 64;
/// Shared byte, allocation, node and depth budgets for private control calls.
pub const LIMITS: Limits = Limits {
    bytes: FRAME_BYTES,
    allocation: 4096,
    nodes: 128,
    depth: 16,
};

/// Snapshot-contained heap state observed by the fixture-reuse workload.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct State {
    /// Number of mutations since the state was seeded.
    pub mutations: u64,
    /// Size of the seeded heap allocation.
    pub bytes: u64,
    /// First seeded byte, absent for an empty allocation.
    pub first: Option<u8>,
}

/// Seed the fixture's deterministic heap allocation.
pub struct SeedState;
impl SeedState {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0001;
    /// Exported canister method name.
    pub const NAME: &'static str = "fixture_seed_state";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Mutate the fixture's counter and first byte.
pub struct MutateState;
impl MutateState {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0002;
    /// Exported canister method name.
    pub const NAME: &'static str = "fixture_mutate_state";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Inspect the fixture's heap without mutating it.
pub struct InspectState;
impl InspectState {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0003;
    /// Exported canister method name.
    pub const NAME: &'static str = "fixture_state";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Confirm the same canister remains callable after an intentional trap.
pub struct Ping;
impl Ping {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0004;
    /// Exported canister method name.
    pub const NAME: &'static str = "ping";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Execute the measured arithmetic workload.
pub struct BenchmarkOnce;
impl BenchmarkOnce {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0005;
    /// Exported canister method name.
    pub const NAME: &'static str = "benchmark_once";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Emit a start marker and intentionally trap before its matching end.
pub struct BenchmarkTrap;
impl BenchmarkTrap {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0006;
    /// Exported canister method name.
    pub const NAME: &'static str = "benchmark_start_then_trap";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Observe the actual caller of a query through the typed native adapter.
pub struct QueryCaller;
impl QueryCaller {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0007;
    /// Exported canister method name.
    pub const NAME: &'static str = "fixture_query_caller";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Observe the actual caller of an update through the typed native adapter.
pub struct UpdateCaller;
impl UpdateCaller {
    /// Stable Wire method discriminator for this private endpoint.
    pub const ID: u64 = 0x544b_5052_4f42_0008;
    /// Exported canister method name.
    pub const NAME: &'static str = "fixture_update_caller";
    /// Request frame and decoding budgets.
    pub const REQUEST_LIMITS: Limits = LIMITS;
    /// Response frame and decoding budgets.
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_limits_admit_the_largest_control_values() {
        let request =
            ic_wire::native::encode_request(SeedState::ID, &u32::MAX, SeedState::REQUEST_LIMITS)
                .unwrap();
        assert_eq!(
            ic_wire::native::decode_request::<u32>(
                SeedState::ID,
                &request,
                SeedState::REQUEST_LIMITS
            ),
            Ok(u32::MAX)
        );
        for first in [None, Some(u8::MAX)] {
            let state = State {
                mutations: u64::MAX,
                bytes: u64::MAX,
                first,
            };
            let reply = ic_wire::native::encode_response(
                InspectState::ID,
                &state,
                InspectState::RESPONSE_LIMITS,
            )
            .unwrap();
            assert_eq!(
                ic_wire::native::decode_response::<State>(
                    InspectState::ID,
                    &reply,
                    InspectState::RESPONSE_LIMITS
                ),
                Ok(state)
            );
        }
        let reply = ic_wire::native::encode_response(
            BenchmarkOnce::ID,
            &u64::MAX,
            BenchmarkOnce::RESPONSE_LIMITS,
        )
        .unwrap();
        assert_eq!(
            ic_wire::native::decode_response::<u64>(
                BenchmarkOnce::ID,
                &reply,
                BenchmarkOnce::RESPONSE_LIMITS
            ),
            Ok(u64::MAX)
        );
    }
}
