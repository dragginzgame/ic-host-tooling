# Current handoff

## Source and ownership

The checkout remains on committed preparation
`195ec97b99256532a7aae5521f6b628291d7ae51`; its commit label is 0.68.1, while
Cargo's actual workspace version remains 0.68.0. The latest finalized release
notes are 0.68.0 at `8f281177faaeee0adc51edc1031d4a7b268b0ce0`. Local changes
extend the compatible 0.68.1 draft. No own version or Git ref was changed here.

Host, Build and Testkit retain the [extraction boundaries](../extraction.md).
Consumers own credentials, budgets, targets, retries, installation and recovery.
The reviewed Shared Tooling snapshot is 0.26.14 at
`e14050bcc0c627f9c8a9064051699cac44fd7ab3`, recorded in
[the snapshot manifest](../../.shared-tooling.snapshot). Rust edition 2024,
MSRV 1.91 and Linux/native macOS 15 Intel and Apple Silicon remain required.

## Current local work

Manifest minimums and examples select published Wire/derive 0.7.2 and
Platform/macros 0.6.15. Their supplying releases are Wire
`a4da015251beb8fa3346fb3780c0b3dc85c0ef32` and Platform
`6dcf54a229201f85775d218b7a9f41809297878f`. The incoming lockfile already
selected these versions and is preserved byte-for-byte. Feature selections,
existing protocol fields, encoder bytes and the 0.7 decoding contract remain.
See the [0.68.1 notes](../changelog/0.68.md#0681---unreleased).

Shared Tooling adds setup phase diagnostics, safer recovery from manually
committed candidates and distinct busy-tool admission. The local release adapter
now preserves editor status 75 instead of dispatching setup for an admitted busy
slot. Fresh preflight, standalone verification and saved-preparation recovery
have substituted refusal cases. No implicit retry/wait or real release is added.

The pending Host/Build layering repair and its six-case negative fixture remain
in the working tree; their Make/guard/fixture source matches this task's starting
snapshot. Its owning
[#250 — Enforce downward Host and Build dependencies](https://github.com/dragginzgame/ic-tooling/issues/250)
remains open for delivery and required qualification. The maintainer retired
[#249 — Held-fixture-only Platform capabilities in default Testkit](https://github.com/dragginzgame/ic-tooling/issues/249)
in favour of retaining the existing graph rather than adding a published crate.

## Qualification and evidence

Current adoption logs and the task-start snapshot are under
`/tmp/ic-tooling-adopt-20261024/`. Eight Wire adapter cases, one private protocol
case and three held-fixture cases pass on Linux. A real PocketIC Wire
query/update, caller, rejection and trap probe also passes with the prepared
server; no server download was needed. Changed-caller Clippy, actual
Rust 1.91 compilation, all-feature Testkit/observer Wasm and probe Wasm checks
pass. An isolated exact-registry Rust 1.91 consumer confirms that Platform's
Principal and Account satisfy the same Wire 0.7.2 native metadata trait.

Snapshot integrity, prepared Cargo/release-editor admission, dependency/graph
policy, the pending layering fixture, formatting, shell/workflow lint and local
doc links pass. Release-runner and consumer adapter fixtures, including busy
admission, pass with substituted effects. Formatting-hook and Make/jobserver
fixtures pass. Native
macOS qualification of this later dependency/snapshot selection has not run.
Earlier host CI qualifies its own source, not these local changes.

## Dependency blocker and retained history

[Crypto qualification #240](https://github.com/dragginzgame/ic-tooling/issues/240)
remains separate, with the supplying correction at
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50). The selected
graph still contains crypto-bigint 0.5.5. Actual ECDSA signing prevents an
unused-code exemption; no local crypto fork, override or exemption was added.
Platform 0.6.15 and Wire adoption do not resolve that dependency risk.

Keep `/tmp/ic-tooling-249-250/`, `/tmp/ic-tooling-wire-07/` and
`/tmp/ic-tooling-wire-071/` as earlier graph/codec evidence. Original audits remain
under `/tmp/ic-tooling-audit-fixes/`, `/tmp/ic-tooling-audit-2/`,
`/tmp/ic-tooling-audit-2-fixes/` and `/tmp/ic-tooling-shared-02610/`.
Preserve the browser reproduction at `/tmp/ic-tooling-audit-browser-RwZzM9/`.

The nested-workspace target fix is delivered. Preserve its original failed
`target/package-qualification-3180841-0/`, `/tmp/ic-tooling-239/` and
`/tmp/ic-tooling-package-phase/`. Earlier detailed qualification is retained in
[the released handoff](https://github.com/dragginzgame/ic-tooling/blob/248df96b27b44366095d9d8e195cca8a1023bde1/docs/status/current.md).

Migration and local-repository retirement archives remain under
`/tmp/ic-tooling-migration/`, including `retirement/`; original refs were
preserved in the former Host repository. Stale release-lock recovery evidence
remains at `/tmp/ic-tooling-stale-release-lock.qHQFTn/lock`. Retained plans,
failed attempts, original artifacts and backups are not disposable.
