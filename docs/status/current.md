# Current handoff

## Authoritative state

Read [Cargo.toml](../../Cargo.toml), [CHANGELOG.md](../../CHANGELOG.md), Git tags,
[AGENTS.md](../../AGENTS.md) and the exact [Shared snapshot](../../.shared-tooling.snapshot).
Published Host 0.28.0 uses `b9c20de084a8f8176d8c73c61a3b30e740c8875c`, including
the initial notice correction after the unchanged `v0.28.0` tag at `5150105`.
Pending work belongs to [0.28.1](../changelog/0.28.md#0281);
package versions and further release effects remain user-owned.
Sibling checkouts are read-only. GitHub issues are the sole follow-up tracker.

## Prepared full Wasm validation

Artifacts' opt-in `wasm-validation` feature adds full core-module validation over
the private existing wasmparser dependency. Structural inspection remains
independent. The API admits caller-selected input bytes before validation, checks
all function bodies and types, rejects components and returns Host diagnostics;
IC target/admission policy remains consumer-owned. Testkit's two currently staged
fixture-test Validator calls are the verified pending consumer; its source and
direct dependency remain unchanged until it adopts a published supplying Host version.

Focused Linux receipt: `/tmp/host-wasm-validation.ZH5CgD/`. Actual Rust 1.88 passes
default (48), inspection (64 plus six public contracts), validation (69 plus seven)
and all-feature (85 plus seven) Artifacts cases. Strict MSRV Clippy and Rustdoc
pass, including inspection-only Rustdoc. The lockfile and dependency versions are
unchanged. Native macOS execution and registry adoption of this new API remain
separate qualification requirements.

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

The snapshot selects reviewed Shared main
`86464be41588182be6c979070a26ea9dfa5aa566` (0.24.3 in its changelog; committed
`VERSION` and canonical snapshot label remain 0.24.2). Its canonical source helpers
coexist
with byte-identical release metadata/receipt custody, resolving
[#193's supplier prerequisite](https://github.com/dragginzgame/ic-host-tooling/issues/193).
Both local source-evidence adapters are removed and callers use Shared directly.
The former contribution PR remains open, but committed source and successful
Shared 0.24.2 Linux/macOS CI establish helper availability. Formatting/hooks and every release
manifest check use grouped sorting. No tool version or installed bundle changes.

The latest refresh adopts Shared's external IC dependency rule and extends the
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
