# Current handoff

## Authoritative state

Read the workspace version in [Cargo.toml](../../Cargo.toml), release notes in
[CHANGELOG.md](../../CHANGELOG.md), and exact release identities from Git tags.
[AGENTS.md](../../AGENTS.md) and [the snapshot](../../.shared-tooling.snapshot)
identify the reviewed Shared baseline and exact selected payloads.
The pending [0.27.2 notes](../changelog/0.27.md#0272) describe compatible changes;
released tree-descriptor and stream-copy interfaces remain supported. Versions,
Git effects, publication and releases remain user-owned. Siblings are read-only.

## Ownership

The maintainer's 2026-10-18 direction is a single controlled Candid/CDK dependency
boundary, with IC Wire for owned protocols. The
[proposed boundary](../extraction.md#proposed-fleet-candidcdk-boundary) records the
observed owners, macro/runtime constraints and migration qualification. The new
local `../ic-platform` repository implements the initial bounded decoder and
independent executor features; it is uncommitted and unpublished. Its
`docs/migration.md` owns the adoption contract. Preserve PR #189's prepared source but
hold its independent Host adoption pending consolidation with Testkit's existing
decoder owner. Earlier candidate notes below describe the prepared source, not
approval to establish a second fleet owner. Existing released APIs remain intact.
Siblings remain read-only; no cross-repository migration has been executed.

The [extraction contract](../extraction.md) owns the five crate boundaries:
Artifacts owns bounded byte mechanics, hashing, checksum framing and optional
Wasm/gzip; FS owns filesystem admission, traversal and durable publication;
Process owns execution, custody and observations; Text owns pure rendering and
normalization; Tools owns IC format interpretation, including normal Candid
parsing. The standalone Wasm inspector composes these libraries.

Shared owns common development, release and CI mechanisms. Host retains package
selection, publication preparation custody, runner choices and real integration
checks. Native-host admission and pipe-jobserver observation now call the pinned
Shared helpers; their synthetic matrices stay with the producer. IC Build owns
build/cache orchestration and native Candid extractor execution. Testkit owns
PocketIC setup and lifecycle. Consumers retain budgets, schemas, tool trust,
credentials, targets, retries, recovery and product lifecycle.

Backup's current production adapters already delegate directory hashing to FS,
exact checksum framing to Artifacts and ordered sync/checksum traversal to FS.
The audited Backup source is `3317b4e6e49c0fe59e343069f68eb539324c0612`, selecting
published Host 0.27.1. Its checksum records, expected-identity checks, publication
hooks and recovery remain consumer integration, not competing generic owners.

The earlier [handoff archive](archive-2026-10-18.md) preserves receipts and source
identities verbatim, including the superseded claim that Backup adoption was
pending. Its older 0.28 API migration proposals are not current requirements.
Historical statements and qualification results apply only to their named source;
archives link earlier evidence rather than assigning current ownership.

## Qualification and coordination

Use [host qualification](../hosts.md) for focused checks. Native macOS 15 Intel
and Apple Silicon execution is separate from Linux and substituted-host tests.
The architecture cleanup receipt is `/tmp/host-sediment.vSyJrI/`, relative to its
captured incoming dirty tree at Host `c8cd434`. The current tree includes earlier
FS, Text and logical-size work; this cleanup preserves that source and the index.
Validation for this cleanup is recorded in that receipt and the delivery handoff.

The PR intake receipt is `/tmp/host-pr-issues.G75wqr/`, relative to the captured
incoming working tree at the same Host commit. The compatible 0.27.2 candidate
also prepares [PR #189 — Bounded Candid reply decoding](https://github.com/dragginzgame/ic-host-tooling/pull/189)
and [PR #174 — Scoped shared descriptor lock](https://github.com/dragginzgame/ic-host-tooling/pull/174),
alongside [PR #183 — Cache-directory tag handling](https://github.com/dragginzgame/ic-host-tooling/pull/183).
Decoder bytes/work and aggregate accounting remain explicit; domain admission
stays in Canic. The shared-lock guard explicitly unlocks before close; shared
record lifetimes and cache policy stay in Build. Focused Candid and descriptor-
lock tests pass on current Rust and actual Rust 1.88, with affected all-target/
all-feature MSRV checks, strict Clippy and warning-denied Rustdoc. These APIs
are local preparation, not published availability or complete consumer/native
qualification. PR branch CI applies only to its own named head. The real index,
own versions, dependency selection and normal sibling checkouts are preserved.
Cache-tag handling retains the existing durable writer's ordinary parent creation,
which Build cache initialization requires; its original no-parent-creation claim
is corrected rather than changing that behavior. Native source qualification and
Build adoption remain separate from the focused producer cases.

Shared 0.24.0 is adopted at published main
`cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`, with 114 selected payloads.
Production-role labels retain global formatting and matching source/prepared/
committed release ordering checks. The normal/build graph checker now selects
an explicit native target and covers combined Tools features. Host has no
deployment entrypoint or test-only external catalog; the
[deployment evidence boundary](../hosts.md#dependency-exclusions-and-deployment-evidence)
records what these checks do and do not enforce.
The released Shared baseline does not include the Cargo source-evidence helpers from
[Shared PR #269 — Share Cargo source evidence](https://github.com/dragginzgame/shared-tooling/pull/269).
Their reviewed behavior remains in explicit Host release/publication adapters,
with source provenance in [the publication guide](../publishing.md). The old
snapshot-owned copies retire through canonical export rather than being edited.
Common Cargo setup now selects only the two formatters; Host selects no extractor.
Focused adoption checks pass on Linux: byte-stable catalog formatting, hook
preservation, source/prepared/committed release fixtures, publication admission,
Make routing, snapshot/pins/links and ShellCheck. The common PATH was refreshed
offline from admitted retained binaries, and `tools-check` passes. Graph checks
pass for Linux x86-64 and both macOS triples; they are not native macOS runs.
The lockfile, package versions, pending Rust source and real index are preserved.
No full release gate, release, publication or deployment ran.
GitHub issues remain the sole follow-up tracker. Consumer adoption is coordinated
through [Canic #458 — Adopt published IC Host Tooling](https://github.com/dragginzgame/canic/issues/458)
and [IcyDB #307 — Adopt split Host owners](https://github.com/dragginzgame/icydb/issues/307).
Strict certificate wire admission remains local pending a contract-preserving
upstream replacement; see
[DFINITY #503 — Strict CBOR decoding without byte/array coercions](https://github.com/dfinity/response-verification/issues/503).
Operational-directory CR/LF admission retains its current contract pending
[Canic #575 — Own Backup operational path admission](https://github.com/dragginzgame/canic/issues/575).
