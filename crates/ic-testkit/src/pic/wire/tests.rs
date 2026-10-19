use super::{WireCallError, call_wire, context};
use crate::pic::{
    ErrorCode, PocketIcCallFailure, RejectCode, RejectResponse, is_dead_pocket_ic_transport_error,
};
use ic_platform::Principal;
use ic_wire::{Limits, Method};

const LIMITS: Limits = Limits {
    bytes: 64,
    allocation: 128,
    nodes: 8,
    depth: 4,
};

#[test]
fn borrowed_requests_return_owned_responses() {
    use ic_wire::{Bytes, BytesRef};
    use std::marker::PhantomData;

    struct Borrowed<'a>(PhantomData<&'a ()>);
    impl<'a> Method for Borrowed<'a> {
        type Request = BytesRef<'a, 8>;
        type Response = Bytes<8>;
        const ID: u64 = 73;
        const NAME: &'static str = "borrowed";
        const REQUEST_LIMITS: Limits = LIMITS;
        const RESPONSE_LIMITS: Limits = LIMITS;
    }

    let _ = super::query_wire::<Borrowed<'_>>;
    let _ = super::update_wire::<Borrowed<'_>>;
    let result = {
        let payload = vec![1, 2, 3];
        let request = BytesRef::new(&payload).unwrap();
        let mut calls = 0;
        let response = call_wire::<Borrowed<'_>>(
            context::<Borrowed<'_>>(
                "update_call",
                Principal::anonymous(),
                Principal::anonymous(),
            ),
            &request,
            |frame| {
                calls += 1;
                assert_eq!(
                    ic_wire::decode_request::<Borrowed<'_>>(&frame)
                        .unwrap()
                        .as_slice(),
                    payload
                );
                Ok(
                    ic_wire::encode_response::<Borrowed<'_>>(&Bytes::new(vec![3, 2, 1]).unwrap())
                        .unwrap()
                        .into_bytes(),
                )
            },
        )
        .unwrap();
        assert_eq!(calls, 1);
        response
    };
    assert_eq!(result.as_slice(), &[3, 2, 1]);
}

struct Sample;
impl Method for Sample {
    type Request = u32;
    type Response = Result<u64, ()>;
    const ID: u64 = 71;
    const NAME: &'static str = "sample";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

struct BadRequestBudget;
impl Method for BadRequestBudget {
    type Request = u32;
    type Response = ();
    const ID: u64 = 72;
    const NAME: &'static str = "bad_request_budget";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 11,
        ..LIMITS
    };
    const RESPONSE_LIMITS: Limits = LIMITS;
}

struct BadReplyBudget;
impl Method for BadReplyBudget {
    type Request = u32;
    type Response = Result<u64, ()>;
    const ID: u64 = Sample::ID;
    const NAME: &'static str = "bad_reply_budget";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = Limits { nodes: 1, ..LIMITS };
}

#[test]
fn one_call_preserves_method_context_and_application_results() {
    for operation in ["query_call", "update_call"] {
        for value in [Ok(u64::MAX), Err(())] {
            let ctx = context::<Sample>(
                operation,
                Principal::from_slice(&[1]),
                Principal::from_slice(&[2]),
            );
            assert_eq!(ctx.operation, operation);
            assert_eq!(ctx.canister_id, Principal::from_slice(&[1]));
            assert_eq!(ctx.caller, Principal::from_slice(&[2]));
            assert_eq!(ctx.method, Sample::NAME);
            assert_eq!(ctx.method_id, Sample::ID);
            let mut calls = 0;
            let result = call_wire::<Sample>(ctx, &u32::MAX, |bytes| {
                calls += 1;
                assert_eq!(ic_wire::decode_request::<Sample>(&bytes), Ok(u32::MAX));
                Ok(ic_wire::encode_response::<Sample>(&value)
                    .unwrap()
                    .into_bytes())
            })
            .unwrap();
            assert_eq!(result, value);
            assert_eq!(calls, 1);
        }
    }
}

#[test]
fn request_admission_precedes_transport() {
    let error = call_wire::<BadRequestBudget>(
        context::<BadRequestBudget>(
            "update_call",
            Principal::anonymous(),
            Principal::anonymous(),
        ),
        &1,
        |_| panic!("refused request must not reach transport"),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        WireCallError::Encode {
            source: ic_wire::Error::SchemaLimit,
            ..
        }
    ));
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<ic_wire::Error>()
    );
}

#[test]
fn invalid_responses_preserve_typed_refusal_and_context_without_payload_logs() {
    let valid = ic_wire::encode_response::<Sample>(&Ok(7))
        .unwrap()
        .into_bytes();
    let mut wrong_method = valid.clone();
    wrong_method[2] ^= 1;
    let cases = [
        (wrong_method, ic_wire::Error::Identity),
        (
            ic_wire::encode_request::<Sample>(&7).unwrap().into_bytes(),
            ic_wire::Error::Identity,
        ),
        (vec![], ic_wire::Error::Malformed),
        (valid[..valid.len() - 1].to_vec(), ic_wire::Error::Malformed),
        ([valid.as_slice(), &[0]].concat(), ic_wire::Error::Trailing),
        (vec![0; LIMITS.bytes + 1], ic_wire::Error::Limit),
    ];
    for (reply, expected) in cases {
        let length = reply.len();
        let ctx = context::<Sample>(
            "query_call",
            Principal::from_slice(&[1]),
            Principal::from_slice(&[2]),
        );
        let error = call_wire::<Sample>(ctx.clone(), &7, |_| Ok(reply)).unwrap_err();
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
    let reply = ic_wire::encode_response::<Sample>(&Ok(1))
        .unwrap()
        .into_bytes();
    let error = call_wire::<BadReplyBudget>(
        context::<BadReplyBudget>("query_call", Principal::anonymous(), Principal::anonymous()),
        &1,
        |_| Ok(reply),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        WireCallError::Decode {
            source: ic_wire::Error::SchemaLimit,
            ..
        }
    ));
}

#[test]
fn rejection_remains_structured_and_is_never_retried() {
    let response = RejectResponse {
        reject_code: RejectCode::CanisterReject,
        reject_message: "original rejection payload".repeat(1024),
        error_code: ErrorCode::CanisterRejectedMessage,
        certified: true,
    };
    let ctx = context::<Sample>(
        "update_call",
        Principal::from_slice(&[1]),
        Principal::from_slice(&[2]),
    );
    let mut calls = 0;
    let error = call_wire::<Sample>(ctx.clone(), &1, |_| {
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
    let ctx = context::<Sample>("query_call", Principal::anonymous(), Principal::anonymous());
    let error = call_wire::<Sample>(ctx.clone(), &1, |_| {
        panic!("HTTP failure: reqwest::Error {{ kind: Request, url: \"http://127.0.0.1:1234/instances/0/read/query\", source: ConnectError(\"tcp connect error\", Os {{ code: 111, kind: ConnectionRefused, message: \"Connection refused\" }}) }}")
    }).unwrap_err();
    assert!(is_dead_pocket_ic_transport_error(&error));
    let payload = Box::new(9_u8);
    let original = std::ptr::from_ref(payload.as_ref());
    let error = std::panic::catch_unwind(|| {
        call_wire::<Sample>(ctx, &1, |_| std::panic::panic_any(payload))
    })
    .unwrap_err();
    let resumed = error.downcast::<Box<u8>>().unwrap();
    assert_eq!(std::ptr::from_ref(resumed.as_ref().as_ref()), original);
}
