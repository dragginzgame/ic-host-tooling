#![cfg(feature = "wire")]

mod support;

use ic_platform::pocket_ic::{ErrorCode, RejectCode};
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pic::{PocketIcCallFailure, WireCallError, WireCallSpec, query_wire, update_wire};
use ic_wire::Limits;

const PROBE: WireCallSpec = WireCallSpec {
    method: "probe",
    method_id: 91,
    request_limits: Limits {
        bytes: 64,
        allocation: 512,
        nodes: 8,
        depth: 4,
    },
    response_limits: Limits {
        bytes: 64,
        allocation: 512,
        nodes: 8,
        depth: 4,
    },
};

#[cfg(feature = "reply-fixtures")]
#[test]
fn wire_calls_preserve_codec_refusals_and_error_identity() {
    use ic_testkit::pic::{
        CanisterInstallExt as _, InstallSpec,
        fixtures::{RawReplyLimits, RawReplyMethod, ReplyMode, raw_reply_canister},
    };
    let valid =
        ic_wire::native::encode_response(PROBE.method_id, &42_u64, PROBE.response_limits).unwrap();
    let owned = support::pocket_ic();
    let pic = owned.instance();
    for mode in [ReplyMode::Query, ReplyMode::Update] {
        for reply in [valid.as_slice(), &[]] {
            let wasm = raw_reply_canister(
                &[RawReplyMethod {
                    name: PROBE.method,
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
                ReplyMode::Query => {
                    query_wire::<_, u64>(pic, id, Principal::anonymous(), PROBE, &())
                }
                ReplyMode::Update => {
                    update_wire::<_, u64>(pic, id, Principal::anonymous(), PROBE, &())
                }
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
    let pic = owned.instance();
    let removed = pic.create_canister();
    pic.stop_canister(removed, None).unwrap();
    pic.delete_canister(removed, None).unwrap();
    for caller in [Principal::anonymous(), Principal::management_canister()] {
        for (result, operation) in [
            (
                query_wire::<_, u64>(pic, removed, caller, PROBE, &()),
                "query_call",
            ),
            (
                update_wire::<_, u64>(pic, removed, caller, PROBE, &()),
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
            assert_eq!(context.method, PROBE.method);
        }
    }
}
