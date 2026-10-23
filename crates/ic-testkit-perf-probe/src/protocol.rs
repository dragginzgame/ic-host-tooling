//! Private method contracts shared by the probe and its native callers.
//! These bounds cover control values; seeded heap bytes never cross the wire.

use ic_wire::Limits;
use serde::{Deserialize, Serialize};

/// Maximum frame size admitted before the CDK copies an argument.
pub const FRAME_BYTES: usize = 64;
pub const LIMITS: Limits = Limits {
    bytes: FRAME_BYTES,
    allocation: 4096,
    nodes: 128,
    depth: 16,
};

/// Snapshot-contained heap state observed by the fixture-reuse workload.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct State {
    pub mutations: u64,
    pub bytes: u64,
    pub first: Option<u8>,
}

/// Seed the fixture's deterministic heap allocation.
pub struct SeedState;
impl SeedState {
    pub const ID: u64 = 0x544b_5052_4f42_0001;
    pub const NAME: &'static str = "fixture_seed_state";
    pub const REQUEST_LIMITS: Limits = LIMITS;
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Mutate the fixture's counter and first byte.
pub struct MutateState;
impl MutateState {
    pub const ID: u64 = 0x544b_5052_4f42_0002;
    pub const NAME: &'static str = "fixture_mutate_state";
    pub const REQUEST_LIMITS: Limits = LIMITS;
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Inspect the fixture's heap without mutating it.
pub struct InspectState;
impl InspectState {
    pub const ID: u64 = 0x544b_5052_4f42_0003;
    pub const NAME: &'static str = "fixture_state";
    pub const REQUEST_LIMITS: Limits = LIMITS;
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Confirm the same canister remains callable after an intentional trap.
pub struct Ping;
impl Ping {
    pub const ID: u64 = 0x544b_5052_4f42_0004;
    pub const NAME: &'static str = "ping";
    pub const REQUEST_LIMITS: Limits = LIMITS;
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Execute the measured arithmetic workload.
pub struct BenchmarkOnce;
impl BenchmarkOnce {
    pub const ID: u64 = 0x544b_5052_4f42_0005;
    pub const NAME: &'static str = "benchmark_once";
    pub const REQUEST_LIMITS: Limits = LIMITS;
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Emit a start marker and intentionally trap before its matching end.
pub struct BenchmarkTrap;
impl BenchmarkTrap {
    pub const ID: u64 = 0x544b_5052_4f42_0006;
    pub const NAME: &'static str = "benchmark_start_then_trap";
    pub const REQUEST_LIMITS: Limits = LIMITS;
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Observe the actual caller of a query through the typed native adapter.
pub struct QueryCaller;
impl QueryCaller {
    pub const ID: u64 = 0x544b_5052_4f42_0007;
    pub const NAME: &'static str = "fixture_query_caller";
    pub const REQUEST_LIMITS: Limits = LIMITS;
    pub const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Observe the actual caller of an update through the typed native adapter.
pub struct UpdateCaller;
impl UpdateCaller {
    pub const ID: u64 = 0x544b_5052_4f42_0008;
    pub const NAME: &'static str = "fixture_update_caller";
    pub const REQUEST_LIMITS: Limits = LIMITS;
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
