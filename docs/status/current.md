# Current handoff

## Authoritative state

Read the workspace version in [Cargo.toml](../../Cargo.toml), release notes in
[CHANGELOG.md](../../CHANGELOG.md), and exact release identities from Git tags.
[AGENTS.md](../../AGENTS.md) and [the snapshot](../../.shared-tooling.snapshot)
identify the reviewed Shared baseline and exact selected payloads.
The pending [0.27.3 notes](../changelog/0.27.md#0273) describe compatible changes;
released tree-descriptor and stream-copy interfaces remain supported. Versions,
Git effects, publication and releases remain user-owned. Siblings are read-only.

## Ownership

The maintainer's 2026-10-18 direction is a single controlled Candid/CDK dependency
boundary, with IC Wire for owned protocols. The
[boundary](../extraction.md#fleet-candidcdk-boundary) records the observed owners,
macro/runtime constraints and migration qualification. Host now selects published
Platform 0.1.4 for Candid decoding, native parsing and optional certification.
Its `docs/migration.md` owns the fleet adoption contract. The released Host
aggregate-byte adapter delegates per-reply decoding and retains its limits,
typed errors and debit ordering; diagnostics follow Platform's compact policy.
Host retains strict CBOR admission and service inventory projection. Siblings
remain read-only; this work does not establish their migration status.

The [extraction contract](../extraction.md) owns the five crate boundaries:
Artifacts owns bounded byte mechanics, hashing, checksum framing and optional
Wasm/gzip; FS owns filesystem admission, traversal and durable publication;
Process owns execution, custody and observations; Text owns pure rendering and
normalization; Tools owns IC format interpretation, including normal Candid
parsing. The standalone Wasm inspector composes these libraries.

Shared owns common development, release and CI mechanisms. Host retains package
selection, metadata rendering and qualification, publication preparation custody,
runner choices and real integration checks. Release metadata backup, installation,
caught-failure restoration and receipt archiving use pinned Shared helpers; Host
retains receipt schemas and recovery decisions. Native-host admission and
pipe-jobserver observation now call the pinned
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

The working tree prepares PR #154's protected in-place export API for
[#153 — Own non-aliasing in-place export writes](https://github.com/dragginzgame/ic-host-tooling/issues/153),
using the current path canonicalizer, native identity owner and derived errors.
Query still owns its normal export implementation and its protected-path policy;
adoption after Host delivery is tracked in
[Query #69 — Adopt published Host non-aliasing export writes](https://github.com/dragginzgame/ic-query/issues/69).
Receipt: `/tmp/host-export-intake.2AGekX/`; incoming work and sibling checkouts remain
preserved. No package version, commit, merge or publication is implied.
Eight focused export cases pass on current Rust and actual Rust 1.88, with strict
FS Clippy, Rustdoc, formatting, links and changelog admission. Combined-source
native macOS qualification and Query adoption remain separate. PR #100 was closed
as superseded by canonical Shared integration in the CI-qualified Host 0.27.2.

PR #180's admitted symlink publication is prepared in the 0.27.3 working tree for
[#121 — Extract retained tool-bundle publication](https://github.com/dragginzgame/ic-host-tooling/issues/121).
It uses FS's native identity owner and existing publication errors. Five focused
cases pass on current Rust and actual Rust 1.88, including faults before/after
rename and observed staging replacement; strict FS Clippy and Rustdoc pass.
Receipt: `/tmp/host-pr-next.8oT6i9/`. Canic remains unchanged; publication, consumer
adoption and combined-source native macOS qualification remain separate.
PRs #173, #174 and #183 were closed as superseded by their released 0.27.2
implementations and its complete successful required CI matrix.

The 0.27.3 working tree prepares
[#95 — Expose cancellable group capture](https://github.com/dragginzgame/ic-host-tooling/issues/95)
from PR #99 on the current child/fixture APIs, and
[#191 — Keep source-evidence adapters hash-format independent](https://github.com/dragginzgame/ic-host-tooling/issues/191)
from PR #192. The process engine passes 77 focused tool cases, eight group cases
on actual Rust 1.88, strict Clippy and warning-denied Rustdoc. Release/publication
fixtures and real local-Git source-evidence refusals pass on Linux. Receipt:
`/tmp/host-issue-work.VEZSmz/`. These are local changes over `8b45876`, preserving
incoming Platform and release-helper work; delivered/native qualification and
Build/Testkit adoption remain separate. No full local gate or release ran.

Use [host qualification](../hosts.md) for focused checks. Native macOS 15 Intel
and Apple Silicon execution is separate from Linux and substituted-host tests.
The architecture cleanup receipt is `/tmp/host-sediment.vSyJrI/`, relative to its
captured incoming dirty tree at Host `c8cd434`. The current tree includes earlier
FS, Text and logical-size work; this cleanup preserves that source and the index.
Validation for this cleanup is recorded in that receipt and the delivery handoff.

The earlier PR intake receipt is `/tmp/host-pr-issues.G75wqr/`, relative to the
captured incoming working tree at the same Host commit. That batch prepared
[PR #189 — Bounded Candid reply decoding](https://github.com/dragginzgame/ic-host-tooling/pull/189)
and [PR #174 — Scoped shared descriptor lock](https://github.com/dragginzgame/ic-host-tooling/pull/174),
alongside [PR #183 — Cache-directory tag handling](https://github.com/dragginzgame/ic-host-tooling/pull/183).
Decoder bytes/work and aggregate accounting remain explicit; domain admission
stays in Canic. The shared-lock guard explicitly unlocks before close; shared
record lifetimes and cache policy stay in Build. Focused Candid and descriptor-
lock tests passed on current Rust and actual Rust 1.88, with affected all-target/
all-feature MSRV checks, strict Clippy and warning-denied Rustdoc. These APIs
were local preparation at intake and are now recorded in released 0.27.2;
consumer/native qualification remains separate. PR branch CI applies only to its
own named head. The real index,
own versions, dependency selection and normal sibling checkouts are preserved.
Cache-tag handling retains the existing durable writer's ordinary parent creation,
which Build cache initialization requires; its original no-parent-creation claim
is corrected rather than changing that behavior. Native source qualification and
Build adoption remain separate from the focused producer cases.

Shared 0.24.0 was adopted at published main
`cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`. The snapshot now selects 117 payloads
from reviewed contribution `e0b6478a6f0a557b75caeb3e23e34cff6589a9fd` in
[Shared PR #283 — Share release metadata preparation and retained evidence custody](https://github.com/dragginzgame/shared-tooling/pull/283),
stacked on [Shared PR #279 — Share atomic release receipt mechanics](https://github.com/dragginzgame/shared-tooling/pull/279).
Neither PR is treated as a published release. Host keeps its validation receipt
format and selected-source/commit checks, with no receipt migration. Focused Shared
receipt/preparation fixtures and Host's release-adapter fixture pass on Linux,
including caught interruption and failed restoration with retained evidence.
The preparation helper does not promise multi-file atomicity or crash durability.
Testkit and Backup retain their existing adapters pending separate qualification.
Their working trees are currently changing those adapters and their Shared
baselines. Testkit additionally retains old/new payloads for automatic interrupted
installation recovery; its receipt adoption must preserve that contract rather
than replacing it with Host's caught-failure restoration model.
Shared's initial PR matrix failed in a pre-existing retention fixture that still
looked for an extracted inline setup step. The contribution now includes Shared's
prepared composite-action fixture repair; the focused retention suite passes on
Linux. The release helpers also guard initially empty inventories for Bash 3.2
strict mode. Their focused Linux fixtures pass; the refreshed hosted matrix and
native macOS execution remain separate qualification.
Receipt: `/tmp/host-release-custody.zslFZB/`; incoming Platform work is preserved.
The additional canonical direct-dependency checker enforces Host's Platform
boundary through the existing CI/release graph check. The 0.27.3 adoption starts
from released Host `8b45876`, preserving the incoming removal of the root parser
declaration. Receipt: `/tmp/host-platform-adoption.ue0cGD/`. Older qualification
paragraphs below and above apply to their named earlier batches.
Platform adoption passes all 29 Tools tests on Rust 1.99 and actual 1.88, the
default-feature tests, strict Clippy and warning-denied Rustdoc. Direct-boundary
negative fixtures reject renamed Candid edges in normal/dev/build/target tables.
Transitive graphs pass for Linux and both macOS triples; these are not native
macOS runs. Formatting, snapshot, declaration pins, ShellCheck and local links
pass. Only Platform 0.1.4 is added to the lock; existing upstream versions remain.
Package versions and the real index are unchanged. No full release gate ran.
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
