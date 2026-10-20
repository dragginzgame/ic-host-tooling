//! Private method contracts shared by the probe and its native callers.
//! These bounds cover control values; seeded heap bytes never cross the wire.

use ic_wire::{Bytes, Limits, Method, Text, Wire};

/// Maximum frame size admitted before the CDK copies an argument.
pub const FRAME_BYTES: usize = 64;
const LIMITS: Limits = Limits {
    bytes: FRAME_BYTES,
    allocation: 256,
    nodes: 16,
    depth: 4,
};

/// Snapshot-contained heap state observed by the fixture-reuse workload.
#[derive(Debug, Eq, PartialEq, Wire)]
pub struct State {
    #[wire(id = 1)]
    pub mutations: u64,
    #[wire(id = 2)]
    pub bytes: u64,
    #[wire(id = 3)]
    pub first: Option<u8>,
}

/// Seed the fixture's deterministic heap allocation.
pub struct SeedState;
impl Method for SeedState {
    type Request = u32;
    type Response = ();
    const ID: u64 = 0x544b_5052_4f42_0001;
    const NAME: &'static str = "fixture_seed_state";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Mutate the fixture's counter and first byte.
pub struct MutateState;
impl Method for MutateState {
    type Request = ();
    type Response = ();
    const ID: u64 = 0x544b_5052_4f42_0002;
    const NAME: &'static str = "fixture_mutate_state";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Inspect the fixture's heap without mutating it.
pub struct InspectState;
impl Method for InspectState {
    type Request = ();
    type Response = State;
    const ID: u64 = 0x544b_5052_4f42_0003;
    const NAME: &'static str = "fixture_state";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Confirm the same canister remains callable after an intentional trap.
pub struct Ping;
impl Method for Ping {
    type Request = ();
    type Response = Text<4>;
    const ID: u64 = 0x544b_5052_4f42_0004;
    const NAME: &'static str = "ping";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Execute the measured arithmetic workload.
pub struct BenchmarkOnce;
impl Method for BenchmarkOnce {
    type Request = ();
    type Response = u64;
    const ID: u64 = 0x544b_5052_4f42_0005;
    const NAME: &'static str = "benchmark_once";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Emit a start marker and intentionally trap before its matching end.
pub struct BenchmarkTrap;
impl Method for BenchmarkTrap {
    type Request = ();
    type Response = ();
    const ID: u64 = 0x544b_5052_4f42_0006;
    const NAME: &'static str = "benchmark_start_then_trap";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Observe the actual caller of a query through the typed native adapter.
pub struct QueryCaller;
impl Method for QueryCaller {
    type Request = ();
    type Response = Bytes<29>;
    const ID: u64 = 0x544b_5052_4f42_0007;
    const NAME: &'static str = "fixture_query_caller";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

/// Observe the actual caller of an update through the typed native adapter.
pub struct UpdateCaller;
impl Method for UpdateCaller {
    type Request = ();
    type Response = Bytes<29>;
    const ID: u64 = 0x544b_5052_4f42_0008;
    const NAME: &'static str = "fixture_update_caller";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_limits_admit_the_largest_control_values() {
        let request = ic_wire::encode_request::<SeedState>(&u32::MAX).unwrap();
        assert_eq!(
            ic_wire::decode_request::<SeedState>(request.as_bytes()),
            Ok(u32::MAX)
        );
        for first in [None, Some(u8::MAX)] {
            let state = State {
                mutations: u64::MAX,
                bytes: u64::MAX,
                first,
            };
            let reply = ic_wire::encode_response::<InspectState>(&state).unwrap();
            assert_eq!(
                ic_wire::decode_response::<InspectState>(reply.as_bytes()),
                Ok(state)
            );
        }
        let reply = ic_wire::encode_response::<BenchmarkOnce>(&u64::MAX).unwrap();
        assert_eq!(
            ic_wire::decode_response::<BenchmarkOnce>(reply.as_bytes()),
            Ok(u64::MAX)
        );
    }
}
