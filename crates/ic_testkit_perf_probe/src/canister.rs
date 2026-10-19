use crate::protocol::{
    self, BenchmarkOnce, BenchmarkTrap, InspectState, MutateState, Ping, QueryCaller, SeedState,
    State, UpdateCaller,
};
use ic_platform::cdk::{query, update};
use ic_testkit::performance::Performance;
use ic_wire::{Bytes, Method, Text};
use std::cell::RefCell;

thread_local! {
    static FIXTURE_STATE: RefCell<(u64, Vec<u8>)> = const { RefCell::new((0, Vec::new())) };
}

// Deterministic snapshot-contained state for the opt-in fixture benchmark.
#[update(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<SeedState>",
    encode_with = "encode_unit::<SeedState>"
)]
fn fixture_seed_state(bytes: u32) {
    FIXTURE_STATE.with_borrow_mut(|state| *state = (0, vec![42; bytes as usize]));
}

#[update(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<MutateState>",
    encode_with = "encode_unit::<MutateState>"
)]
fn fixture_mutate_state() {
    FIXTURE_STATE.with_borrow_mut(|state| {
        state.0 += 1;
        if let Some(first) = state.1.first_mut() {
            *first = 255;
        }
    });
}

#[query(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<InspectState>",
    encode_with = "encode::<InspectState>"
)]
fn fixture_state() -> State {
    FIXTURE_STATE.with_borrow(|state| State {
        mutations: state.0,
        bytes: state.1.len() as u64,
        first: state.1.first().copied(),
    })
}

#[query(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<Ping>",
    encode_with = "encode::<Ping>"
)]
fn ping() -> Text<4> {
    Text::new("pong".into()).expect("fixed probe reply fits")
}

#[update(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<BenchmarkOnce>",
    encode_with = "encode::<BenchmarkOnce>"
)]
fn benchmark_once() -> u64 {
    Performance::measure("probe/benchmark_once:start");

    let mut acc = 0_u64;
    for n in 0..1_000 {
        acc = acc.wrapping_add(n * 3);
    }

    Performance::measure("probe/benchmark_once:end");
    acc
}

#[update(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<BenchmarkTrap>",
    encode_with = "encode_unit::<BenchmarkTrap>"
)]
fn benchmark_start_then_trap() {
    Performance::measure("probe/benchmark_start_then_trap:start");
    ic_platform::ic0::trap(b"intentional perf probe trap");
}

#[query(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<QueryCaller>",
    encode_with = "encode::<QueryCaller>"
)]
fn fixture_query_caller() -> Bytes<29> {
    Bytes::new(ic_platform::cdk::api::msg_caller().as_slice().to_vec())
        .expect("principal byte bound")
}

#[update(
    hidden = true,
    guard = "guard",
    decode_with = "decode::<UpdateCaller>",
    encode_with = "encode::<UpdateCaller>"
)]
fn fixture_update_caller() -> Bytes<29> {
    Bytes::new(ic_platform::cdk::api::msg_caller().as_slice().to_vec())
        .expect("principal byte bound")
}

fn guard() -> Result<(), String> {
    if ic_platform::ic0::msg_arg_data_size() > protocol::FRAME_BYTES {
        Err("perf probe Wire request exceeds limit".into())
    } else {
        Ok(())
    }
}

fn decode<M: Method>(bytes: Vec<u8>) -> M::Request {
    ic_wire::decode_request::<M>(&bytes).expect("bounded perf probe Wire request")
}

fn encode<M: Method>(value: M::Response) -> Vec<u8> {
    ic_wire::encode_response::<M>(&value)
        .expect("bounded perf probe Wire reply")
        .into_bytes()
}

fn encode_unit<M: Method<Response = ()>>() -> Vec<u8> {
    encode::<M>(())
}
