//! Native typed transport; IC Wire remains the sole frame and codec owner.

use super::{PocketIc, PocketIcCallFailure, RejectResponse, transport};
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_wire::Method;
use serde::de::DeserializeOwned;

/// Target, caller and method identity attached to a Wire call failure.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireCallContext {
    /// Underlying PocketIC operation: query_call or update_call.
    pub operation: &'static str,
    /// Target canister selected by the caller.
    pub canister_id: Principal,
    /// Explicit effective message caller.
    pub caller: Principal,
    /// Exact exported name from the selected Method.
    pub method: &'static str,
    /// Wire identity from the selected Method.
    pub method_id: u64,
}

/// Codec or transport refusal with typed causes and bounded diagnostics.
///
/// Raw reply bytes are not retained or rendered. Use PocketIC's raw methods when
/// original bytes or deliberately malformed requests are required as evidence.
#[non_exhaustive]
#[derive(Debug)]
pub enum WireCallError {
    /// Request encoding failed before transport was invoked.
    Encode {
        /// Target, caller, operation and method identity.
        context: Box<WireCallContext>,
        /// Original Wire refusal.
        source: ic_wire::Error,
    },
    /// The underlying operation rejected or lost its transport.
    Call {
        /// Target, caller, operation and method identity.
        context: Box<WireCallContext>,
        /// Complete rejection or original classified operation failure.
        source: Box<PocketIcCallFailure>,
    },
    /// The returned bytes failed the selected method's response admission.
    Decode {
        /// Target, caller, operation and method identity.
        context: Box<WireCallContext>,
        /// Number of already allocated PocketIC reply bytes.
        bytes: usize,
        /// Original Wire refusal, including oversized reply admission.
        source: ic_wire::Error,
    },
}

impl std::fmt::Display for WireCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let context = match self {
            Self::Encode { context, source } => {
                write!(f, "Wire request encoding failed: {source}")?;
                context
            }
            Self::Call { context, source } => {
                match source.as_ref() {
                    PocketIcCallFailure::Rejected(response) => write!(
                        f,
                        "PocketIC call rejected ({:?}, {:?})",
                        response.reject_code, response.error_code
                    )?,
                    PocketIcCallFailure::Operation(_) => {
                        f.write_str("PocketIC transport failed")?;
                    }
                }
                context
            }
            Self::Decode {
                context,
                bytes,
                source,
            } => {
                write!(f, "Wire response decoding failed (bytes={bytes}): {source}")?;
                context
            }
        };
        write!(
            f,
            " (operation={}, canister={}, caller={}, method={}, method_id={})",
            context.operation,
            context.canister_id,
            context.caller,
            context.method,
            context.method_id
        )
    }
}

impl std::error::Error for WireCallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encode { source, .. } | Self::Decode { source, .. } => Some(source),
            Self::Call { source, .. } => Some(source.as_ref()),
        }
    }
}

/// Encode a method-bound request, execute one query as `caller`, and decode its reply.
///
/// Method owns the exact name, types, identity and finite request/response limits.
/// Explicit anonymous callers are permitted. Application-level results are returned
/// unchanged. Known PocketIC transport failures are captured; unrelated panics resume.
/// Requests may borrow their input; responses must own their decoded data because
/// the helper releases PocketIC's reply buffer before returning.
pub fn query_wire<M: Method>(
    pic: &PocketIc,
    canister_id: Principal,
    caller: Principal,
    request: &M::Request,
) -> Result<M::Response, WireCallError>
where
    M::Response: DeserializeOwned,
{
    call_wire::<M>(
        context::<M>("query_call", canister_id, caller),
        request,
        |bytes| pic.query_call(canister_id, caller, M::NAME, bytes),
    )
}

/// Encode a method-bound request and execute one update as `caller`, without retry.
///
/// The response uses the same admission as query_wire. No installation, authority,
/// effect reconciliation or replay policy is selected by this helper.
/// A refused reply does not establish that the update had no effect.
pub fn update_wire<M: Method>(
    pic: &PocketIc,
    canister_id: Principal,
    caller: Principal,
    request: &M::Request,
) -> Result<M::Response, WireCallError>
where
    M::Response: DeserializeOwned,
{
    call_wire::<M>(
        context::<M>("update_call", canister_id, caller),
        request,
        |bytes| pic.update_call(canister_id, caller, M::NAME, bytes),
    )
}

const fn context<M: Method>(
    operation: &'static str,
    canister_id: Principal,
    caller: Principal,
) -> WireCallContext {
    WireCallContext {
        operation,
        canister_id,
        caller,
        method: M::NAME,
        method_id: M::ID,
    }
}

fn call_wire<M: Method>(
    context: WireCallContext,
    request: &M::Request,
    call: impl FnOnce(Vec<u8>) -> Result<Vec<u8>, RejectResponse>,
) -> Result<M::Response, WireCallError>
where
    M::Response: DeserializeOwned,
{
    let request =
        ic_wire::encode_request::<M>(request).map_err(|source| WireCallError::Encode {
            context: Box::new(context.clone()),
            source,
        })?;
    let reply = transport::call_or_resume(|| call(request.into_bytes())).map_err(|source| {
        WireCallError::Call {
            context: Box::new(context.clone()),
            source: Box::new(source),
        }
    })?;
    let response = if reply.len() > M::RESPONSE_LIMITS.bytes {
        Err(ic_wire::Error::Limit)
    } else {
        ic_wire::decode_response::<M>(&reply)
    };
    response.map_err(|source| WireCallError::Decode {
        context: Box::new(context),
        bytes: reply.len(),
        source,
    })
}

#[cfg(test)]
mod tests;
