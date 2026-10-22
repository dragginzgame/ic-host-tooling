# Current handoff

## Bounded cycle-accepting fixtures for 0.64.2

Release main is 0.64.1 at `082aba308571bce742985f8ae7916e013ca9ec12`.
[#235 — Testkit: bounded cycle acceptance in controlled provider reply fixtures](https://github.com/dragginzgame/ic-tooling/issues/235)
is implemented locally with additive APIs under existing `held-reply-fixtures`:
`pic::fixtures::cycle_reply_canister` and
`HeldReplyFixture::new_with_cycle_acceptance`. They share the finite update-slot
generator; each slot accepts at most the explicit cap, including zero. Existing
constructors retain zero acceptance. No dependency or release version changed;
the incoming Wire 0.5.22 lock edit is preserved byte-for-byte.

Actual IC acceptance commits in the original update. A held barrier confirms
that commit; release returns the original bytes without accepting again. A later
timeout/rejection preserves earlier accepted cycles. Traps roll back uncommitted
effects. Unknown/exhausted invocations accept nothing. Product reply semantics,
authorization, journal correlation, funding and recovery remain consumer-owned.

On Linux, development Rust 1.99.1 and actual Rust 1.91 pass six focused fixture
admission/full-validation cases and five installed cases. New proofs cover exact
zero/partial/full acceptance, callback refunds and both balances on a fee-free
system subnet; normal application-subnet refunds; raw success/error/malformed
bytes; exhaustion; held out-of-order release and timeout after acceptance. Existing
held ordering/token cases remain passing. Strict all-target Testkit Clippy,
warning-denied Rustdoc and native/deployed feature/graph checks pass. Focused
evidence is retained under `/tmp/ic-tooling-235`. No fresh native macOS, consumer
funding-journal or live-provider acceptance is claimed. Existing all-feature
native CI owns the new target on all three hosts after delivery. No commits,
pushes, publication, release or sibling edits ran; 0.64.2 notes are prepared.

## Qualification fixture repair

The maintainer committed held-reply preparation at `b6cc3f09` (manifest still
0.64.0). Release verification stopped in the local Make snapshot fixture:
its expected MSRV command still selected `reply-fixtures`, while the real target
now selects `held-reply-fixtures`. The local repair aligns that expectation and
replays nested relative-fixture logs without replacing their failure status.
No Rust, versions, release state or incoming artifacts were changed.
Original failure evidence remains at `/tmp/host-fixture-retention.6LzebK`;
focused repair evidence is retained at `/tmp/ic-tooling-fixture-fix`.
Linux `make tooling-command-check`, Bash syntax, ShellCheck and local documentation
links pass. A controlled nested failure proves diagnostic replay, original exit
code 23 and preservation of both evidence directories. No full release rerun or
fresh native macOS qualification is claimed.

## Held provider replies for 0.64.1

Main is release 0.64.0, `036da0cdf3d3b35297fbc26031e373f386a2f2de`.
Its [native CI](https://github.com/dragginzgame/ic-tooling/actions/runs/39753592846)
and [Testkit runtime](https://github.com/dragginzgame/ic-tooling/actions/runs/39753593052)
passed Linux x86-64 and, independently, native macOS 15 Intel and Apple Silicon.
Those results qualify released source, not the new local fixture below.

[#234 — Testkit: support controllable held replies for installed callback-race tests](https://github.com/dragginzgame/ic-tooling/issues/234)
is implemented locally with native opt-in `held-reply-fixtures`. Single-use
bounded update slots retain immutable original replies and real call contexts
behind PocketIC manual HTTP barriers. Exact tokens bind fixture, request, subnet,
server and instance. Release can be out of order; unknown/repeated/foreign tokens,
slot reuse and automatic progress refuse. Timeout rejects without releasing the
original bytes. Installation, funding, manual progress and teardown stay caller-owned.
See [usage/lifecycle](../../crates/ic-testkit/README.md#held-provider-replies).

Linux evidence is retained at `/tmp/ic-tooling-234`: focused fixture admission
and full Wasm validation, real installed inter-canister ordering/identity and
timeout/foreign-token proofs, strict Clippy, Rustdoc and deployed graph checks.
The same focused cases run on actual Rust 1.91; native macOS qualification of
this new source remains outstanding. Existing all-feature native CI owns it.
No release, commit, push or sibling edit is part of this change. The incoming
Wire lock edits remain preserved; the task adds Platform's external management
type selection without upgrading existing selections. Native management Candid
is required by this barrier; no application codec or deployed dependency is added.

Canic adoption and both mounted callback-race cases remain with its owning issue,
linked from #234. #233 now tracks consumer migration only; #225 still needs the
concrete contract from Canic #613. The older preparation records below describe
prior local evidence; the released source/qualification above supersedes their
then-pending delivery and host status.

## Shared Tooling 0.26.6 adoption

The maintainer committed the 0.64 preparation at `41b1dadb` before this refresh;
the checkout was clean. The canonical exporter now records Shared Tooling 0.26.6
at `4cb9bc907154225a3f7e4c33fd8d75fbeaa125c8`, retaining the previous roots and
adding `audits/ci-release-integrity.md`. All 137 payloads verify against the new
manifest. Host/Rust/IC pins, package versions and Cargo.lock are unchanged.
The snapshot, local baseline pointer and adoption notes shipped in 0.64.0.

Focused Linux checks pass for shell syntax/ShellCheck, local documentation links,
real overlapping Testkit/probe feature graphs on development Cargo and Cargo
1.91, and controlled consumer failure/evidence paths. The latter exercise NUL
checksum refusal, original backend statuses, independent parent/nested errors,
redirected latest-log paths and summary failure without masking target status.
The first temporary proof attempt hit an assertion glob that also selected its
deliberately occupied directory; the corrected proof passes and both attempts
remain at `/tmp/ic-tooling-refresh-0266`. The local tooling-command gate also
passes product-evidence round trips, Host fixture retention, actual Make snapshot
routing and substituted release-command admission. No product Rust changed.

Shared's [source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/39747129095)
passes lint/security, change selection, Linux and native macOS Apple Silicon
portable regression. Native macOS 15 Intel is still running at the closing
inspection; the full workflow remains pending and is not claimed as passing.
No fresh native macOS consumer qualification is claimed.

## 0.64 workspace hard-cut preparation evidence

The maintainer expanded the cleanup review to the whole workspace. Released
[0.64 migration notes](../changelog/0.64.md) cover dependency/helper ownership
under [#233 — Remove duplicate dependency facades and policy types for 0.64](https://github.com/dragginzgame/ic-tooling/issues/233)
and decoder admission under
[#232 — Delegate Testkit Wire response admission to the Wire decoder](https://github.com/dragginzgame/ic-tooling/issues/232).
Testkit no longer forwards Platform's PocketIC surface or uses stateless wrapper
types for principals/counters. Diagnostics use Host's existing batch-label owner;
Host Tools uses Platform's existing Candid limits directly. Generic provisioning
path checks delegate to Host FS. Real local callers/examples are migrated, and
tests of the removed forwarding contract are retired. Build already uses Host's
batch validator; its cache/recovery contracts remain intact. Consumer import and
call changes belong to the migration notes and owning issue; sibling checkouts
remain read-only.

All 30 focused Candid, diagnostics, provisioning, Wire and principal cases pass
on development Rust 1.99.1 and actual Rust 1.91. Native Testkit/private-harness
all-target/all-feature checks and library/probe Wasm checks pass on both compilers.
Strict Clippy passes for the affected native packages and Wasm probe; all five
Testkit doctests and warning-free Testkit/Host Tools Rustdoc pass on Linux.
The maintainer committed these changes at `41b1dadb`; package versions remain
0.63.3. Evidence: `/tmp/ic-tooling-064-api`; the first Wire-only pass
is retained at `/tmp/ic-tooling-wire-cleanup`. No new native macOS or live server
integration qualification is claimed for this working tree.
An additional incoming lock edit selected Tokio 1.54.1 during review. Affected
checks were refreshed against it; prior logs and both lock selections remain
in the same evidence directory. No dependency update was made by this task.

## Delivered 0.63.3 cleanup

Release 0.63.3 narrows six single-feature Host Tools targets to their own
tests while retaining isolated all-target compilation. Default/combined suites
remain complete. The workspace publication adapter retains its failure/recovery
checks; the separate generic Shared publication-runner replay is retired here.
All six focused targets (30 cases), publication and Make fixtures, shell lint,
snapshot integrity and documentation checks passed on Linux. The delivered
source subsequently passed native qualification below. Focused evidence:
`/tmp/ic-tooling-cleanup-0633`.
The second pass replaces Build's duplicate extractor/CLI fixture cleanup with
Host FS's tested guard. Eight extractor cases, five actual CLI cases and strict
all-target Build Clippy pass on Linux. Private lockfile field helpers remain
shared by archive admission and Wasm cache projection. Second-pass evidence:
`/tmp/ic-tooling-cleanup-pass2`.
The third pass derives the private benchmark worker's Clap schema and mode
admission. Help output is byte-identical; nine worker/driver argument cases pass
on development Rust and actual Rust 1.91, with strict all-target Clippy. Evidence:
`/tmp/ic-tooling-cleanup-pass3`. Incoming lock selections remain unchanged.
Testkit's server asset/runner and benchmark-driver fixtures now use the same
guard; the single-caller integration directory helper is removed. All 34 affected
cases, strict Clippy, Rust 1.91 compilation and native/Wasm graph checks pass on
Linux. Installed-server cases remain explicit. Evidence:
`/tmp/ic-tooling-cleanup-pass4`; delivered native qualification is linked below.
The dependency/production audit found no unused direct dependencies with
cargo-machete. Eighteen redundant Testkit lint entries were removed after
confirming their enabled group membership on both compilers; strict Clippy
passes. [Issue #231](https://github.com/dragginzgame/ic-tooling/issues/231) shipped
a private capture composition that returns the durable copy's digest
to Wasm evidence export. The second byte traversal is removed; staged-file
readability admission and freshness/retention ordering remain. All 20 affected
cases pass on development Rust and Rust 1.91, with strict Clippy/Rustdoc. No
timing claim is made. Evidence: `/tmp/ic-tooling-231`; native CI is linked below.
Evidence: `/tmp/ic-tooling-cleanup-audit5`.

## Integrated workspace

Delivered main is release 0.64.0, `036da0cdf3d3b35297fbc26031e373f386a2f2de`,
tagged `v0.64.0`. The committed lockfile selects Wire 0.5.20 and PocketIC 16.1.0;
Testkit's provisioned server pin is also 16.1.0. The earlier release-preparation
blockers are superseded. The working-tree lockfile has a separate incoming edit.
Host, Build and Testkit were consolidated through
[PR #212](https://github.com/dragginzgame/ic-tooling/pull/212), retaining crate
boundaries and imported histories. Ten packages are public and two are private.
Rust edition 2024 and MSRV 1.91 remain approved. See the
[consolidation design](../design/tooling-consolidation.md) and
[0.63 migration notes](../changelog/0.63.md).

Delivered 0.63 source removes Testkit-owned Candid and Metrics coupling,
uses Platform 0.6 and makes Build/Testkit binaries opt in through `cli`.
Native PocketIC still brings upstream Candid; Testkit's Wasm graph excludes it.
The optional HTTP crate and native CI setup regression are committed at local HEAD.
Publication of the observer and HTTP packages is verified below; consumer
adoption and complete native qualification remain distinct.

## Published interfaces and consumer ownership

[#117](https://github.com/dragginzgame/ic-tooling/issues/117): the HTTP reader now
also accepts caller-owned storage so an outer timeout preserves accepted bytes.
Returned errors move retained evidence into the existing error; cancellation
leaves the prefix for the caller to retain or discard. Status, requests, timeout
and redaction remain caller-owned. Query and Blob adoption have their own issues:
[Query #93](https://github.com/dragginzgame/ic-query/issues/93) and
[Blob #96](https://github.com/dragginzgame/ic-blob-storage/issues/96).
Blob's streaming provider download must remain a verifier/file stream.

[#227](https://github.com/dragginzgame/ic-tooling/issues/227): published opt-in
`ic-testkit-native/stable-memory` adapts preserved Testkit PR #97 source
`3cd17999a53689a035dc86b6f01155eca78c7140` through Platform. The already-selected
registry Platform 0.6.2 supplies Memory and VectorMemory; the manifest floor now
requires that supplying version. Default native and all Wasm graphs remain
independent of the substrate. Clones share backing/counters, resets preserve
bytes, and overflow is sticky and must refuse exact reports. Metrics stays out
of Tooling. Adoption belongs to
[Persistence #98](https://github.com/dragginzgame/ic-persistence/issues/98) and
[Jobs #56](https://github.com/dragginzgame/ic-jobs/issues/56).
Metrics #82 was closed as not planned and is not an acceptance requirement.

Both non-yanked 0.63.2 archives were downloaded and authenticated against the
canonical registry index; their Rust sources match the release checkout.
Actual Rust 1.91 execution from the published archives passes six native
observer/allocation tests, the observer Wasm check and eight HTTP loopback tests.
Evidence and archive hashes are recorded in #227/#117 and retained under
`/tmp/ic-tooling-registry-0632`. Earlier focused Clippy, Rustdoc and dependency
graph evidence remains valid for the matching source.

## Existing contributions and qualification

Local Shared snapshot is now 0.26.6 at `4cb9bc907154225a3f7e4c33fd8d75fbeaa125c8`,
with adoption evidence above. Earlier optimizer-only routing from
[PR #226](https://github.com/dragginzgame/ic-tooling/pull/226) shipped without
duplicating its native setup regression. Refresh evidence and incoming dirty-file
backups are under `/tmp/ic-tooling-refresh-0262`.
[PR #229](https://github.com/dragginzgame/ic-tooling/pull/229) merged the ICP
decoder at `5b41e182`. PR #226 merged at `3e61a613` with retained release notes.
Earlier integration evidence is under `/tmp/ic-tooling-pr-integration`.

Verified 2026-10-22 on exact release commit `966dbfed`:
[main CI](https://github.com/dragginzgame/ic-tooling/actions/runs/39637698526)
passed lint/security, MSRV and native Linux x86-64. Native macOS 15 Intel and
Apple Silicon also passed independently on their native runners.
[Testkit runtime](https://github.com/dragginzgame/ic-tooling/actions/runs/39637698688),
[Build optimizer](https://github.com/dragginzgame/ic-tooling/actions/runs/39637698491)
and [browser session](https://github.com/dragginzgame/ic-tooling/actions/runs/39637698443)
each passed Linux and both macOS hosts. These results qualify the delivered
0.63.3 source, not the prepared 0.64 source or Shared refresh.
The earlier 0.63.2 Intel DNS failure was recovered under closed
[issue #230](https://github.com/dragginzgame/ic-tooling/issues/230);
original failure logs remain at `/tmp/ic-tooling-0632-intel.log`.

Ordinary integration CI follows the adopted baseline's
[asynchronous cadence](../../DRAGGINZGAME.md#validation-and-evidence);
the former local exception is no longer needed.
Required merge/release/issue acceptance gates remain separate. There are no
open Tooling PRs at this review. [Issue #225](https://github.com/dragginzgame/ic-tooling/issues/225)
still depends on Canic #613 supplying the concrete Wire schema/artifact contract.

## Preserved evidence

Original HTTP evidence: `/tmp/ic-tooling-http`; native setup regression:
`/tmp/ic-tooling-0632`. Earlier implementation evidence remains under
`/tmp/ic-tooling-adoption`, `/tmp/ic-tooling-cleanup`, `/tmp/ic-tooling-candid`,
`/tmp/ic-tooling-platform-06`, `/tmp/ic-tooling-no-metrics` and `/tmp/ic-tooling-cli`.
Migration backups, full checkout archives and retirement evidence remain under
`/tmp/ic-tooling-migration`, including `retirement/`. Original proposal objects
remain readable in the ordinary former Host checkout; siblings are read-only.
The verified stale release lock was moved to
`/tmp/ic-tooling-stale-release-lock.qHQFTn/lock`; saved plans were preserved.
