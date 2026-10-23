use crate::protocol::{
    self, BenchmarkOnce, BenchmarkTrap, InspectState, MutateState, Ping, QueryCaller, SeedState,
    State, UpdateCaller,
};
use ic_platform::cdk::{query, update};
use ic_testkit::performance;
use ic_wire::{Bytes, Text, native};
use serde::{Serialize, de::DeserializeOwned};
use std::cell::RefCell;

thread_local! {
    static FIXTURE_STATE: RefCell<(u64, Vec<u8>)> = const { RefCell::new((0, Vec::new())) };
}

// Deterministic snapshot-contained state for the opt-in fixture benchmark.
#[update(
    guard = "guard",
    decode_with = "decode::<u32, {SeedState::ID}>",
    encode_with = "encode_unit::<{SeedState::ID}>"
)]
fn fixture_seed_state(bytes: u32) {
    FIXTURE_STATE.with_borrow_mut(|state| *state = (0, vec![42; bytes as usize]));
}

#[update(
    guard = "guard",
    decode_with = "decode::<(), {MutateState::ID}>",
    encode_with = "encode_unit::<{MutateState::ID}>"
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
    guard = "guard",
    decode_with = "decode::<(), {InspectState::ID}>",
    encode_with = "encode::<State, {InspectState::ID}>"
)]
fn fixture_state() -> State {
    FIXTURE_STATE.with_borrow(|state| State {
        mutations: state.0,
        bytes: state.1.len() as u64,
        first: state.1.first().copied(),
    })
}

#[query(
    guard = "guard",
    decode_with = "decode::<(), {Ping::ID}>",
    encode_with = "encode::<Text<4>, {Ping::ID}>"
)]
fn ping() -> Text<4> {
    Text::new("pong".into()).expect("fixed probe reply fits")
}

#[update(
    guard = "guard",
    decode_with = "decode::<(), {BenchmarkOnce::ID}>",
    encode_with = "encode::<u64, {BenchmarkOnce::ID}>"
)]
fn benchmark_once() -> u64 {
    performance::measure("probe/benchmark_once:start");

    let mut acc = 0_u64;
    for n in 0..1_000 {
        acc = acc.wrapping_add(n * 3);
    }

    performance::measure("probe/benchmark_once:end");
    acc
}

#[update(
    guard = "guard",
    decode_with = "decode::<(), {BenchmarkTrap::ID}>",
    encode_with = "encode_unit::<{BenchmarkTrap::ID}>"
)]
fn benchmark_start_then_trap() {
    performance::measure("probe/benchmark_start_then_trap:start");
    ic_platform::ic0::trap(b"intentional perf probe trap");
}

#[query(
    guard = "guard",
    decode_with = "decode::<(), {QueryCaller::ID}>",
    encode_with = "encode::<Bytes<29>, {QueryCaller::ID}>"
)]
fn fixture_query_caller() -> Bytes<29> {
    Bytes::new(ic_platform::cdk::api::msg_caller().as_slice().to_vec())
        .expect("principal byte bound")
}

#[update(
    guard = "guard",
    decode_with = "decode::<(), {UpdateCaller::ID}>",
    encode_with = "encode::<Bytes<29>, {UpdateCaller::ID}>"
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

fn decode<Request: Serialize + DeserializeOwned, const ID: u64>(bytes: Vec<u8>) -> Request {
    native::decode_request(ID, &bytes, protocol::LIMITS).expect("bounded perf probe Wire request")
}

fn encode<Response: Serialize, const ID: u64>(value: Response) -> Vec<u8> {
    native::encode_response(ID, &value, protocol::LIMITS).expect("bounded perf probe Wire reply")
}

fn encode_unit<const ID: u64>() -> Vec<u8> {
    encode::<(), ID>(())
}
