//! Bounded, single-use replies held by `PocketIC`'s manual HTTP mock barrier.
//!
//! The fixture issues no application-specific protocol. Each supplied update slot
//! retains its own raw reply and call context until the driver releases its exact
//! HTTP request. Never enable auto progress/live mode, advance simulated time or
//! upgrade/reinstall/snapshot/restore the fixture while calls are held. HTTP timeout or
//! management rejection rejects the original call; it never releases its reply.
//! Drop the whole isolated instance to abandon pending work. This is a trusted
//! test-control facility, not a deployed service or an authorization boundary.

use super::{RawReplyError, RawReplyLimits, RawReplyMethod, ReplyMode, charge};
use crate::pic::{PocketIcOperationError, try_pocket_ic_operation};
use ic_host_artifacts::wasm_text::{CompileError, CompileLimits, compile};
use ic_platform::{
    candid::contract::encode_one,
    management_canister_types::{HttpMethod, HttpRequestArgs},
    pocket_ic::{
        PocketIc,
        common::rest::{CanisterHttpReply, CanisterHttpResponse, MockCanisterHttpResponse},
    },
    principal::ExternalPrincipal as Principal,
};
use std::{collections::BTreeMap, fmt::Write};
use thiserror::Error;

/// Identity of one observed original call. Fields cannot be forged by callers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeldRequest {
    server: String,
    instance: usize,
    fixture: String,
    slot: usize,
    subnet: Principal,
    request: u64,
}

impl HeldRequest {
    /// Position in the constructor's reply catalog, assigned once for its lifetime.
    #[must_use]
    pub const fn slot(&self) -> usize {
        self.slot
    }
}

/// Refusal to construct, observe or release a held-reply fixture.
#[derive(Debug, Error)]
pub enum HeldReplyError {
    /// Catalog/name/aggregate-byte admission failed.
    #[error(transparent)]
    Catalog(#[from] RawReplyError),
    /// Held replies require replicated update calls.
    #[error("held reply slot {slot} is not an update")]
    Query {
        /// Original catalog position.
        slot: usize,
    },
    /// Automatic progress could make real HTTP requests and bypass driver control.
    #[error("held replies require manual PocketIC progress")]
    AutomaticProgress,
    /// A controller must remain bound to its original instance/server.
    #[error("held reply controller belongs to a different PocketIC instance")]
    WrongInstance,
    /// The exact request is absent, expired, already released or from another fixture.
    #[error("unknown or already released held request")]
    UnknownRequest,
    /// The native external management request could not be encoded.
    #[error("cannot encode HTTP barrier: {0}")]
    Management(String),
    /// Host could not compile the admitted fixture.
    #[error(transparent)]
    Compile(#[from] CompileError),
    /// The `PocketIC` client lost its transport.
    #[error(transparent)]
    Operation(#[from] PocketIcOperationError),
}

/// Native controller and generated Wasm for a single fixture installation.
///
/// Create/fund the canister first, construct this controller, then install
/// [`Self::wasm`] there. Slots with the same method name are consumed in catalog
/// order; distinct methods may arrive in any order. Each slot is used at most
/// once, even after release. Exhaustion rejects. Payloads are ignored, as with
/// the immediate raw-reply fixture; callers correlate observed slots themselves.
///
/// Observation does not advance rounds. Tick with a caller-selected bound until
/// the expected slots appear, perform unrelated ingress, then explicitly release.
/// Do not use `await_call` on the held ingress before release: it cannot complete.
/// Keep simulated time fixed (HTTP has a 60-second timeout). Funding and per-call
/// cycles are caller-owned. A failure requires abandoning the isolated instance;
/// do not retry an uncertain release or reconstruct the controller over live calls.
/// The controller is not cloneable and its drop performs no simulator operations.
#[derive(Debug)]
pub struct HeldReplyFixture {
    wasm: Vec<u8>,
    server: String,
    instance: usize,
    prefix: String,
    released: Vec<bool>,
}

impl HeldReplyFixture {
    /// Generate a native fixture with bounded total calls and immutable replies.
    ///
    /// `limits.methods` bounds lifetime request slots, including repeated names;
    /// reply/name byte limits sum every slot. `module_bytes` bounds compiled Wasm.
    /// No server, canister, installation, funding or tool download is created.
    /// This constructor never accepts attached cycles.
    ///
    /// # Errors
    /// Rejects invalid method limits, oversized modules or failed simulator inspection.
    pub fn new(
        pic: &PocketIc,
        canister: Principal,
        methods: &[RawReplyMethod<'_>],
        limits: RawReplyLimits,
        call_cycles: u128,
        module_bytes: usize,
    ) -> Result<Self, HeldReplyError> {
        Self::build(
            pic,
            canister,
            methods,
            limits,
            call_cycles,
            None,
            module_bytes,
        )
    }

    /// Generate held replies accepting at most `accept_cycles` per admitted call.
    ///
    /// Zero accepts nothing. Acceptance is `min(available, cap)` in the original
    /// update, before its HTTP barrier. It commits with that update; observing the
    /// barrier confirms the commit. Later timeout/rejection does not undo it; the
    /// remainder refunds through the real IC callback path. A call-setup trap rolls
    /// back the update. Construction/installation alone accepts nothing. The cap
    /// applies separately to each slot; HTTP barrier funding is independent.
    ///
    /// # Errors
    /// Rejects invalid method limits, oversized modules or failed simulator inspection.
    pub fn new_with_cycle_acceptance(
        pic: &PocketIc,
        canister: Principal,
        methods: &[RawReplyMethod<'_>],
        limits: RawReplyLimits,
        call_cycles: u128,
        accept_cycles: u128,
        module_bytes: usize,
    ) -> Result<Self, HeldReplyError> {
        Self::build(
            pic,
            canister,
            methods,
            limits,
            call_cycles,
            Some(accept_cycles),
            module_bytes,
        )
    }

    fn build(
        pic: &PocketIc,
        canister: Principal,
        methods: &[RawReplyMethod<'_>],
        limits: RawReplyLimits,
        call_cycles: u128,
        accept_cycles: Option<u128>,
        module_bytes: usize,
    ) -> Result<Self, HeldReplyError> {
        manual(pic)?;
        let prefix = format!("https://ic-testkit.invalid/held/{canister}/");
        let wasm = module(
            methods,
            limits,
            Some((&prefix, call_cycles)),
            accept_cycles,
            module_bytes,
        )?;
        Ok(Self {
            wasm,
            server: pic.get_server_url().to_string(),
            instance: pic.instance_id(),
            prefix,
            released: vec![false; methods.len()],
        })
    }

    /// Exact module bytes to install once into the constructor's canister.
    #[must_use]
    pub fn wasm(&self) -> &[u8] {
        &self.wasm
    }

    /// Observe currently held slots without ticking or settling any call.
    ///
    /// # Errors
    /// Returns an admission or transport error when the instance cannot be inspected.
    pub fn pending(&self, pic: &PocketIc) -> Result<Vec<HeldRequest>, HeldReplyError> {
        self.admit(pic)?;
        Ok(try_pocket_ic_operation(|| pic.get_canister_http())?
            .into_iter()
            .filter_map(|request| {
                let slot = request
                    .url
                    .strip_prefix(&self.prefix)?
                    .parse::<usize>()
                    .ok()?;
                (slot < self.released.len()
                    && !self.released[slot]
                    && request.body == (slot as u64).to_le_bytes())
                .then_some(HeldRequest {
                    server: self.server.clone(),
                    instance: self.instance,
                    fixture: self.prefix.clone(),
                    slot,
                    subnet: request.subnet_id,
                    request: request.request_id,
                })
            })
            .collect())
    }

    /// Release only this exact still-pending request, preserving its original reply.
    ///
    /// Unknown/repeated tokens refuse before mocking. A transport failure after
    /// dispatch is uncertain: the token remains consumed and must not be retried.
    /// The empty mock body is only a barrier signal; application bytes were frozen
    /// in the original slot and are returned by its own callback, never another call.
    ///
    /// # Errors
    /// Rejects unknown or consumed request tokens and preserves transport failures.
    pub fn release(&mut self, pic: &PocketIc, request: &HeldRequest) -> Result<(), HeldReplyError> {
        if !self.pending(pic)?.contains(request) {
            return Err(HeldReplyError::UnknownRequest);
        }
        self.released[request.slot] = true;
        try_pocket_ic_operation(|| {
            pic.mock_canister_http_response(MockCanisterHttpResponse {
                subnet_id: request.subnet,
                request_id: request.request,
                response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
                    status: 200,
                    headers: vec![],
                    body: vec![],
                }),
                additional_responses: vec![],
            });
        })?;
        Ok(())
    }

    fn admit(&self, pic: &PocketIc) -> Result<(), HeldReplyError> {
        if pic.instance_id() != self.instance || pic.get_server_url().as_str() != self.server {
            return Err(HeldReplyError::WrongInstance);
        }
        manual(pic)
    }
}

fn manual(pic: &PocketIc) -> Result<(), HeldReplyError> {
    if try_pocket_ic_operation(|| pic.auto_progress_enabled())? {
        return Err(HeldReplyError::AutomaticProgress);
    }
    Ok(())
}

pub(super) fn module(
    methods: &[RawReplyMethod<'_>],
    limits: RawReplyLimits,
    barrier: Option<(&str, u128)>,
    accept_cycles: Option<u128>,
    module_bytes: usize,
) -> Result<Vec<u8>, HeldReplyError> {
    let exports = admit_catalog(methods, limits)?;
    let mut source = String::from(
        r#"(module
(import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
(import "ic0" "msg_reply" (func $reply))
(import "ic0" "msg_reject" (func $reject (param i32 i32)))
(import "ic0" "call_new" (func $new (param i32 i32 i32 i32 i32 i32 i32 i32)))
(import "ic0" "call_data_append" (func $data (param i32 i32)))
(import "ic0" "call_cycles_add128" (func $cycles (param i64 i64)))
(import "ic0" "call_perform" (func $perform (result i32)))
"#,
    );
    if accept_cycles.is_some() {
        source.push_str(
            "(import \"ic0\" \"msg_cycles_accept128\" (func $accept (param i64 i64 i32)))\n",
        );
    }
    // Keep the aligned acceptance-result scratch separate from immutable replies.
    let mut offset = if accept_cycles.is_some() { 16 } else { 0 };
    let failure = segment(&mut source, &mut offset, b"held reply barrier failed")?;
    let exhausted = segment(&mut source, &mut offset, b"held reply slots exhausted")?;
    let http = segment(&mut source, &mut offset, b"http_request")?;
    writeln!(
        source,
        "(func $failed (param i32) (call $reject (i32.const {failure}) (i32.const 25)))"
    )
    .unwrap();
    for (slot, method) in methods.iter().enumerate() {
        let reply = segment(&mut source, &mut offset, method.reply)?;
        let sending = if let Some(barrier) = barrier {
            barrier_call(&mut source, &mut offset, barrier, http, slot)?
        } else {
            format!("(call $reply{slot} (i32.const 0))")
        };
        writeln!(
            source,
            r"
(global $used{slot} (mut i32) (i32.const 0))
(func $reply{slot} (param i32)
 (call $append (i32.const {reply}) (i32.const {reply_len})) (call $reply))
(func $send{slot}",
            reply_len = method.reply.len()
        )
        .unwrap();
        if let Some(cap) = accept_cycles {
            let high = u64::try_from(cap >> 64).unwrap().cast_signed();
            let low = u64::try_from(cap & u128::from(u64::MAX))
                .unwrap()
                .cast_signed();
            writeln!(
                source,
                "(call $accept (i64.const {high}) (i64.const {low}) (i32.const 0))"
            )
            .unwrap();
        }
        writeln!(source, "{sending})").unwrap();
    }
    for (name, slots) in exports {
        let export = escaped(format!("canister_update {name}").as_bytes());
        writeln!(source, "(func (export \"{export}\")").unwrap();
        for slot in slots {
            writeln!(source, "(if (i32.eqz (global.get $used{slot})) (then (global.set $used{slot} (i32.const 1)) (call $send{slot}) (return)))").unwrap();
        }
        writeln!(
            source,
            "(call $reject (i32.const {exhausted}) (i32.const 26)))"
        )
        .unwrap();
    }
    let pages = offset.div_ceil(65_536).max(1);
    writeln!(source, "(memory (export \"memory\") {pages} {pages})\n(table {} funcref)\n(elem (i32.const 0) $failed", methods.len() + 1).unwrap();
    for slot in 0..methods.len() {
        write!(source, " $reply{slot}").unwrap();
    }
    source.push_str("))");
    Ok(compile(
        &source,
        CompileLimits {
            source_bytes: source.len(),
            module_bytes,
        },
    )?)
}

fn barrier_call(
    source: &mut String,
    offset: &mut usize,
    (prefix, cycles): (&str, u128),
    http: usize,
    slot: usize,
) -> Result<String, HeldReplyError> {
    let request = encode_one(HttpRequestArgs {
        url: format!("{prefix}{slot}"),
        max_response_bytes: Some(256),
        method: HttpMethod::GET,
        headers: vec![],
        body: Some((slot as u64).to_le_bytes().to_vec()),
        transform: None,
        is_replicated: Some(true),
        pricing_version: None,
    })
    .map_err(|error| HeldReplyError::Management(error.to_string()))?;
    let payload = segment(source, offset, &request)?;
    Ok(format!(
        r"
 (call $new (i32.const 0) (i32.const 0) (i32.const {http}) (i32.const 12)
  (i32.const {callback}) (i32.const 0) (i32.const 0) (i32.const 0))
 (call $cycles (i64.const {high}) (i64.const {low}))
 (call $data (i32.const {payload}) (i32.const {payload_len}))
 (if (call $perform) (then unreachable))
",
        callback = slot + 1,
        high = u64::try_from(cycles >> 64).unwrap().cast_signed(),
        low = u64::try_from(cycles & u128::from(u64::MAX))
            .unwrap()
            .cast_signed(),
        payload_len = request.len()
    ))
}

fn admit_catalog<'a>(
    methods: &[RawReplyMethod<'a>],
    limits: RawReplyLimits,
) -> Result<BTreeMap<&'a str, Vec<usize>>, HeldReplyError> {
    charge(0, methods.len(), limits.methods, "requests")?;
    u32::try_from(methods.len()).map_err(|_| RawReplyError::Wasm32Limits)?;
    let mut names = 0;
    let mut replies = 0;
    let mut exports = BTreeMap::<&str, Vec<usize>>::new();
    for (slot, method) in methods.iter().enumerate() {
        if method.mode != ReplyMode::Update {
            return Err(HeldReplyError::Query { slot });
        }
        if method.name.is_empty() || method.name.chars().any(char::is_control) {
            return Err(RawReplyError::InvalidName { index: slot }.into());
        }
        names = charge(
            names,
            method.name.len(),
            limits.method_name_bytes,
            "method names",
        )?;
        replies = charge(
            replies,
            method.reply.len(),
            limits.reply_bytes,
            "reply bytes",
        )?;
        exports.entry(method.name).or_default().push(slot);
    }
    Ok(exports)
}

fn segment(source: &mut String, offset: &mut usize, bytes: &[u8]) -> Result<usize, RawReplyError> {
    let start = *offset;
    *offset = offset
        .checked_add(bytes.len())
        .filter(|end| i32::try_from(*end).is_ok())
        .ok_or(RawReplyError::Wasm32Limits)?;
    writeln!(source, "(data (i32.const {start}) \"{}\")", escaped(bytes)).unwrap();
    Ok(start)
}

fn escaped(bytes: &[u8]) -> String {
    let mut result = String::new();
    for byte in bytes {
        write!(result, "\\{byte:02x}").unwrap();
    }
    result
}

#[cfg(test)]
mod tests;
