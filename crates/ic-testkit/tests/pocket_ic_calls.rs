#![cfg(feature = "wire")]

mod support;

use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pic::{
    ErrorCode, PocketIcCallFailure, RejectCode, WireCallError, query_wire, update_wire,
};
use ic_wire::{Limits, Method};

struct Probe;
impl Method for Probe {
    type Request = ();
    type Response = u64;
    const ID: u64 = 91;
    const NAME: &'static str = "probe";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 64,
        allocation: 64,
        nodes: 8,
        depth: 4,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}

#[cfg(feature = "reply-fixtures")]
#[test]
fn wire_calls_preserve_codec_refusals_and_error_identity() {
    use ic_testkit::pic::{
        CanisterInstallExt as _, InstallSpec,
        fixtures::{RawReplyLimits, RawReplyMethod, ReplyMode, raw_reply_canister},
    };
    let valid = ic_wire::encode_response::<Probe>(&42).unwrap().into_bytes();
    let owned = support::pocket_ic();
    let pic = &owned.instance;
    for mode in [ReplyMode::Query, ReplyMode::Update] {
        for reply in [valid.as_slice(), &[]] {
            let wasm = raw_reply_canister(
                &[RawReplyMethod {
                    name: Probe::NAME,
                    mode,
                    reply,
                }],
                RawReplyLimits {
                    methods: 1,
                    reply_bytes: valid.len(),
                    method_name_bytes: 5,
                },
            )
            .unwrap();
            let id = pic
                .try_create_and_install(InstallSpec {
                    wasm,
                    init_bytes: vec![],
                    cycles: 1_000_000_000_000,
                    install_sender: None,
                    label: None,
                })
                .unwrap();
            let result = match mode {
                ReplyMode::Query => query_wire::<Probe>(pic, id, Principal::anonymous(), &()),
                ReplyMode::Update => update_wire::<Probe>(pic, id, Principal::anonymous(), &()),
            };
            if reply.is_empty() {
                let error = result.unwrap_err();
                assert!(
                    std::error::Error::source(&error)
                        .unwrap()
                        .is::<ic_wire::Error>()
                );
                assert!(!ic_testkit::pic::is_dead_pocket_ic_transport_error(&error));
                assert!(matches!(
                    error,
                    WireCallError::Decode {
                        source: ic_wire::Error::Malformed,
                        ..
                    }
                ));
            } else {
                assert_eq!(result.unwrap(), 42);
            }
        }
    }
}

#[test]
fn wire_call_errors_preserve_live_pocket_ic_rejections() {
    let owned = support::pocket_ic();
    let pic = &owned.instance;
    let removed = pic.create_canister();
    pic.stop_canister(removed, None).unwrap();
    pic.delete_canister(removed, None).unwrap();
    for caller in [Principal::anonymous(), Principal::management_canister()] {
        for (result, operation) in [
            (query_wire::<Probe>(pic, removed, caller, &()), "query_call"),
            (
                update_wire::<Probe>(pic, removed, caller, &()),
                "update_call",
            ),
        ] {
            let error = result.unwrap_err();
            assert!(!ic_testkit::pic::is_dead_pocket_ic_transport_error(&error));
            let WireCallError::Call { context, source } = error else {
                panic!("expected call failure");
            };
            let PocketIcCallFailure::Rejected(rejection) = *source else {
                panic!("expected rejection");
            };
            assert_eq!(rejection.reject_code, RejectCode::DestinationInvalid);
            assert_eq!(rejection.error_code, ErrorCode::CanisterNotFound);
            assert_eq!(context.operation, operation);
            assert_eq!(context.canister_id, removed);
            assert_eq!(context.caller, caller);
            assert_eq!(context.method, Probe::NAME);
        }
    }
}
