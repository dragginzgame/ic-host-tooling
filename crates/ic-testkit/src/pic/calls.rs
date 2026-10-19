use super::{CandidCallContext, CandidCallError, transport};
use ic_platform::candid::{DecodeError, DecodeLimits, decode_args};
use ic_platform::pocket_ic::{PocketIc, RejectResponse};
use ic_platform::{
    Principal,
    candid::contract::{ArgumentDecoder, ArgumentEncoder, encode_args},
};

#[derive(Clone, Copy)]
struct CallContext<'a> {
    operation: &'static str,
    canister_id: Principal,
    caller: Principal,
    method: &'a str,
}

impl CallContext<'_> {
    fn to_error_context(self) -> CandidCallContext {
        CandidCallContext::new(self.operation, self.canister_id, self.caller, self.method)
    }
}

/// Execute an update as `caller`, decoding the top-level Candid reply arguments.
///
/// `(u64, bool)` means two reply arguments; `((u64, bool),)` means one
/// tuple-valued reply. Use `()` for no replies or
/// `(T,)` for one reply. Pass [`Principal::anonymous()`] for an anonymous caller.
/// Application-level `Result<T, E>` values are returned without unwrapping.
pub fn update_candid_args<T, A>(
    pic: &PocketIc,
    canister_id: Principal,
    caller: Principal,
    method: &str,
    args: A,
    limits: DecodeLimits,
) -> Result<T, CandidCallError>
where
    T: for<'de> ArgumentDecoder<'de>,
    A: ArgumentEncoder,
{
    let context = CallContext {
        operation: "update_call",
        canister_id,
        caller,
        method,
    };
    call_candid(
        context,
        args,
        |bytes| pic.update_call(canister_id, caller, method, bytes),
        |bytes| decode_args(bytes, limits),
    )
}

/// Execute a query as `caller`, decoding the top-level Candid reply arguments.
///
/// `(u64, bool)` means two reply arguments; `((u64, bool),)` means one
/// tuple-valued reply. Use `()` for no replies or
/// `(T,)` for one reply. Pass [`Principal::anonymous()`] for an anonymous caller.
/// Application-level `Result<T, E>` values are returned without unwrapping.
pub fn query_candid_args<T, A>(
    pic: &PocketIc,
    canister_id: Principal,
    caller: Principal,
    method: &str,
    args: A,
    limits: DecodeLimits,
) -> Result<T, CandidCallError>
where
    T: for<'de> ArgumentDecoder<'de>,
    A: ArgumentEncoder,
{
    let context = CallContext {
        operation: "query_call",
        canister_id,
        caller,
        method,
    };
    call_candid(
        context,
        args,
        |bytes| pic.query_call(canister_id, caller, method, bytes),
        |bytes| decode_args(bytes, limits),
    )
}

fn call_candid<T, A, F, D>(
    context: CallContext<'_>,
    args: A,
    call: F,
    decode: D,
) -> Result<T, CandidCallError>
where
    A: ArgumentEncoder,
    F: FnOnce(Vec<u8>) -> Result<Vec<u8>, RejectResponse>,
    D: FnOnce(&[u8]) -> Result<T, DecodeError>,
{
    let bytes = encode_args(args).map_err(|err| CandidCallError::Encode {
        context: Box::new(context.to_error_context()),
        message: err.to_string(),
    })?;
    let result = run_raw_call(context, || call(bytes))?;
    decode(&result).map_err(|source| CandidCallError::Decode {
        context: Box::new(context.to_error_context()),
        bytes: result.len(),
        source: Box::new(source),
    })
}

fn run_raw_call<F>(context: CallContext<'_>, call: F) -> Result<Vec<u8>, CandidCallError>
where
    F: FnOnce() -> Result<Vec<u8>, RejectResponse>,
{
    match transport::try_pocket_ic_operation(call) {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(response)) => Err(CandidCallError::CanisterReject {
            context: Box::new(context.to_error_context()),
            response: Box::new(response),
        }),
        Err(source) => Err(CandidCallError::Transport {
            context: Box::new(context.to_error_context()),
            source,
        }),
    }
}

#[cfg(test)]
mod tests {
    use ic_platform::Principal;
    use ic_platform::candid::{DecodeError, DecodeLimits, decode_args};

    use super::{CallContext, CandidCallError, call_candid, run_raw_call};

    const LIMITS: DecodeLimits = DecodeLimits {
        reply_bytes: 65_536,
        decoding_quota: 1_000_000,
        skipping_quota: 100_000,
        type_entries: 1024,
        header_bytes: 8192,
    };

    #[test]
    fn decode_error_includes_call_context() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "get",
        };

        let err = call_candid(
            context,
            (),
            |_| Ok(vec![0xde, 0xad]),
            |bytes| decode_args::<(u64,)>(bytes, LIMITS),
        )
        .expect_err("decode fails");

        assert!(err.to_string().contains("candid decode failed"));
        assert!(err.to_string().contains("operation=query_call"));
        assert!(err.to_string().contains("method=get"));
        assert!(err.to_string().contains("bytes=2"));
        assert!(std::error::Error::source(&err).unwrap().is::<DecodeError>());
        let CandidCallError::Decode {
            context,
            bytes,
            source,
        } = err
        else {
            panic!("expected decode failure");
        };
        assert_eq!(bytes, 2);
        assert_eq!(context.method, "get");
        assert!(matches!(*source, DecodeError::Candid(_)));
    }

    #[test]
    fn caller_decode_limits_retain_typed_refusals() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::anonymous(),
            method: "get",
        };
        let bytes = ic_platform::candid::contract::encode_one(42_u64).unwrap();
        let length = bytes.len();
        let error = call_candid(
            context,
            (),
            |_| Ok(bytes),
            |bytes| {
                decode_args::<(u64,)>(
                    bytes,
                    DecodeLimits {
                        reply_bytes: 0,
                        ..LIMITS
                    },
                )
            },
        )
        .unwrap_err();
        let cause = std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<DecodeError>()
            .unwrap();
        assert!(
            matches!(cause, DecodeError::ReplyTooLarge { actual, maximum: 0 } if *actual == length)
        );

        // Tiny wire data can still describe expensive values: work is bounded
        // independently of the reply length, including the arguments API.
        let bytes = ic_platform::candid::contract::encode_args((vec![(); 100_000],)).unwrap();
        let error = call_candid(
            context,
            (),
            |_| Ok(bytes),
            |bytes| {
                decode_args::<(Vec<()>,)>(
                    bytes,
                    DecodeLimits {
                        decoding_quota: 0,
                        ..LIMITS
                    },
                )
            },
        )
        .unwrap_err();
        let cause = std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<DecodeError>()
            .unwrap();
        assert!(matches!(cause, DecodeError::Candid(_)));
        assert!(std::error::Error::source(cause).is_some());
    }

    #[test]
    fn reply_arguments_preserve_top_level_arity_and_wide_values() {
        let context = CallContext {
            operation: "update_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "measure",
        };
        let values = (u128::MAX, u128::from(u64::MAX) + 1);
        let bytes = ic_platform::candid::contract::encode_args(values).unwrap();
        assert_eq!(
            ic_platform::candid::decode_args::<(u128, u128)>(&bytes, LIMITS).unwrap(),
            values
        );
        assert!(ic_platform::candid::decode_one::<(u128, u128)>(&bytes, LIMITS).is_err());
        let tuple = ic_platform::candid::contract::encode_one(values).unwrap();
        assert!(ic_platform::candid::decode_args::<(u128, u128)>(&tuple, LIMITS).is_err());
        assert_eq!(
            ic_platform::candid::decode_one::<(u128, u128)>(&tuple, LIMITS).unwrap(),
            values
        );
        let no_replies = ic_platform::candid::contract::encode_args(()).unwrap();
        ic_platform::candid::decode_args::<()>(&no_replies, LIMITS).unwrap();
        let one_reply = ic_platform::candid::contract::encode_args((7_u64,)).unwrap();
        assert_eq!(
            ic_platform::candid::decode_args::<(u64,)>(&one_reply, LIMITS).unwrap(),
            (7,)
        );
        let result: (u64, bool, String) = call_candid(
            context,
            (42_u64, false),
            |args| {
                assert_eq!(
                    ic_platform::candid::decode_args::<(u64, bool)>(&args, LIMITS).unwrap(),
                    (42, false)
                );
                Ok(ic_platform::candid::contract::encode_args((9_u64, true, "ready")).unwrap())
            },
            |bytes| decode_args(bytes, LIMITS),
        )
        .unwrap();
        assert_eq!(result, (9, true, "ready".into()));
    }

    #[test]
    fn invalid_reply_arguments_keep_decode_context() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "measure",
        };
        for bytes in [
            vec![0xde, 0xad],
            ic_platform::candid::contract::encode_args((7_u64,)).unwrap(),
        ] {
            let length = bytes.len();
            let err = call_candid(
                context,
                (),
                |_| Ok(bytes),
                |bytes| decode_args::<(u64, bool)>(bytes, LIMITS),
            )
            .unwrap_err();
            assert!(err.to_string().contains(&format!("bytes={length}")));
            let CandidCallError::Decode {
                context: observed,
                bytes,
                ..
            } = err
            else {
                panic!("expected decode failure");
            };
            assert_eq!(bytes, length);
            assert_eq!(observed.operation, "query_call");
            assert_eq!(observed.canister_id, context.canister_id);
            assert_eq!(observed.caller, context.caller);
            assert_eq!(observed.method, "measure");
        }
    }

    #[test]
    fn dead_instance_panic_is_classified_as_transport() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "get",
        };
        let error = run_raw_call(context, || -> Result<Vec<u8>, ic_platform::pocket_ic::RejectResponse> {
            panic!("HTTP failure: reqwest::Error {{ kind: Request, url: \"http://127.0.0.1:1234/instances/0/update/execute_ingress_message\", source: ConnectError(\"tcp connect error\", Os {{ code: 111, kind: ConnectionRefused, message: \"Connection refused\" }}) }}");
        })
        .unwrap_err();

        let source = std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<super::super::PocketIcOperationError>()
            .unwrap();
        assert!(source.is_transport());
        assert!(source.message().starts_with("HTTP failure: "));
        assert!(error.to_string().contains("operation=query_call"));
        assert!(super::super::is_dead_pocket_ic_transport_error(&error));
        let CandidCallError::Transport { source: owned, .. } = error else {
            panic!("expected transport failure");
        };
        assert!(owned.is_transport());
    }

    #[test]
    fn unrelated_panics_resume_with_the_original_payload() {
        for message in [
            "application worker channel closed",
            "fixture expected ConnectionRefused but observed another result",
        ] {
            let context = CallContext {
                operation: "query_call",
                canister_id: Principal::anonymous(),
                caller: Principal::management_canister(),
                method: "get",
            };
            let payload = message.to_owned();
            let original_allocation = payload.as_ptr() as usize;
            let result = std::panic::catch_unwind(|| {
                run_raw_call(
                    context,
                    || -> Result<Vec<u8>, ic_platform::pocket_ic::RejectResponse> {
                        std::panic::panic_any(payload);
                    },
                )
            });
            let resumed = result
                .expect_err("application panic must resume")
                .downcast::<String>()
                .expect("original panic type must be preserved");
            assert_eq!(resumed.as_str(), message);
            assert_eq!(resumed.as_ptr() as usize, original_allocation);
        }
        let payload = Box::new(7_u8);
        let original = std::ptr::from_ref(payload.as_ref());
        let result = std::panic::catch_unwind(|| {
            run_raw_call(
                CallContext {
                    operation: "query_call",
                    canister_id: Principal::anonymous(),
                    caller: Principal::anonymous(),
                    method: "get",
                },
                || -> Result<Vec<u8>, ic_platform::pocket_ic::RejectResponse> {
                    std::panic::panic_any(payload);
                },
            )
        });
        let resumed = result.unwrap_err().downcast::<Box<u8>>().unwrap();
        assert_eq!(std::ptr::from_ref(resumed.as_ref().as_ref()), original);
    }
}
