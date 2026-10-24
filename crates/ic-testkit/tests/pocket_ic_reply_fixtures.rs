//! Integration checks for pocket ic reply fixtures behavior.
#![cfg(feature = "reply-fixtures")]

mod support;

use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pic::{
    CanisterInstallExt, InstallSpec,
    fixtures::{RawReplyLimits, RawReplyMethod, ReplyMode, raw_reply_canister},
};

#[test]
fn installed_fixture_returns_exact_query_and_update_bytes() {
    let query_reply = vec![0xff; 65_537];
    let update_reply = b"not candid\0";
    let wasm = raw_reply_canister(
        &[
            RawReplyMethod {
                name: "read",
                mode: ReplyMode::Query,
                reply: &query_reply,
            },
            RawReplyMethod {
                name: "write",
                mode: ReplyMode::Update,
                reply: update_reply,
            },
        ],
        RawReplyLimits {
            methods: 2,
            reply_bytes: query_reply.len() + update_reply.len(),
            method_name_bytes: 9,
        },
    )
    .expect("admit the explicit fixture catalog");
    let owned = support::pocket_ic();
    let pocket_ic = owned.instance();
    let canister = pocket_ic
        .try_create_and_install(InstallSpec {
            wasm,
            init_bytes: vec![],
            cycles: 1_000_000_000_000,
            install_sender: None,
            label: Some("raw-reply-fixture".into()),
        })
        .expect("install generated Wasm");
    let caller = Principal::anonymous();
    assert_eq!(
        pocket_ic
            .query_call(canister, caller, "read", vec![])
            .unwrap(),
        query_reply
    );
    assert_eq!(
        pocket_ic
            .update_call(canister, caller, "write", vec![])
            .unwrap(),
        update_reply
    );
}
