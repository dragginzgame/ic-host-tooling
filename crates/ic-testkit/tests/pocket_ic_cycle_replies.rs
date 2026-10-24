//! Integration checks for pocket ic cycle replies behavior.
#![cfg(feature = "held-reply-fixtures")]

mod support;

use ic_host_artifacts::{
    wasm::validate,
    wasm_text::{CompileLimits, compile},
};
use ic_platform::{
    pocket_ic::{PocketIc, PocketIcBuilder},
    principal::ExternalPrincipal as Principal,
};
use ic_testkit::pic::fixtures::{
    RawReplyLimits, RawReplyMethod, ReplyMode, cycle_reply_canister, held::HeldReplyFixture,
};
use std::{fmt::Write, time::Duration};

const ATTACHED: u128 = 10_000;
const BARRIER_CYCLES: u128 = 50_000_000_000;
const LIMITS: RawReplyLimits = RawReplyLimits {
    methods: 3,
    reply_bytes: 100,
    method_name_bytes: 9,
};
const REPLIES: [&[u8]; 3] = [b"acknowledged", b"provider error", b"\xff\0"];

#[test]
fn installed_zero_partial_full_acceptance_preserves_bytes_refunds_and_balances() {
    // System subnet fees are zero: exact balance deltas isolate actual transfers,
    // while callbacks still execute real inter-canister calls and IC refund APIs.
    let owned = support::startup_config()
        .build_instance(PocketIcBuilder::new().with_system_subnet())
        .unwrap();
    let pic = owned.instance();
    for cap in [0, 3_000, ATTACHED, u128::MAX] {
        let provider = funded(pic);
        let initial = pic.cycle_balance(provider);
        let methods = methods();
        let wasm = cycle_reply_canister(&methods, LIMITS, cap, 10_000).unwrap();
        validate(&wasm, 10_000).unwrap();
        assert_eq!(
            pic.cycle_balance(provider),
            initial,
            "construction accepts nothing"
        );
        pic.install_canister(provider, wasm, vec![], None);
        assert_eq!(
            pic.cycle_balance(provider),
            initial,
            "installation accepts nothing"
        );
        let consumer = install_consumer(pic, provider);
        let accepted = cap.min(ATTACHED);
        for reply in REPLIES {
            let source = pic.cycle_balance(consumer);
            let destination = pic.cycle_balance(provider);
            let result = pic
                .update_call(
                    consumer,
                    Principal::anonymous(),
                    "send",
                    ATTACHED.to_le_bytes().to_vec(),
                )
                .unwrap();
            assert_result(&result, ATTACHED - accepted, false, reply);
            assert_eq!(pic.cycle_balance(consumer), source - accepted);
            assert_eq!(pic.cycle_balance(provider), destination + accepted);
        }
        let source = pic.cycle_balance(consumer);
        let destination = pic.cycle_balance(provider);
        let result = pic
            .update_call(
                consumer,
                Principal::anonymous(),
                "send",
                ATTACHED.to_le_bytes().to_vec(),
            )
            .unwrap();
        assert_result(&result, ATTACHED, true, b"held reply slots exhausted");
        assert_eq!(
            pic.cycle_balance(consumer),
            source,
            "exhaustion refunds everything"
        );
        assert_eq!(pic.cycle_balance(provider), destination);
    }
}

#[test]
fn barrier_setup_trap_rolls_back_acceptance_and_slot_consumption() {
    let owned = support::startup_config()
        .build_instance(PocketIcBuilder::new().with_system_subnet())
        .unwrap();
    let pic = owned.instance();
    let provider = funded(pic);
    let destination = pic.cycle_balance(provider);
    // More than the provider can send even after accepting the attached cycles.
    let barrier_cycles = destination + ATTACHED + BARRIER_CYCLES;
    let mut fixture = HeldReplyFixture::new_with_cycle_acceptance(
        pic,
        provider,
        &methods(),
        LIMITS,
        barrier_cycles,
        3_000,
        10_000,
    )
    .unwrap();
    pic.install_canister(provider, fixture.wasm().to_vec(), vec![], None);
    let consumer = install_consumer(pic, provider);
    let source = pic.cycle_balance(consumer);
    let result = pic
        .update_call(
            consumer,
            Principal::anonymous(),
            "send",
            ATTACHED.to_le_bytes().to_vec(),
        )
        .unwrap();
    assert_eq!(
        u128::from_le_bytes(result[..16].try_into().unwrap()),
        ATTACHED
    );
    assert_ne!(u32::from_le_bytes(result[16..20].try_into().unwrap()), 0);
    assert_eq!(pic.cycle_balance(consumer), source);
    assert_eq!(pic.cycle_balance(provider), destination);
    assert_eq!(fixture.pending(pic).unwrap(), []);

    // Repair only the funding, retaining the installed module and its state.
    pic.add_cycles(provider, barrier_cycles);
    let call = pic
        .submit_call(
            consumer,
            Principal::anonymous(),
            "send",
            ATTACHED.to_le_bytes().to_vec(),
        )
        .unwrap();
    let mut pending = vec![];
    for _ in 0..20 {
        pic.tick();
        pending = fixture.pending(pic).unwrap();
        if !pending.is_empty() {
            break;
        }
    }
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].slot(), 0, "the trapped update consumed no slot");
    assert_eq!(pic.cycle_balance(provider), destination + 3_000);
    fixture.release(pic, &pending[0]).unwrap();
    let result = pic.await_call(call).unwrap();
    assert_result(&result, ATTACHED - 3_000, false, REPLIES[0]);
    assert_eq!(pic.cycle_balance(consumer), source - 3_000);
}

#[test]
fn held_acceptance_commits_before_release_and_survives_timeout() {
    let owned = support::startup_config()
        .build_instance(PocketIcBuilder::new().with_system_subnet())
        .unwrap();
    let pic = owned.instance();
    let provider = funded(pic);
    let destination = pic.cycle_balance(provider);
    let methods = methods();
    let mut fixture = HeldReplyFixture::new_with_cycle_acceptance(
        pic,
        provider,
        &methods,
        LIMITS,
        BARRIER_CYCLES,
        3_000,
        10_000,
    )
    .unwrap();
    assert_eq!(pic.cycle_balance(provider), destination);
    validate(fixture.wasm(), 10_000).unwrap();
    pic.install_canister(provider, fixture.wasm().to_vec(), vec![], None);
    let consumer = install_consumer(pic, provider);
    let source = pic.cycle_balance(consumer);
    let mut calls = vec![];
    for slot in 0..2 {
        calls.push(
            pic.submit_call(
                consumer,
                Principal::anonymous(),
                "send",
                ATTACHED.to_le_bytes().to_vec(),
            )
            .unwrap(),
        );
        let mut observed = false;
        for _ in 0..20 {
            pic.tick();
            if fixture
                .pending(pic)
                .unwrap()
                .iter()
                .any(|request| request.slot() == slot)
            {
                observed = true;
                break;
            }
        }
        assert!(observed, "real barrier for slot {slot}");
        assert_eq!(
            pic.cycle_balance(provider),
            destination + (slot as u128 + 1) * 3_000 - (slot as u128 + 1) * BARRIER_CYCLES
        );
    }
    assert_eq!(pic.cycle_balance(consumer), source - 2 * ATTACHED);
    let pending = fixture.pending(pic).unwrap();
    for slot in [1, 0] {
        let request = pending
            .iter()
            .find(|request| request.slot() == slot)
            .unwrap();
        fixture.release(pic, request).unwrap();
        let result = pic.await_call(calls[slot].clone()).unwrap();
        assert_result(&result, ATTACHED - 3_000, false, REPLIES[slot]);
    }
    assert_eq!(pic.cycle_balance(consumer), source - 2 * 3_000);

    let call = pic
        .submit_call(
            consumer,
            Principal::anonymous(),
            "send",
            ATTACHED.to_le_bytes().to_vec(),
        )
        .unwrap();
    let mut pending = vec![];
    for _ in 0..20 {
        pic.tick();
        pending = fixture.pending(pic).unwrap();
        if !pending.is_empty() {
            break;
        }
    }
    assert_eq!(pending.len(), 1);
    pic.advance_time(Duration::from_secs(61));
    let result = pic.await_call(call).unwrap();
    assert_result(
        &result,
        ATTACHED - 3_000,
        true,
        b"held reply barrier failed",
    );
    assert_eq!(
        pic.cycle_balance(consumer),
        source - 3 * 3_000,
        "timeout cannot undo previously committed acceptance"
    );
    assert!(fixture.release(pic, &pending[0]).is_err());
}

fn methods() -> Vec<RawReplyMethod<'static>> {
    REPLIES
        .iter()
        .map(|reply| RawReplyMethod {
            name: "pay",
            mode: ReplyMode::Update,
            reply,
        })
        .collect()
}

fn funded(pic: &PocketIc) -> Principal {
    let id = pic.create_canister();
    pic.add_cycles(id, 10_000_000_000_000);
    id
}

fn assert_result(result: &[u8], refunded: u128, rejected: bool, reply: &[u8]) {
    assert_eq!(
        u128::from_le_bytes(result[..16].try_into().unwrap()),
        refunded
    );
    assert_eq!(
        u32::from_le_bytes(result[16..20].try_into().unwrap()) != 0,
        rejected
    );
    assert_eq!(&result[20..], reply);
}

fn install_consumer(pic: &PocketIc, provider: Principal) -> Principal {
    let consumer = funded(pic);
    let mut principal = String::new();
    for byte in provider.as_slice() {
        write!(principal, "\\{byte:02x}").unwrap();
    }
    // The test transport returns IC-observed refunded cycles, rejection code and
    // exact provider bytes. No journal observation or application DTO is injected.
    let source = format!(
        r#"(module
(import "ic0" "call_new" (func $new (param i32 i32 i32 i32 i32 i32 i32 i32)))
(import "ic0" "call_cycles_add128" (func $cycles (param i64 i64)))
(import "ic0" "call_perform" (func $perform (result i32)))
(import "ic0" "msg_arg_data_size" (func $size (result i32)))
(import "ic0" "msg_arg_data_copy" (func $copy (param i32 i32 i32)))
(import "ic0" "msg_cycles_refunded128" (func $refunded (param i32)))
(import "ic0" "msg_reject_code" (func $code (result i32)))
(import "ic0" "msg_reject_msg_size" (func $reject_size (result i32)))
(import "ic0" "msg_reject_msg_copy" (func $reject_copy (param i32 i32 i32)))
(import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
(import "ic0" "msg_reply" (func $reply))
(memory (export "memory") 1 1)
(data (i32.const 0) "{principal}") (data (i32.const 32) "pay")
(table 2 funcref) (elem (i32.const 0) $callback $failed)
(func $header
 (call $refunded (i32.const 80)) (i32.store (i32.const 96) (call $code))
 (call $append (i32.const 80) (i32.const 20)))
(func $callback (param i32)
 (call $header) (call $copy (i32.const 128) (i32.const 0) (call $size))
 (call $append (i32.const 128) (call $size)) (call $reply))
(func $failed (param i32)
 (call $header) (call $reject_copy (i32.const 128) (i32.const 0) (call $reject_size))
 (call $append (i32.const 128) (call $reject_size)) (call $reply))
(func (export "canister_update send")
 (call $copy (i32.const 64) (i32.const 0) (i32.const 16))
 (call $new (i32.const 0) (i32.const {length}) (i32.const 32) (i32.const 3)
  (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 0))
 (call $cycles (i64.load offset=8 (i32.const 64)) (i64.load (i32.const 64)))
 (if (call $perform) (then unreachable))))"#,
        length = provider.as_slice().len()
    );
    let wasm = compile(
        &source,
        CompileLimits {
            source_bytes: source.len(),
            module_bytes: 10_000,
        },
    )
    .unwrap();
    validate(&wasm, 10_000).unwrap();
    pic.install_canister(consumer, wasm, vec![], None);
    consumer
}

#[test]
fn application_subnet_uses_the_same_real_refund_path() {
    // Application-subnet execution fees are separate from callback refunds.
    // Exact gross balance conservation is qualified on the system subnet above.
    let owned = support::pocket_ic();
    let pic = owned.instance();
    let provider = funded(pic);
    let wasm = cycle_reply_canister(&methods(), LIMITS, 3_000, 10_000).unwrap();
    pic.install_canister(provider, wasm, vec![], None);
    let consumer = install_consumer(pic, provider);
    for reply in REPLIES {
        let result = pic
            .update_call(
                consumer,
                Principal::anonymous(),
                "send",
                ATTACHED.to_le_bytes().to_vec(),
            )
            .unwrap();
        assert_result(&result, ATTACHED - 3_000, false, reply);
    }
}
