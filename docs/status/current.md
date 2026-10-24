# Current handoff

## Source and ownership

The committed baseline is `3c6aa1b4668f715e95283a1c8fd656eabb566e1e`, release
source 0.66.3. The current local audit repairs are uncommitted; package versions
remain unchanged. The incoming lockfile update selects Wire/derive 0.6.12 and
is preserved. A version/tag alone does not prove publication,
consumer adoption or native qualification.

Host, Build and Testkit retain the [extraction boundaries](../extraction.md).
Consumers own credentials, budgets, targets, retries, installation and recovery.
The reviewed Shared Tooling snapshot is 0.26.10 at `a8141e7`, recorded in
[the snapshot manifest](../../.shared-tooling.snapshot). Rust edition 2024,
MSRV 1.91 and Linux/native macOS 15 Intel and Apple Silicon remain required.

## Current local changes

The accepted whole-workspace audit batch is described in
[0.67.0 notes](../changelog/0.67.md). Explicit archive feature selection and new
public evidence fields require a minor release; this task does not bump versions,
commit, push or publish. Consumers of the archive API must pass feature selection. Configuration observation
and Wasm build construction now require caller-selected limits; benchmark comparison
returns a typed error for duplicate row identities.

Build now separates Cargo input resolution privately and exposes comparison-only
recipe identity on snapshots and build records. Source/features remain separately
attributable, and cache admission is unchanged. The producer request is
[#241](https://github.com/dragginzgame/ic-tooling/issues/241); Timers adoption must
use the published API and qualify its own cohort acceptance.

Browser sessions now reject unsuccessful fixture exit after readiness while
retaining independent callback/cleanup failures. Evidence and delivery belong to
[#242](https://github.com/dragginzgame/ic-tooling/issues/242). Testkit manifests
inherit workspace lints; public docs and narrow native-operation annotations
replace duplicated broad exceptions.

CI caches ordinary compilation directories only, preserving fresh archive
qualification targets. Superseded browser runs are cancelled. Publication scripts
are included in ShellCheck. Hosted cache performance has not been measured. Runtime evidence collection and
outer job timeouts are configured; isolated Tools/Text features run in development
and actual MSRV qualification. The repeated publication Clippy gate is retained:
the latest baseline Linux job spent 0.55 seconds on it.

Shared Tooling 0.26.10 adoption reuses the common action's Cargo-tool cache and
external IC dependency inventory, retaining stricter local ownership rules.
Adoption evidence is under `/tmp/ic-tooling-shared-02610/`: snapshot integrity,
actual dependency graphs, an injected renamed optional/dev IC dependency refusal,
formatting/hook and substituted Make/action checks pass on Linux. The aggregate
offline report admits Host/Cargo/release-editor tools; server admission remains
unprepared because `target/debug/ic-testkit-server` is absent. No downloads or
native macOS qualification were performed during this adoption.

## Qualification and blockers

Focused audit evidence is retained under `/tmp/ic-tooling-audit-fixes/`, including
failed intermediate checks. The original browser false-success reproduction is
`/tmp/ic-tooling-audit-browser-RwZzM9/`. The sandbox also failed unchanged Node
child-pipe tests; the focused seven-case browser suite passed outside it using the
installed pinned Node 24.21.0. Build's 268 library cases and focused strict Clippy
pass on Linux. Exact-archive feature tests, bounded diagnostics, seven pool cases,
four native memory cases, ShellCheck/workflow lint, formatting and doc links pass.
Actual Rust 1.91 caller checks, four recipe-related cases and exact-archive tests
pass; no whole-workspace test or release command was run.
Second-audit evidence is under `/tmp/ic-tooling-audit-2-fixes/`. It includes
configuration budget and include-topology regressions, benchmark duplicate refusal,
log-retention diagnostics and runtime evidence collection. All 19 benchmark cases
and 18 isolated feature cases pass on development Rust and actual Rust 1.91;
changed-package strict Clippy, MSRV caller compilation and Testkit Wasm checks
pass. Formatting, shell/workflow lint and local documentation links pass. Native macOS
qualification of these uncommitted changes has not run.

[Crypto qualification #240](https://github.com/dragginzgame/ic-tooling/issues/240)
remains blocked on the canonical correction in
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50). The selected
Platform 0.6.12 graph still contains crypto-bigint 0.5.5. A direct curve update
alone does not correct every Agent/derivation path, and actual ECDSA signing
prevents an unused-code exemption. No local crypto fork, override or exemption
was added. Source/configuration digest APIs do not qualify this dependency risk.

The delivered #239 input-boundary and hashing-phase repairs are included in the
baseline. Their original package failure remains at
`target/package-qualification-3180841-0/`; focused and fresh-archive evidence is
under `/tmp/ic-tooling-239/` and `/tmp/ic-tooling-package-phase/`. Do not remove
those artifacts or weaken source guards to make a later attempt pass.

## Recovery and historical evidence

Migration archives and local-repository retirement evidence remain under
`/tmp/ic-tooling-migration/`, including `retirement/`. Original refs were preserved
in the former Host repository. Stale release-lock recovery evidence remains at
`/tmp/ic-tooling-stale-release-lock.qHQFTn/lock`; retained plans are not disposable.

Earlier ownership, Wire and cycle-fixture qualification is described in
[the committed handoff](https://github.com/dragginzgame/ic-tooling/blob/3c6aa1b4668f715e95283a1c8fd656eabb566e1e/docs/status/current.md)
and the [release ledger](../../CHANGELOG.md). Historical Linux or macOS results
do not qualify the current dirty source. Inspect owning GitHub issues and exact
workflow revisions for delivery; this handoff is not a parallel issue tracker.
