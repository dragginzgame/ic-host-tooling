use super::{WireCallError, WireCallSpec, call_wire, context};

use crate::pic::{PocketIcCallFailure, is_dead_pocket_ic_transport_error};
use ic_platform::pocket_ic::{ErrorCode, RejectCode, RejectResponse};
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_wire::{Limits, native};

const LIMITS: Limits = Limits {
    bytes: 64,
    allocation: 2048,
    nodes: 64,
    depth: 16,
};

#[test]
fn borrowed_requests_return_owned_responses() {
    use ic_wire::{Bytes, BytesRef};
    const BORROWED: WireCallSpec = WireCallSpec {
        method: "borrowed",
        method_id: 73,
        request_limits: LIMITS,
        response_limits: LIMITS,
    };

    let _ = super::query_wire::<BytesRef<'_, 8>, Bytes<8>>;
    let _ = super::update_wire::<BytesRef<'_, 8>, Bytes<8>>;
    let result = {
        let payload = vec![1, 2, 3];
        let request = BytesRef::new(&payload).unwrap();
        let mut calls = 0;
        let response = call_wire::<BytesRef<'_, 8>, Bytes<8>>(
            context(
                "update_call",
                BORROWED,
                Principal::anonymous(),
                Principal::anonymous(),
            ),
            BORROWED,
            &request,
            |frame| {
                calls += 1;
                assert_eq!(
                    native::decode_request::<BytesRef<'_, 8>>(BORROWED.method_id, &frame, LIMITS)
                        .unwrap()
                        .as_slice(),
                    payload
                );
                Ok(native::encode_response(
                    BORROWED.method_id,
                    &Bytes::<8>::new(vec![3, 2, 1]).unwrap(),
                    LIMITS,
                )
                .unwrap())
            },
        )
        .unwrap();
        assert_eq!(calls, 1);
        response
    };
    assert_eq!(result.as_slice(), &[3, 2, 1]);
}

const SAMPLE: WireCallSpec = WireCallSpec {
    method: "sample",
    method_id: 71,
    request_limits: LIMITS,
    response_limits: LIMITS,
};
const BAD_REQUEST: WireCallSpec = WireCallSpec {
    method: "bad_request_budget",
    method_id: 72,
    request_limits: Limits {
        bytes: 11,
        ..LIMITS
    },
    response_limits: LIMITS,
};
const BAD_REPLY: WireCallSpec = WireCallSpec {
    response_limits: Limits { nodes: 1, ..LIMITS },
    ..SAMPLE
};

#[test]
fn one_call_preserves_method_context_and_application_results() {
    for operation in ["query_call", "update_call"] {
        for value in [Ok(u64::MAX), Err(())] {
            let ctx = context(
                operation,
                SAMPLE,
                Principal::from_slice(&[1]),
                Principal::from_slice(&[2]),
            );
            assert_eq!(ctx.operation, operation);
            assert_eq!(ctx.canister_id, Principal::from_slice(&[1]));
            assert_eq!(ctx.caller, Principal::from_slice(&[2]));
            assert_eq!(ctx.method, SAMPLE.method);
            assert_eq!(ctx.method_id, SAMPLE.method_id);
            let mut calls = 0;
            let result = call_wire::<u32, Result<u64, ()>>(ctx, SAMPLE, &u32::MAX, |bytes| {
                calls += 1;
                assert_eq!(
                    native::decode_request::<u32>(SAMPLE.method_id, &bytes, LIMITS),
                    Ok(u32::MAX)
                );
                Ok(native::encode_response(SAMPLE.method_id, &value, LIMITS).unwrap())
            })
            .unwrap();
            assert_eq!(result, value);
            assert_eq!(calls, 1);
        }
    }
}

#[test]
fn request_admission_precedes_transport() {
    for request_limits in [
        BAD_REQUEST.request_limits,
        Limits { nodes: 3, ..LIMITS },
        Limits { depth: 2, ..LIMITS },
        Limits {
            allocation: LIMITS.bytes,
            ..LIMITS
        },
    ] {
        let spec = WireCallSpec {
            request_limits,
            ..BAD_REQUEST
        };
        let error = call_wire::<u32, ()>(
            context(
                "update_call",
                spec,
                Principal::anonymous(),
                Principal::anonymous(),
            ),
            spec,
            &1,
            |_| panic!("refused request must not reach transport"),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            WireCallError::Encode {
                source: ic_wire::Error::Limit,
                ..
            }
        ));
        assert!(
            std::error::Error::source(&error)
                .unwrap()
                .is::<ic_wire::Error>()
        );
    }
}

#[test]
fn invalid_responses_preserve_typed_refusal_and_context_without_payload_logs() {
    let valid = native::encode_response(SAMPLE.method_id, &Ok::<u64, ()>(7), LIMITS).unwrap();
    let mut wrong_method = valid.clone();
    wrong_method[2] ^= 1;
    let cases = [
        (wrong_method, ic_wire::Error::Identity),
        (
            native::encode_request(SAMPLE.method_id, &7_u32, LIMITS).unwrap(),
            ic_wire::Error::Identity,
        ),
        (vec![], ic_wire::Error::Malformed),
        (valid[..valid.len() - 1].to_vec(), ic_wire::Error::Malformed),
        ([valid.as_slice(), &[0]].concat(), ic_wire::Error::Trailing),
        (
            [&valid[..valid.len() - 1], &[0x87, 0]].concat(),
            ic_wire::Error::Malformed,
        ),
        (vec![0; LIMITS.bytes + 1], ic_wire::Error::Limit),
    ];
    for (reply, expected) in cases {
        let length = reply.len();
        let ctx = context(
            "query_call",
            SAMPLE,
            Principal::from_slice(&[1]),
            Principal::from_slice(&[2]),
        );
        let error =
            call_wire::<u32, Result<u64, ()>>(ctx.clone(), SAMPLE, &7, |_| Ok(reply)).unwrap_err();
        assert!(
            std::error::Error::source(&error)
                .unwrap()
                .is::<ic_wire::Error>()
        );
        assert!(!is_dead_pocket_ic_transport_error(&error));
        let WireCallError::Decode {
            context,
            bytes,
            source,
        } = error
        else {
            panic!("decode failure required")
        };
        assert_eq!(*context, ctx);
        assert_eq!(bytes, length);
        assert_eq!(source, expected);
    }
}

#[test]
fn declared_decoder_work_budget_is_not_replaced_with_a_default() {
    let reply = native::encode_response(SAMPLE.method_id, &Ok::<u64, ()>(1), LIMITS).unwrap();
    // Native decoding owns both work-budget and oversized-frame admission.
    for reply in [reply, vec![0; LIMITS.bytes + 1]] {
        let length = reply.len();
        let ctx = context(
            "query_call",
            BAD_REPLY,
            Principal::anonymous(),
            Principal::anonymous(),
        );
        let error = call_wire::<u32, Result<u64, ()>>(ctx.clone(), BAD_REPLY, &1, |_| Ok(reply))
            .unwrap_err();
        let WireCallError::Decode {
            context,
            bytes,
            source,
        } = error
        else {
            panic!("decode failure required")
        };
        assert_eq!(*context, ctx);
        assert_eq!(bytes, length);
        assert_eq!(source, ic_wire::Error::Limit);
    }
}

#[test]
fn rejection_remains_structured_and_is_never_retried() {
    let response = RejectResponse {
        reject_code: RejectCode::CanisterReject,
        reject_message: "original rejection payload".repeat(1024),
        error_code: ErrorCode::CanisterRejectedMessage,
        certified: true,
    };
    let ctx = context(
        "update_call",
        SAMPLE,
        Principal::from_slice(&[1]),
        Principal::from_slice(&[2]),
    );
    let mut calls = 0;
    let error = call_wire::<u32, Result<u64, ()>>(ctx.clone(), SAMPLE, &1, |_| {
        calls += 1;
        Err(response.clone())
    })
    .unwrap_err();
    assert_eq!(calls, 1);
    assert!(error.to_string().len() < 512);
    let WireCallError::Call { context, source } = error else {
        panic!("call failure required")
    };
    assert_eq!(*context, ctx);
    assert!(matches!(*source, PocketIcCallFailure::Rejected(saved) if saved == response));
}

#[test]
fn transport_classification_and_original_unrelated_panics_are_preserved() {
    let ctx = context(
        "query_call",
        SAMPLE,
        Principal::anonymous(),
        Principal::anonymous(),
    );
    let error = call_wire::<u32, Result<u64, ()>>(ctx.clone(), SAMPLE, &1, |_| {
        panic!("HTTP failure: reqwest::Error {{ kind: Request, url: \"http://127.0.0.1:1234/instances/0/read/query\", source: ConnectError(\"tcp connect error\",BORROWED.method_id, BAD_REPLY.method_id, BAD_REQUEST.method_id, SAMPLE.method_id,  Os {{ code: 111, kind: ConnectionRefused, message: \"Connection refused\" }}) }}")
    }).unwrap_err();
    assert!(is_dead_pocket_ic_transport_error(&error));
    let payload = Box::new(9_u8);
    let original = std::ptr::from_ref(payload.as_ref());
    let error = std::panic::catch_unwind(|| {
        call_wire::<u32, Result<u64, ()>>(ctx, SAMPLE, &1, |_| std::panic::panic_any(payload))
    })
    .unwrap_err();
    let resumed = error.downcast::<Box<u8>>().unwrap();
    assert_eq!(std::ptr::from_ref(resumed.as_ref().as_ref()), original);
}

#[test]
fn deserialize_only_and_normalizing_responses_are_admitted() {
    use serde::Deserialize;
    use std::collections::BTreeMap;

    struct ResponseOnly(u32);
    impl<'de> Deserialize<'de> for ResponseOnly {
        fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
            u32::deserialize(decoder).map(Self)
        }
    }
    // Both public helpers accept Deserialize-only responses independently of
    // the request's encoding bound; Postcard still requires a known value shape.
    let _ = super::query_wire::<(), ResponseOnly>;
    let _ = super::update_wire::<(), ResponseOnly>;
    let ctx = context(
        "query_call",
        SAMPLE,
        Principal::anonymous(),
        Principal::anonymous(),
    );
    let decoded = call_wire::<(), ResponseOnly>(ctx.clone(), SAMPLE, &(), |_| {
        Ok(native::encode_response(SAMPLE.method_id, &7_u32, LIMITS).unwrap())
    });
    assert_eq!(decoded.unwrap().0, 7);

    // Keep the caller-owned received frame for byte identity. Wire permits map
    // insertion order to differ from the BTreeMap's normalized iteration order.
    let mut received = native::encode_response(SAMPLE.method_id, &(), LIMITS).unwrap();
    received.extend_from_slice(&[2, 2, 20, 1, 10]);
    let value =
        call_wire::<(), BTreeMap<u8, u8>>(ctx, SAMPLE, &(), |_| Ok(received.clone())).unwrap();
    assert_eq!(value, BTreeMap::from([(1, 10), (2, 20)]));
    let reencoded = native::encode_response(SAMPLE.method_id, &value, LIMITS).unwrap();
    assert_ne!(received, reencoded);
}
