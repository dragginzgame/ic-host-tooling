# Current handoff

## Optional filesystem backend boundary — pending 0.30.0

Over released `ef1e632`, FS's existing Unix capability and scoped-tempfile APIs
now require independent `capability` and `scoped-tempfile` features. Minimal FS,
file CLI and Process graphs exclude both private backend trees. Their safety,
permission, resource and lifetime implementations remain unchanged. Query's
confined reader must enable the first feature; IcyDB's named argument file guard
must enable the second when adopting a supplying published Host release.
The graph matrix and native CI qualify core, each backend and combined selection.
This contribution leaves package versions, sibling checkouts and releases alone.

Focused Linux evidence: actual Rust 1.88 passes 177 minimal FS, twelve capability,
six scoped-tempfile, 147 Process and eight file-CLI cases. Strict affected-package
all-target/all-feature Clippy, Rust 1.88 checks, warning-denied FS Rustdoc and the
all-feature unsupported-Wasm compile pass. Minimal/independent/combined/CLI/Process
graph checks pass for Linux and both macOS target selections. Source-following
ShellCheck, Actionlint, formatting and local links pass. A sandbox-only loopback
EPERM affected the first Process run; the native-permission rerun passes.

An isolated Build 0.21 source qualifies this exact Host source via three temporary
path overrides, retaining own package versions and other selected identities.
Its Linux normal/build graph drops from 66 to 51 packages; all 156 focused
Build cases and actual Rust 1.91 all-target/all-feature checking pass. Those
overrides are qualification-only and are not delivered to consumers. Native
macOS execution and complete release gates remain hosted/maintainer obligations.
Evidence: `/tmp/ic-build-fleet-followup.KGzFhf/`.

## Authoritative state

Read [Cargo.toml](../../Cargo.toml), [CHANGELOG.md](../../CHANGELOG.md), Git tags,
[AGENTS.md](../../AGENTS.md) and the exact [Shared snapshot](../../.shared-tooling.snapshot).
Host 0.28.1 is released at `a4c2bf48d520391cfd326f98ea9fda864241a1a5`
and is available from the registry. That source still uses Ciborium: the producer
changes described in its notes were not integrated before the release. The next
draft is [0.29.0](../changelog/0.29.md#0290), selected for the explicit certificate
depth-contract change below. Staging, commits, version changes,
Git release/push and registry publication remain user-owned.
Sibling checkouts are read-only. GitHub issues are the sole follow-up tracker.

## Shared Tooling 0.25.1 adoption

The snapshot now selects committed main
`d57c631206e3618755e45bebf2c88c673564ddc1` (0.25.1), verified against GitHub
main. The sibling started unrelated edits during review; export used a clean
isolated checkout of that exact commit. No sibling edits were copied or changed.
The 125-file snapshot adds canonical graph, symlink and license admission and
retires the two Perl Cargo renderers. Host's graph matrix delegates package
observation/counting while keeping its targets, profiles and enabled-feature checks.

Host release candidate rendering now uses released IC Build 0.18.0's admitted
editor and the positive 8-MiB per-document allowance in `ci/release-editor.env`.
The optional release tool is prepared through Shared's verified store and local
setup extensions, with offline checks and a pin-sensitive CI cache key. Source
custody, exact receipts and metadata transaction/rollback remain Host/Shared-owned.
First-use preflight prepares a missing editor after source admission; verification
and saved-plan recovery never install. Saved-plan preflight admits dependencies
offline. Its Rust 1.91 tool floor does not raise Host's 1.88 MSRV.

Maintained binaries and argument-taking examples use Clap. FS's `ic-host-file`
requires the explicit `cli` feature so normal library graphs remain Clap-free.
Report/operation bytes and refusal exit code 1 remain; Clap owns typed arguments,
help, version and diagnostics. The [0.29 notes](../changelog/0.29.md) document the
installation change. Required Candid helpers still target external Candid/DID
interoperability through Platform; Host owns no canister RPC to migrate to Wire.

Receipt: `/tmp/host-shared-025.a3okEz/`. Actual Rust 1.88 passes both CLI suites
and compiles every changed example. Strict Clippy passes MSRV/development Rust
for affected packages, and the actual MSRV non-Unix compile gate passes. The
inspector's existing oversized integration test was split at the argument-refusal
boundary to satisfy its pre-existing MSRV Clippy complexity failure, preserving
coverage. Real IC Build rendering/rollback/first-use/offline adapter fixtures and
publication/license rejection fixtures pass with Git/upload/setup substituted.
Make routing covers the local editor extension; Linux/both macOS graph profiles
pass. The 125-file snapshot, source-symlink checks, grouped formatting, real hook
preservation/installation, pins, local links, ShellCheck and retained-fixture
contract checks pass. Package selections/checksums are preserved; only five Clap-family packages
are added. No full local workspace gate, Git/release or real publication ran.
Shared 0.25.0's CI was superseded; 0.25.1's exact-revision CI was still running
at review. Local Linux and graph
checks do not qualify native macOS. Prior source/artifacts and the index are retained.

## Consolidated filesystem and Wasm tooling

The approved capability API from PR #203 and scoped-tempfile API from PR #205 are
now present in this working checkout, preserving the existing backends and generic
regression coverage. FS privately owns Unix cap-std/cap-fs-ext and tempfile;
non-Unix graphs exclude them. `read::open_regular_file_no_follow` shares the
existing read-only opener and returns an admitted standard file without consuming
bytes, supporting consumer reference synchronization and lock inspection.

Artifacts adds independent `wasm-text` through wat 1.262.0 with components disabled,
owned limits/errors and source/output-byte admission. Compilation remains distinct
from full validation. Text-only production graphs exclude wasmparser; defaults,
inspection and encoding-only graphs exclude wat/wast. The lock retains an optional
encoder/parser integration selection without compiling it in text-only profiles.
All existing selected dependency versions/checksums are preserved. Direct backend
declarations are now checked against their owning Host crate in every dependency
kind, alias and target, alongside the existing Platform rules.

Receipt: `/tmp/host-boundary-cleanup.70rMhG/`. Actual Rust 1.88 passes 52
text-only Artifacts cases, 95 all-feature unit plus eight integration cases,
195 FS unit plus 12 integration cases and both isolated signal-action cases.
Strict Clippy passes on MSRV/development Rust for the changed packages; strict
Rustdoc passes all features and text-only. Formatting, pins, links, ShellCheck and
Linux/both macOS normal/build graph profiles pass. The actual 1.88 non-Unix FS
library/binary/unsupported-lock compile gate passes. This does not qualify native
macOS behavior. No full local gate, release, index/HEAD change or sibling edit ran.

The [0.29 notes](../changelog/0.29.md) describe Query capability/discovery and
Testkit encoder/validator adoption, Backup's WAT calls, Query's replacement of
generated WAT with Testkit fixtures, and consumers of the existing signal guard
and no-follow opener. These remain consumer-owned migrations after the supplying
release. IcyDB already uses caller-retained scratch files without tempfile.
Testkit temporary directories/unnamed captures and Canic cooperative signal-hook
callbacks keep their distinct custody/registration contracts. No new signal facade
or broad tempfile replacement was introduced. The earlier qualification receipts
below describe their own preceding source, rather than this expanded batch.

## Private Wasm encoder boundary

Artifacts now prepares independent `wasm-encoding` with owned call-module inputs
and a private wasm-encoder 0.262.0 backend. It supports the fixed-memory,
i32/void import-call mechanics demonstrated by Testkit's raw-reply fixtures;
Testkit retains all IC names, reply/layout budgets, error mapping and lifecycle.
That sibling remains read-only (inspected HEAD
`76f20411a368a230a282f7c445026f009b31fe4c`, with its existing dirty lockfile).
The full validator already prepared in PR #204 is also present locally through
`wasm-validation`; released 0.28.1 source did not integrate that earlier proposal.
The next draft remains 0.29.0, with both codec and Wasm adoption instructions.

Receipt: `/tmp/host-wasm-encoding.F8Zi5R/`. Actual Rust 1.88 passes default,
inspection-only, validation-only, encoding-only and encoding/validation tests;
the all-feature profile passes 90 unit and eight integration cases. Strict Clippy
passes on MSRV and development Rust; strict Rustdoc passes for encoding,
inspection and combined validation. Formatting, dependency pins, links,
ShellCheck and Linux/both macOS target graphs pass. Encoding-only normal/build
graphs exclude the parser; inspection-only graphs exclude encoder and validator.
Only wasm-encoder and leb128fmt are added; existing package selections/checksums
are unchanged. The parser remains 0.261.0. Native macOS execution of this source
is not qualified by these graph checks. No full local gate or release ran.
Testkit adoption requires the supplying registry release and its focused fixture
checks before removing its direct encoder/parser declarations. No sibling changed.

## IC Agent codec alignment

The maintainer selected serde_cbor, matching IC Agent, instead of the unmerged
Minicbor proposal in PRs #206/#207. Tools retains strict schema and byte admission
but removes `certificate::DecodeLimits.depth`; recursion and semantic-tag
handling belong to the shared codec. Query is the verified consumer of that
field; its source and lock are unchanged. The private parser error source becomes
serde_cbor. Ciborium and its helpers plus unused half 2.x leave the lockfile;
the existing upstream ledger Minicbor dependency remains separate.

Receipt: `/tmp/host-serde-cbor.bh63Mr/`. The selected SDK implementation is
IC Agent 0.49.2 with serde_cbor 0.11.2. Actual Rust 1.88 passes all 24 Tools
certificate-profile tests, all 52 Tools tests and default/interface/parser-free
combined profiles. Strict Clippy passes on MSRV and development Rust; Rustdoc,
formatting, dependency pins, links, ShellCheck and Linux/both macOS target graphs
pass. Existing external versions/checksums remain selected; only four obsolete
packages leave. Native macOS execution of this source remains separate; no full
local workspace gate, release or publication ran.
The replacement is prepared for decoder PR #206 and combined producer PR #207;
their superseded Minicbor heads are not qualification of this new contract.

## Prepared Candid isolation

Tools Candid helpers require the explicit `candid` feature. Empty defaults and
certificate-only, interface-only and combined certificate/interface normal/build
graphs exclude both Candid and its parser. Independent interface projection tests
remain enabled; only the DID comparison requires `candid`. Agent and ledger
evidence select their required codec without the parser, while Registry selects
Platform principal types alone. The prepared duration and MSRV CI fixes remain
in this minor batch. Query owns its separate lockfile refresh and certificate-host
graph verification after registry publication.

## Ownership

The [extraction contract](../extraction.md) owns the five library boundaries.
Artifacts owns bounded bytes, identities, checksum framing, batch admission and
optional gzip/Wasm facts. FS owns path/descriptor admission, traversal, allocation,
publication and explicitly selected cleanup custody. Process owns execution,
child/descriptor custody and native observations. Text owns pure normalization
and rendering. Tools owns IC format interpretation and its aggregate reply adapter.
The [Wasm inspector](../../apps/ic-host-wasm-inspect/README.md) composes Artifacts
and FS and owns its arguments/output; it performs no build or tool installation.

Platform 0.1.11 owns the upstream Candid/parser/certification dependency boundary;
Host retains service projection, aggregate debit ordering and strict CBOR admission.
Shared owns common development, CI and release mechanics. Host retains package
eligibility, metadata projection, real integration checks and recovery decisions.
IC Build owns native Candid extraction and build/cache orchestration; Testkit owns
PocketIC lifecycle. Consumers retain schemas, tool pins, budgets, credentials,
targets, retries, authentication and paid-operation recovery.

## Current qualification

Released 0.27.4 passed Linux x86-64, macOS 15 Intel/Apple Silicon and MSRV in
[its exact-source CI run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/39125028075).
Consumer adoption remains distinct from that producer qualification.

Delivered 0.27.5 preparation has the following focused Linux evidence. Its
exact-source native CI was still running when the Canic issue pass began:

- [#194 — Lock platform guards](https://github.com/dragginzgame/ic-host-tooling/issues/194):
  18 lock cases pass on current Rust and actual 1.88, with directory and fixture
  cases, strict native Clippy and Rustdoc. The library/test-support/non-Unix test
  compiles on actual 1.88 for Wasm; this is not Wasm test or filesystem execution.
  Optional Wasm Clippy reports target-specific uninhabited-File/const lints outside
  the compile-only gate. Receipt: `/tmp/host-lock-cfg.5k9tgx/`.
- [PR #135 — Bounded traversal and directory allocation](https://github.com/dragginzgame/ic-host-tooling/pull/135):
  61 artifact, 15 directory/custody and five fixture cases pass on current Rust
  and actual 1.88, plus seven integration cases, strict Clippy/Rustdoc and the
  non-Unix compile gate. The earlier Build rehearsal qualifies only its named
  source; [Build #45](https://github.com/dragginzgame/ic-build/issues/45) owns adoption.
  Receipt: `/tmp/host-build-mechanics.8823nY/`.
- Platform 0.1.11: all 29 Tools cases pass on current Rust and actual 1.88, with
  strict Clippy, boundary/feature graphs, pins, links and changelog admission.
  Codec/parser/certification features are unchanged. Receipt:
  `/tmp/host-platform-011.q74eWi/`. Platform's own CI was still running at review.

Use [the host guide](../hosts.md) and the
[shared validation policy](../../DRAGGINZGAME.md#validation-and-evidence).
Focused passing evidence does not claim a full local release gate, publication,
deployment or native macOS execution of pending bytes.

## Retained integration boundaries

PR #162's duration implementation and six tests are reconciled unchanged with
Host 0.27.6 for pending 0.27.7. All 34 Text tests and both duration doctests pass
on actual Rust 1.88; current Text tests, strict Clippy/Rustdoc, Rustfmt, links and
feature graphs pass. Both truncating and rounded displays remain explicit.
Receipt: `/tmp/host-duration-pr.BQDRsO/`. Consumer adoption stays with Query #71
and Canic #458. Native qualification of the refreshed PR remains separate; the
current release's non-Unix CI toolchain mismatch is tracked under Host #194.

The 0.27.6 MSRV job passed native compilation but its non-Unix step selected the
1.99 development pin after installing the Wasm target only for 1.88, causing E0463.
The prepared fix scopes that step to `RUSTUP_TOOLCHAIN=1.88.0`; no target download,
compiler upgrade or product portability change is involved. Host #194 retains
source/native qualification; this corrects CI invocation ownership.

The Canic producer pass brought #97's reviewed ICP command/status implementation
and four tests from PR #101 (`38b114bb30f12e9a6e255ccd85d6b7344331d763`) into
now-delivered 0.27.6. All 33 Tools cases pass on current Rust and actual 1.88, including
the four ICP cases separately on the minimal profile. Strict Tools Clippy/Rustdoc,
Rustfmt, feature boundaries, pins, snapshot and links pass. No dependency, feature,
manifest, lockfile or sibling source changed. Receipt:
`/tmp/host-canic-issues.t1JUg3/`. Integration/publication and native qualification
of this new source remain separate; Canic #458 and IcyDB #307 own adoption.

The preceding adoption selected reviewed Shared main
`86464be41588182be6c979070a26ea9dfa5aa566` (0.24.3 in its changelog; committed
`VERSION` and canonical snapshot label remain 0.24.2). Its canonical source helpers
coexist
with byte-identical release metadata/receipt custody, resolving
[#193's supplier prerequisite](https://github.com/dragginzgame/ic-host-tooling/issues/193).
Both local source-evidence adapters are removed and callers use Shared directly.
The former contribution PR remains open, but committed source and successful
Shared 0.24.2 Linux/macOS CI establish helper availability. Formatting/hooks and every release
manifest check use grouped sorting. No tool version or installed bundle changes.

That preceding refresh adopted Shared's external IC dependency rule and extended the
direct-dependency guard to known Agent, Principal, PocketIC, system, signature,
ledger/management and storage packages. The finite name policy still needs review
for new IC package identities and generated imports. Snapshot, pins, Linux/macOS
feature graphs, links and changelog admission pass; 30 substituted aliased,
optional/platform normal/dev/build cases reject while a Serde control passes.
These graphs are not native macOS execution. Shared's exact-revision CI was still
running at adoption. Receipt: `/tmp/host-shared-adoption.4eu0Td/`. The snapshot's
executable payloads, tool pins, Host Rust source, manifests and lockfile are
unchanged by this refresh; Host already uses the shared metadata transaction.

The Platform aggregate adapter retains Host's published limits/errors and charges
admitted malformed replies before decoding; it is not a competing decoder.
The FS gzip example is an intentional FS/Artifacts integration caller, which
explains the dev-only gzip feature. Private fixed-size reads retain stack storage
and permission/link admission. Read-only and ordered-sync tree visitors share a
checksum collector while preserving different effects. These remain supported.

## Audit evidence and history

The architectural cleanup uses the pinned Shared flow-convergence/module-surface
methods against `bcc37e7` plus the captured incoming 0.27.5 work. Receipt:
`/tmp/host-sediment-final.kVbk8M/`. Incoming source, artifacts, manifests' release
versions, lock selection and index are preserved; changes cover catalog cleanup,
canonical source-evidence adoption and matching documentation/formatting checks.
Actual consumers were inspected in available Build, Backup, Query, Testkit and
Platform siblings; their mutable checkouts are evidence, not published adoption.
The member/dependency metadata is byte-identical before and after cleanup; the
lockfile is unchanged. Selected Process/Text/Tools builds, direct/feature boundary
checks, pins and the 119-file snapshot pass. Real Host release/publication fixtures,
Make routing, formatting/hook preservation, ShellCheck, links and changelog
admission pass with substituted Git/upload/setup effects. No release or native
macOS execution of pending Host bytes ran. Completed 0.27.4 producer issues #138/
#140 and superseded PRs #139/#142/#147/#184 were closed after direct remote source,
tag and full native/MSRV CI verification; publication/adoption remain separate.

The [complete preceding handoff](archive-2026-10-18-02.md) retains every prior
receipt and source identity verbatim, including completed 0.27.3/0.27.4 preparation
and earlier qualification observations. It links the older archives. Historical
0.28 proposals and earlier adoption-pending claims do not change current contracts.
