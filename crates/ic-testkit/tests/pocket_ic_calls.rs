mod support;

use ic_platform::Principal;
use ic_testkit::pic::{
    CandidCallError, ErrorCode, RejectCode, query_candid_args, update_candid_args,
};

#[cfg(feature = "reply-fixtures")]
#[test]
fn candid_calls_preserve_platform_limits_and_error_identity() {
    use ic_platform::candid::{DecodeError, DecodeLimits, contract::encode_args};
    use ic_testkit::pic::{
        CanisterInstallExt as _, InstallSpec,
        fixtures::{RawReplyLimits, RawReplyMethod, ReplyMode, raw_reply_canister},
    };

    let reply = encode_args((vec![42_u64], vec![7_u64; 4])).unwrap();
    let wasm = raw_reply_canister(
        &[
            RawReplyMethod {
                name: "query",
                mode: ReplyMode::Query,
                reply: &reply,
            },
            RawReplyMethod {
                name: "update",
                mode: ReplyMode::Update,
                reply: &reply,
            },
        ],
        RawReplyLimits {
            methods: 2,
            reply_bytes: reply.len() * 2,
            method_name_bytes: 11,
        },
    )
    .unwrap();
    let owned = support::pocket_ic();
    let pic = &owned.instance;
    let id = pic
        .try_create_and_install(InstallSpec {
            wasm,
            init_bytes: vec![],
            cycles: 1_000_000_000_000,
            install_sender: None,
            label: None,
        })
        .unwrap();
    let caller = Principal::anonymous();
    for (mode, method, operation) in [
        (ReplyMode::Query, "query", "query_call"),
        (ReplyMode::Update, "update", "update_call"),
    ] {
        let call = |limits: DecodeLimits| match mode {
            ReplyMode::Query => {
                query_candid_args::<(Vec<u64>,), _>(pic, id, caller, method, (), limits)
            }
            ReplyMode::Update => {
                update_candid_args::<(Vec<u64>,), _>(pic, id, caller, method, (), limits)
            }
        };
        // Decode one argument and skip the second only within the supplied quota.
        assert_eq!(call(support::REPLY_LIMITS).unwrap(), (vec![42],));
        for limits in [
            DecodeLimits {
                reply_bytes: reply.len() - 1,
                ..support::REPLY_LIMITS
            },
            DecodeLimits {
                decoding_quota: 0,
                ..support::REPLY_LIMITS
            },
            DecodeLimits {
                skipping_quota: 0,
                ..support::REPLY_LIMITS
            },
            DecodeLimits {
                type_entries: 0,
                ..support::REPLY_LIMITS
            },
            DecodeLimits {
                header_bytes: 0,
                ..support::REPLY_LIMITS
            },
        ] {
            let error = call(limits).unwrap_err();
            let cause = std::error::Error::source(&error)
                .unwrap()
                .downcast_ref::<DecodeError>()
                .expect("the public error retains Platform's decoder identity");
            if limits.reply_bytes < reply.len() {
                assert!(
                    matches!(cause, DecodeError::ReplyTooLarge { actual, maximum }
                    if *actual == reply.len() && *maximum == limits.reply_bytes)
                );
            } else {
                assert!(matches!(cause, DecodeError::Candid(_)));
                assert!(std::error::Error::source(cause).is_some());
            }
            let CandidCallError::Decode { context, bytes, .. } = error else {
                panic!("expected a typed decode refusal");
            };
            assert_eq!(bytes, reply.len());
            assert_eq!(context.operation, operation);
            assert_eq!(context.canister_id, id);
            assert_eq!(context.caller, caller);
            assert_eq!(context.method, method);
        }
    }
}

#[test]
fn candid_call_errors_preserve_live_pocket_ic_rejections() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let removed_canister = pocket_ic.create_canister();
    pocket_ic
        .stop_canister(removed_canister, None)
        .expect("stop canister before deletion");
    pocket_ic
        .delete_canister(removed_canister, None)
        .expect("delete canister used for rejection test");

    let sender = Principal::management_canister();
    for (result, operation, caller) in [
        (
            query_candid_args::<(), _>(
                pocket_ic,
                removed_canister,
                Principal::anonymous(),
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "query_call",
            Principal::anonymous(),
        ),
        (
            update_candid_args::<(), _>(
                pocket_ic,
                removed_canister,
                Principal::anonymous(),
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "update_call",
            Principal::anonymous(),
        ),
        (
            query_candid_args::<(), _>(
                pocket_ic,
                removed_canister,
                sender,
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "query_call",
            sender,
        ),
        (
            update_candid_args::<(), _>(
                pocket_ic,
                removed_canister,
                sender,
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "update_call",
            sender,
        ),
    ] {
        let error = result.expect_err("a deleted canister must reject the call");
        let CandidCallError::CanisterReject {
            context,
            response: rejection,
        } = error
        else {
            panic!("expected canister rejection: {error}");
        };
        assert_eq!(rejection.reject_code, RejectCode::DestinationInvalid);
        assert_eq!(rejection.error_code, ErrorCode::CanisterNotFound);
        assert_eq!(context.operation, operation);
        assert_eq!(context.canister_id, removed_canister);
        assert_eq!(context.caller, caller);
        assert_eq!(context.method, "missing");
    }
}
