#![cfg(feature = "held-reply-fixtures")]

mod support;

use ic_host_artifacts::{
    wasm::validate,
    wasm_text::{CompileLimits, compile},
};
use ic_platform::{
    pocket_ic::{PocketIc, common::rest::RawMessageId},
    principal::ExternalPrincipal as Principal,
};
use ic_testkit::pic::fixtures::{
    RawReplyLimits, RawReplyMethod, ReplyMode,
    held::{HeldReplyError, HeldReplyFixture, HeldRequest},
};

use std::{fmt::Write, time::Duration};

#[test]
fn holds_real_intercanister_calls_until_exact_out_of_order_release() {
    let owned = support::pocket_ic();
    let pic = owned.instance();
    let caller = Principal::anonymous();
    let provider = pic.create_canister();
    pic.add_cycles(provider, 10_000_000_000_000);
    let replies: [&[u8]; 3] = [
        b"original first\0",
        b"original second\xff",
        b"original third",
    ];
    let methods: Vec<_> = replies
        .iter()
        .map(|reply| RawReplyMethod {
            name: "read",
            mode: ReplyMode::Update,
            reply,
        })
        .collect();
    let mut fixture = HeldReplyFixture::new(
        pic,
        provider,
        &methods,
        RawReplyLimits {
            methods: 3,
            reply_bytes: 100,
            method_name_bytes: 12,
        },
        50_000_000_000,
        100_000,
    )
    .unwrap();
    validate(fixture.wasm(), 100_000).unwrap();
    pic.install_canister(provider, fixture.wasm().to_vec(), vec![], None);

    let consumer = install_consumer(pic, provider);

    let mut calls: Vec<RawMessageId> = vec![];
    for slot in 0..3 {
        calls.push(pic.submit_call(consumer, caller, "start", vec![]).unwrap());
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
        assert!(observed, "slot {slot} must reach the real HTTP barrier");
    }
    let pending = fixture.pending(pic).unwrap();
    assert_eq!(pending.len(), 3);
    pic.update_call(consumer, caller, "cancel", vec![]).unwrap();
    assert_eq!(
        pic.query_call(consumer, caller, "committed", vec![])
            .unwrap(),
        1_u32.to_le_bytes()
    );
    for _ in 0..5 {
        pic.tick();
    }
    for call in &calls {
        assert!(pic.ingress_status(call.clone()).is_none());
    }

    for slot in [1, 0, 2] {
        let request = pending
            .iter()
            .find(|request| request.slot() == slot)
            .unwrap();
        fixture.release(pic, request).unwrap();
        assert!(matches!(
            fixture.release(pic, request),
            Err(HeldReplyError::UnknownRequest)
        ));
        assert_eq!(pic.await_call(calls[slot].clone()).unwrap(), replies[slot]);
        for held in fixture.pending(pic).unwrap() {
            assert!(pic.ingress_status(calls[held.slot()].clone()).is_none());
        }
    }
    assert_eq!(fixture.pending(pic).unwrap(), vec![]);
    assert!(
        pic.update_call(provider, caller, "read", vec![]).is_err(),
        "slots are never reused"
    );

    // Same server, distinct instance must not accept the controller/token.
    let other = support::pocket_ic();
    assert!(matches!(
        fixture.pending(other.instance()),
        Err(HeldReplyError::WrongInstance)
    ));
    pic.auto_progress();
    assert!(matches!(
        fixture.pending(pic),
        Err(HeldReplyError::AutomaticProgress)
    ));
    pic.stop_progress();
}

fn install_consumer(pic: &PocketIc, provider: Principal) -> Principal {
    // A real consumer canister retains its ingress context across a provider call.
    // Its independent update commits while provider replies remain held.
    let consumer = pic.create_canister();
    pic.add_cycles(consumer, 10_000_000_000_000);
    let mut principal = String::new();
    for byte in provider.as_slice() {
        write!(principal, "\\{byte:02x}").unwrap();
    }
    let source = format!(
        r#"(module
      (import "ic0" "call_new" (func $new (param i32 i32 i32 i32 i32 i32 i32 i32)))
      (import "ic0" "call_perform" (func $perform (result i32)))
      (import "ic0" "msg_arg_data_size" (func $size (result i32)))
      (import "ic0" "msg_arg_data_copy" (func $copy (param i32 i32 i32)))
      (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
      (import "ic0" "msg_reply" (func $reply))
      (memory (export "memory") 1 1)
      (data (i32.const 0) "{principal}")
      (data (i32.const 32) "read")
      (global $committed (mut i32) (i32.const 0))
      (table 2 funcref) (elem (i32.const 0) $callback $failed)
      (func $callback (param i32)
        (call $copy (i32.const 100) (i32.const 0) (call $size))
        (call $append (i32.const 100) (call $size)) (call $reply))
      (func $failed (param i32) unreachable)
      (func (export "canister_update start")
        (call $new (i32.const 0) (i32.const {length}) (i32.const 32) (i32.const 4)
          (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 0))
        (if (call $perform) (then unreachable)))
      (func (export "canister_update cancel")
        (global.set $committed (i32.const 1)) (call $reply))
      (func (export "canister_query committed")
        (i32.store (i32.const 100) (global.get $committed))
        (call $append (i32.const 100) (i32.const 4)) (call $reply)))"#,
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
fn expired_and_foreign_requests_never_release_another_original() {
    let owned = support::pocket_ic();
    let pic = owned.instance();
    let caller = Principal::anonymous();
    let methods = [RawReplyMethod {
        name: "hold",
        mode: ReplyMode::Update,
        reply: b"must not leak",
    }];
    let mut fixtures = vec![];
    let mut calls = vec![];
    for _ in 0..2 {
        let id = pic.create_canister();
        pic.add_cycles(id, 10_000_000_000_000);
        let fixture = HeldReplyFixture::new(
            pic,
            id,
            &methods,
            RawReplyLimits {
                methods: 1,
                reply_bytes: 100,
                method_name_bytes: 4,
            },
            50_000_000_000,
            10_000,
        )
        .unwrap();
        pic.install_canister(id, fixture.wasm().to_vec(), vec![], None);
        calls.push(pic.submit_call(id, caller, "hold", vec![]).unwrap());
        fixtures.push(fixture);
    }
    let mut requests: Vec<Vec<HeldRequest>> = vec![];
    for fixture in &fixtures {
        let mut pending = vec![];
        for _ in 0..20 {
            pic.tick();
            pending = fixture.pending(pic).unwrap();
            if !pending.is_empty() {
                break;
            }
        }
        assert_eq!(pending.len(), 1);
        requests.push(pending);
    }
    assert!(matches!(
        fixtures[0].release(pic, &requests[1][0]),
        Err(HeldReplyError::UnknownRequest)
    ));
    for call in &calls {
        assert!(pic.ingress_status(call.clone()).is_none());
    }
    pic.advance_time(Duration::from_secs(61));
    for call in calls {
        let rejection = pic.await_call(call).unwrap_err();
        assert!(
            rejection
                .reject_message
                .contains("held reply barrier failed")
        );
    }
    for (fixture, request) in fixtures.iter_mut().zip(requests) {
        assert_eq!(fixture.pending(pic).unwrap(), vec![]);
        assert!(matches!(
            fixture.release(pic, &request[0]),
            Err(HeldReplyError::UnknownRequest)
        ));
    }
}
