# Current handoff

## Authoritative state

Read the workspace version in [Cargo.toml](../../Cargo.toml), released notes in
[CHANGELOG.md](../../CHANGELOG.md), and exact release identities from Git tags.
Do not copy those version facts into this handoff. The selected Shared baseline
and exact payload identities are in [AGENTS.md](../../AGENTS.md) and
[the snapshot](../../.shared-tooling.snapshot).

The [0.25 migration](../changelog/0.25.md) documents the pending breaking API
changes: one tool-probe configuration, consumer-owned replica install policy and
unconditional Candid service inspection in Host Tools. Durable descriptor and
existing-parent writes now require explicit result-admission callbacks.
The numbered changelog draft owns the candidate version; manifest/version changes
and release execution remain user-owned. Existing sibling checkouts stay read-only.

## Ownership and evidence

The [extraction contract](../extraction.md) owns the five crate boundaries.
Host Tools retains shared Candid interpretation with a normal parser dependency;
IC Build owns extractor execution. Consumers retain tool trust, replica revisions,
byte budgets, schemas, targets,
credentials, retries and lifecycle. There are no compatibility reexports.
Use [host qualification](../hosts.md) to select focused checks; Linux evidence
never substitutes for native macOS 15 qualification on both architectures.

The [previous handoff](archive-2026-10-17.md) preserves historical source identities,
commands, failures and receipts verbatim. Its pending/released statements are
historical observations, not current status. Earlier archives remain linked from
that document. Keep new receipts bound to their exact source and inputs rather
than restating earlier results as current qualification.

GitHub issues are the sole follow-up tracker. Consumer adoption is recorded in
[Canic #458 — Adopt published IC Host Tooling](https://github.com/dragginzgame/canic/issues/458)
and [IcyDB #307 — Adopt split Host owners](https://github.com/dragginzgame/icydb/issues/307).
The current work's qualification and migration patch are reported at handoff.

Publication interruption recovery stays split between Host preparation custody
and Shared dispatch receipts. Strict certificate wire admission remains local
until an upstream replacement preserves its tests; see
[DFINITY #503 — Strict CBOR decoding without byte/array coercions](https://github.com/dfinity/response-verification/issues/503).
Operational directory CR/LF admission remains in Host pending consumer-owned
migration; see [Canic #575 — Own Backup operational path admission](https://github.com/dragginzgame/canic/issues/575).

## Qualification receipt

The preceding simplification slice's Linux receipts and captured sources remain at
`/tmp/host025-simplify.VHKQBw/`. The compiled Host inputs are bound by
`host-source.sha256` (manifest digest
`52b879408333a2071b5e29fa2ce1adc8fd2030ceeb7201b6c4ec90a6ecce9e77`).
Tool cases pass on current Rust and actual Rust 1.88; affected all-target checks,
current strict Clippy/rustdoc, feature graphs and Make routing pass. The frozen
Build rehearsal passes Rust 1.91 all-target compilation and targeted artifact/Candid
cases. The captured Canic module passes its relocated policy and boundary probes
with unrelated DTO stand-ins; it does not qualify the complete Canic graph.
That slice's sibling migration patches passed apply checks; normal checkouts and
selected registry graphs are untouched. Testkit/IcyDB execution and native macOS
qualification are not claimed. Consumer comments remain unposted pending permission.

The Candid dependency change is qualified separately at
`/tmp/host025-candid-default.NlxHH1/`, with inputs bound in
`tools-source.sha256`. All 14 default cases pass on current Rust and actual
Rust 1.88; the 16 interface and 19 certificate cases pass on current Rust.
Affected all-target MSRV compilation, strict current Clippy/rustdoc, graph
boundaries, Make routing, sorting, formatting and local links pass. Refreshed
Canic/IcyDB patches remove the retired `candid-service` selection and apply cleanly;
consumer execution and native macOS qualification remain separate. Package versions,
lock selection, normal sibling checkouts, index and refs remain preserved.

Response/certificate error derives are qualified at
`/tmp/host025-error-derive.yyU5Uk/`, with inputs in `tools-source.sha256`.
Both new source-chain/redaction regressions pass against the original handwritten
implementations and the derives. All 23 Tools cases pass on current Rust and
actual Rust 1.88; strict Clippy/rustdoc, graph checks, formatting, sorting and local
links pass. Candid source and dependency selection are unchanged by this slice;
shared parsing extraction waits on the upstream direction. Native macOS remains
separate. The only new lock edge selects the already-locked thiserror release.

Fixture custody and durable-admission receipts are retained at
`/tmp/host025-custody-cut.kCAiqk/`, bound by `host-source.sha256`.
All 119 FS cases pass on current Rust and actual Rust 1.88. Current Process
validation passes 119 cases in the sandbox; the four loopback fixtures initially
refused sandbox socket creation and pass separately outside it. Actual Rust 1.88
passes the 71 tool and three barrier cases, with all-target affected compilation.
Strict current Clippy/rustdoc, formatting, sorting and links pass. Frozen Canic
Backup compiles on Rust 1.99 and passes selected stop-recovery and manifest-crash
cases; its 1.91 test compilation refuses existing `std::assert_matches!` use.
The reduced workspace retains other registry selections and overrides only three
Host sources; it is not complete Canic graph acceptance. The exact Query confined
module passes 18 cases on Rust 1.91; Blob's exact artifact module passes seven
with its unrelated CLI error enum represented by a stand-in. Query/Blob probes
are scoped module evidence, not full consumer acceptance. Canic CLI's ignored
live-environment control is not run. All three migration patches pass apply checks;
normal sibling checkouts, Candid source, own versions, lock, index and refs remain
preserved. No consumer comment was posted or native macOS acceptance claimed.
