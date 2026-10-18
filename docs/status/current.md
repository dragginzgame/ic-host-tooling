# Current handoff

## Artifact dependency feature cleanup — pending 0.17.5

Build stops enabling Host Artifacts' unused `wasm` parser feature. Its source,
tests and examples use the base byte/artifact APIs; structural inspection remains
a consumer-selected Host capability. The usage guide now requires inspecting
consumers to enable that feature explicitly. Cargo/Wasm production and caching
retain their behavior, and no public API changes.

The refreshed lockfile removes only wasmparser and leb128; every retained package
version, source and checksum is unchanged. Host Artifacts resolves with no enabled
features. Linux Rust 1.91 all-target/all-feature checking, seventeen Cargo/Wasm
integration cases, three Host artifact integration cases, strict development
Clippy, formatting, pins, direct boundaries and documentation links pass.
Full release CI, native macOS and installed-IC qualification remain unrun.
Incoming Rustix/probe work is preserved; evidence and before-images are under
`/tmp/ic-build-wasm-feature-OEpr0h/`.

## Unix dependency consolidation — pending 0.17.5

Starting from the pending input-probe cleanup below, Build replaces its remaining
direct libc calls with Rustix effective-UID, descriptor-flag and signal APIs.
The Unix Rustix dependency explicitly enables `process`; direct libc declarations
and the root package's lockfile edge are removed. Selected package versions and
transitive libc remain unchanged. Existing owner admission, close-on-exec and
exact TERM/KILL assertions retain their behavior; no public API changes.

Linux Rust 1.91 all-target/all-feature checking and six focused output-reuse,
descriptor and process-failure/cancellation cases pass. Strict development Clippy,
formatting, dependency pins and direct boundaries pass. Native macOS and the full
release gate remain unrun. Before-images and logs are retained under
`/tmp/ic-build-rustix-NzKGeG/`; incoming probe changes remain separate.

## Input-probe process ownership cleanup — pending 0.17.5

Starting source is released `c0f23c3` (0.17.4), with a clean working tree and
Host 0.27.4 selected. This pass reviews implementation, public entry points,
manifests, test fixtures, examples, release/CI adapters and current documentation.
Available Host, Query, Canic, Testkit, IcyDB, Jobs and Wire checkouts are read-only
evidence; no sibling integration or native qualification is claimed.

Build's bounded input probes still implemented cancellation/deadline refusal,
process-group spawn and capture setup after their canonical owner became
available in published Host Process 0.27.3. Build now calls Host's cancellable
group capture directly and requires at least that release. Cargo.lock remains
unchanged at 0.27.4. Build retains phase selection, complete-output bounds and
typed error mapping; the caller retains the absolute deadline, cancellation
predicate and cleanup policy. There is no default compilation deadline or public
API change. Query's governance-smoke `resolve_probe_inputs` is a verified caller;
its existing policy requires no migration.

Input traversal and semantic hashing retain different admission/framing contracts
from Host's byte hashing and package traversal. Invocation staging and retained
cache transactions retain distinct lifetimes, while native and Wasm protocol
refusal intentionally use different cleanup/draining policies. Public batch-label
errors are supported contracts; replacing their identity is a separate breaking
decision, not required for this cleanup. Historical cache namespaces and tracked
cache metadata are preserved, with package exclusion and ignore rules retained.
Shared payload copies remain pinned dependencies, and local release adapters
already delegate mechanics to Shared Tooling. Unpublished sibling APIs are not
used as replacements.

Linux Rust 1.91 passes six focused probe cases, all seventeen Cargo/Wasm integration
cases and all-target/all-feature package checking. Strict development Clippy and
the dependency pin, direct-boundary, Shared snapshot and documentation-link
checks pass. Before-images and focused logs are under
`/tmp/ic-build-probe-owner-20261018/`. Full release CI, native macOS and installed
IC qualification remain separate; sibling checkouts were not built or changed.

## Published-engine consumer delivery — 2026-10-18

The live sparse index confirms non-yanked Build 0.17.3, Rust 1.91. Wire's isolated
registry migration is [PR #11](https://github.com/dragginzgame/ic-wire/pull/11),
`8f16545` over released `3f5748f`. It removes the 0.14 Git pin and all three Host
0.25 packages. Rust 1.91 example checks, nineteen negative compiler cases with
passing controls, strict development Clippy, pins/boundaries/links and installed
PocketIC 16.1 macro and Chromium browser trials pass on the registry graph.
The new direct Process edge remains private evaluation tooling. No consumer
runtime or wire contract changes. Full CI/native macOS/live IC remain separate.

[Testkit #127](https://github.com/dragginzgame/ic-testkit/pull/127) is refreshed
without force-push to `ac673f5`, merging released 0.54.0 `ff88087` and preserving
its Platform decoder migration. Nineteen driver cases, floor all-target checking,
strict development Clippy and focused policy/document checks pass. Pending notes
target 0.54.1; own versions and current-main lock selections remain unchanged.
[Query #84](https://github.com/dragginzgame/ic-query/pull/84) needs no source
refresh against upstream `8982c2a4`; all thirty floor smoke cases pass again.
Neither contribution imports the normal sibling checkout's pending work.

[Canic #596](https://github.com/dragginzgame/canic/pull/596) stays draft at its
earlier validated branch head. The newer unified infrastructure pipeline exists
only in the maintainer's dirty normal checkout, so replaying the older bootstrap
modules would reverse that work. A narrow facts patch is prepared against an
isolated snapshot, preserving the process-wide environment and release checks.
Baseline and candidate both stop at the same seventeen existing Core Memory/
Storage errors before Host compilation; older Build callers also remain outside
that patch. Current-pipeline compile/test acceptance is blocked. The patch and
before-images are under `/tmp/ic-build-consumer-finish-20261018/`; no part of the
normal pending batch is pushed or imported into the shared PR. Superseded producer
PRs #59/#67/#73 are closed without merge or branch deletion: their implementations
already ship in current Build. No release, publication or merge occurs here.

## Post-extraction filesystem ownership cleanup — pending 0.17.4

Starting source is released `a4aecf9` (0.17.3), with a clean working tree and
Host 0.27.3 already selected. The repository audit covers public modules,
manifests, examples, test support, release adapters, CI, generated cache metadata
and current guides. Available Canic, Testkit, Query, IcyDB, Jobs and Wire checkouts
provide read-only consumer evidence; their dirty or older adoption paths are not
proof of integrated current-API acceptance. No sibling source is changed.

Three filesystem primitives survived their Host extraction: shared-descriptor
unlock-on-drop, metadata-only directory sizing and standard cache-tag handling.
The published Host FS 0.27.2 source and changelog establish their canonical APIs;
Build now consumes those APIs with a minimum FS requirement of 0.27.2.
The local lock guard, size walker and two copies of Host's
size tests are removed. Build retains its shared record-clone owner, pruning and
scheduling policy, tag suffix and cache-error mapping. Tag generation now names
Build; valid tags, formats, paths and existing artifacts stay intact.

Current usage guides describe 0.17.3 rather than the superseded 0.16/pending-0.17
split. Historical upgrade actions and source-bound extraction records remain.
Input traversal/semantic hashing, invocation staging, cache transactions and the
different native/Wasm protocol-refusal policies remain distinct because their
admission, identity and lifetime contracts differ. Pinned Shared payloads remain
reviewed dependencies; release adapters already delegate to their shared owner.
No additional runtime implementation deletion is justified by this inspection.

Linux Rust 1.91 passes focused cache filesystem, pruning, retained-artifact and
shared-maintenance units, both cross-process cache targets, and all-target/
all-feature checking. Strict development Clippy, formatting, dependency pins,
direct boundaries, snapshot integrity, documentation links and diff checks pass.
After initial qualification on Host 0.27.3, an external lockfile refresh selects
Host 0.27.4. That refresh is preserved; all affected focused tests, process
controls, MSRV checking and strict Clippy pass again on the new graph. The three
delegated Host implementations are byte-identical across these selected releases.
No full release gate, native macOS or installed-IC qualification is claimed.
This cleanup changes no package versions, lock selections, index, refs or retained
artifacts. The concurrent lockfile change remains separate from its source delta.
Before-images and focused logs: `/tmp/ic-build-sediment-20261018-followup/`.

## Current-engine consumer convergence — pending 0.17.3

This batch starts at released main `795143a` (0.17.2). The live sparse index
confirms published, non-yanked Build 0.17.2 with Rust 1.91. Retained Wasm evidence
and compile diagnostics from the earlier Wire branches are adapted onto current
Host 0.27 and the existing cache/decoder/capture owners. Acquisition requires an
explicit spawner. Evidence compares the acquired build fingerprint and rechecks
both semantic and conservative raw input identities; source custody and report
acceptance remain consumer-owned. No new cache, recovery state, tool installer
or format migration is introduced. Own versions and Cargo.lock are unchanged.

Consumer contributions are isolated from clean source: Query `8982c2a4`,
Testkit `55ac397`, Wire `ce56d5e`, and Canic draft #596 `b3e760931`.
The authorized delivery updates [Canic #596](https://github.com/dragginzgame/canic/pull/596)
to `b9169f165` and opens [Query #84](https://github.com/dragginzgame/ic-query/pull/84)
at `cf5fe955` and [Testkit #127](https://github.com/dragginzgame/ic-testkit/pull/127)
at `8a396ba`. These remain open for owner review; Build and Wire changes stay local.
Normal dirty sibling checkouts are never edited or imported. Query and Testkit
use registry Build 0.17.2 with complete environments and project captured compiler
facts. Canic selects registry Build 0.17.2 / direct Host 0.27.3, keeps its custom
recipes, and carries actual command recipe/environment/tool observations through
finalization. Provenance no longer performs independent probes or ambient flag
reconstruction. Generated runtime artifacts use canonical message selection.
Canic's complete-release admission remains intact; its normal checkout's separate
Memory/Storage batch is not qualified by this contribution.

Linux checks pass: four new evidence/diagnostic units, all seventeen real Cargo/
Wasm cases, Rust 1.91 all-target checking and warning-denied Rustdoc, strict
development Clippy, formatting, snapshot, pins, direct boundaries and doc links.
Query's thirty smoke cases and Testkit's nineteen driver cases pass on Rust 1.91;
the affected consumer targets also pass strict Clippy and floor checking.
Canic passes 99 focused cases, strict all-target Clippy and Rust 1.91 production
checks. Its first nested-Cargo attempt lacked global offline policy; explicit
prepared offline execution passes. The opt-in reuse measurement remains ignored.

Wire passes floor example checking/Clippy, nineteen negative diagnostics plus
passing controls, and an installed PocketIC 16.1 macro trial covering query/update
exports, typed failures and sixteen rejection cases. The sandbox first refused
localhost binding; the explicitly elevated retry passes with a prepared Testkit
server and fresh output paths. No live IC, native macOS or full release gate is
claimed. Qualification used a frozen producer source override. Wire's reviewed
registry graph removes its Git-pinned 0.14 engine and Host 0.25 packages, but the
new APIs still require producer publication; this is a dependent migration,
not registry-graph compile acceptance. No source override is included in its
review diff. Full release/integration and published-graph Wire acceptance remain
separate. Source identities, isolated branches, before-images, qualified graph
receipts and logs are retained under `/tmp/ic-build-adoption-20261018/`.


## Outcome facts and consumer hard cuts — pending 0.17.2

Over released `86eddca`, Wasm records retain the same Cargo recipe/tool facts
already admitted by their resolved snapshots. Cold builds, exact reuse and
cloned records expose those facts without another probe. The record constructor
now takes the authoritative resolved snapshot rather than separate digest fields.
No cache key, stamp, source-change check, retention or Debug redaction changes.
Own package versions and the selected Host 0.27.2 dependency graph stay unchanged.

Normal sibling trees contain extensive unrelated migrations and remain untouched.
Isolated Query and Testkit changes project the observed compiler identity instead
of executing another compiler after acquisition; Testkit also uses the observed
Wasm profile. Query consumes the existing resolved-snapshot API; Testkit needs
this pending producer addition. Their source patches omit temporary dependency
overrides and are not published-graph acceptance or integrated consumer changes.

The Canic candidate adapts the existing shared-execution draft to current pending
source: one canonical typed Cargo decode feeds exact artifact selection and
Canic-owned include-export policy, replacing its raw selector and heartbeat
thread. Both baseline and candidate stop at the same seventeen Core/Memory/Storage
errors before Host compilation. This is prepared source, not completed acceptance.
[Canic draft #596](https://github.com/dragginzgame/canic/pull/596) already owns the
shared-execution contribution; do not create a competing implementation.
Canic provenance still uses custom commands without engine records. Its exact
execution facts must be carried through those paths before removing the old
probes; do not substitute facts from an unrelated Wasm recipe or weaken admission.

Rust 1.91 passes seventy cache units, thirteen batch units and all seventeen real
Cargo/Wasm integration cases; floor all-target checking, strict development Clippy,
warning-denied floor Rustdoc, snapshot, pins and direct-boundary checks pass.
Isolated Query smoke tests pass thirty cases; Testkit driver tests pass nineteen
against the frozen candidate and captured consumer lockfiles. Sibling dependency
work continued during this batch; current normal graphs need fresh qualification.
These are Linux development checks, not full release,
native macOS or installed-IC qualification. Source patches, before-images, graph
checks and logs are retained in `/tmp/ic-build-converge-20261018/`. No staging,
commit, version bump, push, release or publication is performed.

## Platform boundary audit — pending 0.17.1

The live crates.io sparse index selects non-yanked IC Platform 0.1.3, published
2026-10-18T08:04:25Z. The downloaded archive's SHA-256 matches the registry checksum
`d68e00f23aa3f397b26ccecc706b27e1d3468dece25f16c26a5aedbc79523ec2`;
its VCS record is `b22481d6884cf57d3960ef869b9280d8d471d94c`.
The crates.io API refused this environment with 403; live sparse-index and archive
receipts establish publication independently of sibling source or cached 0.1.2.
The published manifest/source/README and that exact commit's migration guidance
were inspected: empty default features, unconditional Principal, optional bounded
Candid, certification and Candid-free executor surfaces. No full CDK call/macros
or host parser surface is supplied by this release.

Both actual manifests, every normal/build/dev/target edge and alias, enabled
features, public types, Rust sources, examples, fixture generators and independent
workspace inventory were inspected. Only the root and ic-build manifests exist
outside tools/build output. There are no direct or transitive Candid/CDK packages
in the all-target/all-feature graph, including development dependencies. The
native extraction adapter uses admitted subprocesses and opaque bytes, not a
Candid codec, parser, DTO or executor. No source replacement, dependency addition
or obsolete adapter deletion is justified; wire/error/limit/lifecycle contracts
and application protocols remain untouched. No unused Platform dependency is added.

The committed Shared Tooling 0.24 exporter adds its direct-boundary checker as
the 110th payload without adopting the sibling's unrelated dirty work. Its local
policy bans the listed Candid/parser/CDK package names across member declarations;
`make check-boundary` runs directly and in CI/release validation. It handles
aliases, inactive optional declarations and dev/build/target edges. This is an
exact-name direct guard, not a transitive, future-family-name or deployed-Wasm
admission rule. Future independent workspaces need their own invocation. Build
is a host library; canister consumers own deployment graph/artifact admission.

Actual Rust 1.91 passes library checking, all eight extraction-process controls
and all seventeen real Cargo/Wasm integration cases. The extractor fixtures are
synthetic tools, not an installed extractor qualification. Snapshot integrity,
boundary/pin checks, formatting and shared release routing pass. No Platform
API blocker applies to this repository, so no upstream issue is filed. Full
release gates, native macOS and installed-IC qualification remain separate.
Local release-adapter fixtures and ShellCheck also pass. The standard execution
guard refuses CI dry-run inspection; no full CI run or substitute success is claimed.
Dependency manifests, Cargo.lock, the real index and the earlier Shared adoption
are preserved. Evidence: `/tmp/ic-build-platform-adoption-20261018/`.

## Shared Tooling 0.24 and dependency roles — pending 0.17.1

The canonical exporter refreshes the existing selection plus the formatting-hook
qualifier to Shared Tooling 0.24.0 `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`.
Its 109 recorded payloads include the release fixture-preparation companion closure.
Common setup/checks select cargo-sort and cargo-sort-derives, preserving retained
extractor installations; Build's Candid API still receives caller-selected tools.

The sole production package is `ic-build`. All eleven root catalog entries serve
normal or Unix-specific normal dependencies. Test/tooling-only member edges enable
Host fixture features on shared production entries; no separate test-only package
or catalog entry exists. Inline role labels retain global manifest ordering.
Current, commit-preparation and selected-commit release checks now explicitly use
that same policy. Hook qualification preserves role labels, rejects partial staging
and formatter failures, and preserves locks and unrelated edits. Local adapter
fixtures reject unsorted current/staged/selected-source manifests, including lost
preparation replies, while accepting a sorted selected commit over an unsorted
working manifest. No live release or real index/ref changes occur.

Locked normal/build graphs for Linux x86-64, macOS Intel and Apple Silicon contain
no Testkit/PocketIC packages or Host test-support features. Default/all-feature
graphs agree; Build declares no feature switches. Actual Rust 1.91 production-only
Linux checking passes. This is graph inspection for macOS, not native compilation.
The pin checker enforces declarations/inheritance, not comment roles or transitive
deployment exclusions. This host-library repository has no deployment admission
gate; canister consumers must enforce their own target/feature exclusions and
artifact/source binding. Snapshot/format success is not deployment acceptance.

Common setup, tools/format checks, formatting-hook controls, release-routing and
local release-adapter fixtures pass. The first sandboxed setup could not write
the shared tool store; explicitly elevated setup succeeds. Root/member metadata
selections, package version 0.17.0, Cargo.lock and real index remain preserved.
Evidence and before-images: `/tmp/ic-build-shared024-20261018/`. Full release and
native macOS gates remain separate; no sibling source is changed.

## Post-link fixture compiler selection — pending 0.17

The reported host-composition failure reproduces on `aa0ff08` with the validation
adapter's explicit RUSTC: it overrides the identity-only shim, so changing the
unused shim correctly leaves both caches warm. The synthetic fixture now omits
RUSTC from its complete environment to exercise its selected shim. Production
precedence and all compiler/optimizer invalidation assertions remain unchanged.
All three Host composition tests pass with explicitly selected Rust 1.99 and
1.91; the independent explicit-RUSTC precedence control also passes on 1.91.
Before-images and logs: `/tmp/ic-build-post-link-fixture-20261018/`.

## Observed provenance facts and tracker reconciliation — pending 0.17

Issue #71 / PR #73's producer proposal is adapted locally to the incoming 0.17
explicit-environment source over released `546ea07`. Resolved input snapshots
retain the exact successful probe bytes already used by their fingerprints,
typed Cargo selection, target and complete selected environment. Fingerprinting
uses those retained facts without another probe or ambient capture. Identity
bytes preserve the existing stdout-then-stderr encoding; they are not stdout-only.
Debug omits environment values and probe payloads. Consumers still own execution
binding, source custody, provenance redaction and complete-release admission.

PR #57's Host 0.26 update is superseded by released main's 0.27 boundary and is
closed without merge or branch deletion. Issue #70's public progress-signature
proposal is withdrawn in favor of #75; #72 now explicitly retains the five-part
batch tuple. Issue #9 names the current published boundary and remains open for
consumer acceptance. Normal sibling trees and manifests remain unchanged; no
Canic provenance probe deletion, producer release or PR-head refresh is claimed.

Actual Rust 1.91 passes 69 Wasm-cache units, 13 batch units, the shared Cargo
execution integration and all 17 real Cargo/Wasm cases. The fingerprint fixture
is unchanged; retained facts survive source changes and probe executable removal,
batch resolution keeps its existing probe counts, and Debug hides values/bytes.
Floor all-target checking, strict development Clippy, warning-denied floor Rustdoc,
formatting, changed-document links and diff checks pass. Clippy's initial test-length
failure was corrected by removing redundant assertions. Root/child manifests,
lockfile and real index are preserved. Evidence: `/tmp/ic-build-issues-20261018/`.
Full release gates, native macOS and Canic provenance adoption remain separate.

## Extraction-boundary cleanup — 2026-10-18, pending 0.17

The audit starts from `546ea07` plus the incoming dirty 0.17 implementation.
Build owns Cargo observations/input resolution, acquisition, artifact transactions,
retention and explicit maintenance. The selected registry Host 0.27.1 owns byte,
filesystem and process primitives. Consumers retain recipes, source custody,
tools and lifecycle. Normal Canic, Testkit, IcyDB, Query and Jobs sources and
manifests were inspected read-only; their adapters confirm direct Build usage,
not completed adoption of the pending 0.17 API. No sibling changes are made.

Staging close now delegates to Host's existing consuming cleanup, preserving one
attempt and caller recovery after failure. The unwind control exercises Build's
staging/capture adapter rather than only Host's guard. Runnable Cargo examples
replace README snippets that still described withdrawn progress arguments.
Current guides distinguish released 0.16 from pending 0.17 and name the three
actual Host dependencies. Obsolete downstream Host/PocketIC migration prose is
removed; pinned extraction designs and the implementation evidence below remain.

Similar paths remain deliberately separate: native Cargo interrupts on protocol
refusal while Wasm drains diagnostics; semantic input fingerprints differ from
raw Host hashes; consumer release/testing/optimizer adapters retain their policy.
The `.ic-testkit` names remain live cache/lock identities, not compatibility code.
Nested package targets are ignored for future additions and excluded from the
package archive. `cargo package --list` exposed the two tracked build files in
the original package inventory; the exclusion removes them from that inventory.
The files remain byte-for-byte intact; the real index and artifacts are untouched.

Actual Rust 1.91 passes example compilation and all 18 staging/capture tests,
including permission refusal as an ordinary user. Strict development Clippy and
formatting pass. The native example builds and runs a real executable; the Wasm
example consumes real Cargo outputs in cold and warm invocations. Changed-document
links, snapshot integrity and diff checks pass. Focused logs and before-images are retained in
`/tmp/ic-build-sediment-20261018/`. Full release gates, native macOS qualification,
producer delivery and published-graph consumer acceptance remain separate.

The entries below are source-bound implementation history. Later decisions can
withdraw earlier proposed API cuts; use the current source and artifact guide
for supported signatures. In particular, the shared-execution review retains
released standalone progress entrypoints and the five-part public batch tuple.

## Shared Cargo execution and one prepared lifecycle — pending 0.17

This batch extends the incoming dirty 0.17 tree over `546ea07`. Wasm acquisition
uses `cargo::execution` for child communication and canonical Cargo messages.
Execution and protocol evidence remain separate: failed exits keep their original
diagnostics, protocol refusal drains the child, output-sink failures retain Host
cleanup, and successful publication checks both outcomes. Exact manifest-bound
cdylib selection moved from Canic; no filename inference or stale-file fallback.
Consumers retain commands, environment, budgets, spawning, source custody and
artifact capture/lifetime. Keys, retained formats and stamp ordering are unchanged.

Prepared snapshots are the sole cross-call source-lease reuse model. Mutable
Wasm sessions, their state/metrics and separate recovery branches are removed.
Prepared admission, concurrent-reader invalidation and publication guards remain;
ordinary batches still resolve current inputs and share work within each batch.
Released silent/progress entrypoints and five-part batch consumption are retained,
with shared private dispatch and canonical internal failure storage. The earlier
pending public progress/tuple cuts are withdrawn. Explicit environments and graph
selections remain the current semantic changes; own version stays 0.16.0.

Actual Rust 1.91 passes all-target/all-feature compilation, 84 focused units,
30 integration parents and the doctest. Controls include real Cargo build-script
exports, colliding custom libraries with retained capture, feature isolation,
source mutation and concurrent prepared readers. Synthetic controls cover process
refusals, retention and interruption. Strict development Clippy, warning-denied
floor Rustdoc, formatting, snapshot, pins, links and diff checks pass. Full release
gates and native macOS qualification remain separate.

Canic's isolated stacked draft is committed as `b3e760931` over PR #593's
`83976fd14`. Its app/bootstrap routes delegate communication and typed selection,
retaining declaration/runtime recipes, export policy and complete-release admission.
Rust 1.91 production-library checks, strict development all-target Clippy and
65 focused cases pass against the frozen candidate and direct Host 0.27.1.
Published dependency selectors/lock are restored to the base until 0.17 publication
permits exact registry qualification. No path override is committed. The maintainer
authorized the topic push and [Canic draft #596](https://github.com/dragginzgame/canic/pull/596)
is open, stacked on #593; no consumer integration, release, publication or native
qualification is claimed.

[#75](https://github.com/dragginzgame/ic-build/issues/75) owns delivery and consumer
acceptance. Build's manifests/lock, index and refs and the normal sibling checkouts
are preserved. Before-images, frozen producer, candidate graph and focused logs
remain in `/tmp/ic-build-execution-reuse-contracts-20261017/`.

## Verified publication and recipe-selected grouping — pending 0.17

This batch extends the incoming dirty 0.17 source over `546ea07`. Wasm
publication carries each cache path, public path and verified digest together.
All cache copies precede cache stamps; all public copies precede public stamps.
Warm repairs preserve matching files and stamps, including their identities and
times. Cargo output-count mismatches refuse before copying, with recovery through
a complete inventory. Existing keys, stamp bytes, namespaces and lifetime stay
unchanged by this batch.

`cargo::compatible_package_groups` requires the existing recipe `CargoSelection`
after `target`. Features, locked and offline choices apply to every isolated and
combined probe; profiles remain outside Cargo tree. Configuration and lock
projection errors derive Display/Error through `thiserror`, preserving variants,
exact messages, redaction and original sources. The direct inherited dependency
reuses selected 2.0.21; no dependency versions or checksums change.

Actual Rust 1.91 passes all-target/all-feature compilation, 87 focused units,
29 integration parents and the library doctest on Host 0.27.1. Real Cargo/Wasm
controls cover feature-dependent grouping, artifact names, source races and
prepared readers; synthetic producers cover retention and process coordination.
Three worker harness entries remain ignored in parent listings and dispatched
explicitly. Shim fixtures explicitly remove adapter-selected RUSTC for identity-
only cases and pass with the normal validation adapter's RUSTC selection. Strict
development Clippy, warning-denied floor Rustdoc, formatting, snapshot, pins,
links and diff checks pass. Full release gates and native macOS qualification
remain separate.

[#74](https://github.com/dragginzgame/ic-build/issues/74) owns delivery and Canic's
grouping-adapter acceptance. Canic must pass its compilation recipe's graph
selection when adopting 0.17. Normal sibling checkouts remain read-only; no
consumer migration, release or publication is claimed. Own version stays 0.16.0;
the lockfile adds only Build's direct `thiserror` edge. Incoming dirty work, index
and refs are preserved. Before-images and focused logs are retained in
`/tmp/ic-build-publication-selection-errors-20261017/`.

## Explicit environments and admitted batch inputs — pending 0.17

This batch extends the incoming dirty source over `546ea07`. Wasm construction
requires a complete OS-native environment. Identity probes, metadata and builds
clear inheritance and use that map; Build adds its owned Cargo target directory
after applying it. Constructors select initial Cargo/Rustc from explicit values
or literal program defaults. Consumers own capture, filtering and recapture.
Environment replacement is complete, absent values remain absent, malformed
entries refuse before work and Debug omits values. Fixture adapters own their
explicit toolchain/runtime selection; no production ambient-capture helper or
compatibility builder is kept.

The recipe domain becomes `ic-build-explicit-environment-fingerprint`, preventing
old inherited execution from authorizing the new cleared-environment behavior.
An independent Node framing fixture reproduces the former v2 golden key and
computes the new explicit-environment key. Existing entries, stamp formats and
namespaces remain retained without migration or cleanup. Prepared admission
binds all requested snapshots once in request order before reuse counts or
progress; source-lease checks and publication invalidation remain intact.
The internal result keeps each error paired with its phase and partial timings.
The public five-part consumption is retained by the later contract review below.

Actual Rust 1.91 passes all-target/all-feature compilation, 123 affected units,
29 integration parents and the library doctest on the unchanged Host 0.27.1
graph. New controls cover cleared inheritance for both identities, metadata and
builds, warm reuse, complete replacement, invalid names/values, OS-native values
and mixed known/unknown admission without partial counting. Real Cargo controls
cover feature isolation, source races and concurrent prepared readers; synthetic
producers cover retention and process coordination. Three worker harness entries
remain ignored in parent listings and explicitly dispatched by their tests.
Strict development Clippy, warning-denied floor Rustdoc, formatting, snapshot,
pins, links and diff checks pass. Full release gates and native macOS qualification
remain separate.

[#72](https://github.com/dragginzgame/ic-build/issues/72) owns delivery and consumer
acceptance. Canic, Testkit and IcyDB adapters must supply their complete environment
when adopting 0.17. Their normal
checkouts remain read-only, and no migration, release or publication is claimed.
[#71](https://github.com/dragginzgame/ic-build/issues/71) separately owns exposing
observed provenance facts; it should reuse this selected map and observations.
Own version stays 0.16.0; manifests/lock, index and refs are preserved. Before-images,
the independent key fixture and focused logs are retained in
`/tmp/ic-build-environment-admission-result-20261017/`.

## Unified observation and published Host mechanics — pending 0.17

An earlier pending proposal unified standalone progress signatures. The later
contract review retains the released public silent/progress entrypoints while
sharing their private dispatch; heartbeat refusal remains shared before work. Custody, stdin, observer panic
cleanup, batch labels, failure details and source-lease invalidation stay intact.

Cargo decoding delegates byte framing to published Host Process, retaining JSON/
text interpretation, newline-inclusive bounds, accepted EOF tails and original
sink errors. Wasm still drains after protocol refusal while native execution
retains its existing interruption policy. Artifact atomic copying composes Host
FS's durable writer with std::io::copy inside Build's existing adapter. The pending
copy_from adoption is withdrawn to match Host's pending removal of that wrapper;
ordinary source opening, permissions, path context and publication/cleanup evidence
remain unchanged. No cache format, key or retained-artifact change.

The unchanged selected graph is registry Host 0.27.1. Actual Rust 1.91 passes
all-target/all-feature compilation, 100 focused units, 30 enabled integration
parents and the library doctest. Real Cargo controls cover native diagnostics,
Wasm feature isolation, exact outputs and source races; synthetic producers cover
process/retention behavior. Three worker harness entries remain ignored in parent
listings and explicitly dispatched by their process tests. Exact framing bounds,
EOF-tail/sink refusals, observer cleanup and atomic publication failures pass.
Strict development Clippy, warning-denied floor Rustdoc, formatting, snapshot,
pins, links and diff checks pass. Full release gates and native macOS qualification
remain separate. Own version stays 0.16.0; manifest/lock, index and refs are intact.

[#70](https://github.com/dragginzgame/ic-build/issues/70) owns API delivery;
[#47](https://github.com/dragginzgame/ic-build/issues/47) owns framing adoption.
[#24](https://github.com/dragginzgame/ic-build/issues/24) records the withdrawn copy
wrapper adoption; Build retains direct durable-writer composition.
Normal Canic, Testkit and IcyDB checkouts remain read-only. Testkit's standalone
callers need the new optional argument; IcyDB's progress caller needs the unified
entrypoint when adopting 0.17. Canic's older normal checkout still needs its
existing broader adoption. No consumer migration, release or publication is
claimed. Before-images and focused logs are retained at
`/tmp/ic-build-framing-progress-copy-20261017/`.

## Shared Tooling 0.23 and Host 0.27 — pending 0.16

Shared Tooling is refreshed through its canonical exporter to 0.23.0 revision
`0b351cd2ba2bcd8927b783a5fec89afc8013f708`. All 104 payloads verify against
the recorded digests and modes. The incoming manifest/lock update already
selected registry Host 0.27.0; it is preserved byte-for-byte and qualified here.
Exposed Host values require matching direct consumer dependencies. Build names
no former Process-owned `FileIdentity`, so that relocation requires no source fix.

Common setup and offline admission now cover the seven host/Cargo tools.
This checkout has no installed IC-executable caller. Its release-adapter fixture
therefore copies only the selected common installations, and no longer depends
on an IC installation. README setup describes explicit product-tool selection;
existing IC installations and retained artifacts are untouched. Optional optimizer
qualification remains separately owned by the existing Build PR #62.

Actual Rust 1.91 passes all-target/all-feature compilation, 44 focused units,
nine enabled integration parents and the library doctest on Host 0.27.0.
Real Cargo controls cover native diagnostic streaming, both Wasm cache modes and
source-change rejection. Synthetic producer controls cover retained handoff and
cross-process coordination; worker harness entries remain explicitly dispatched.
Common setup/checks, snapshot integrity, pins, links, release-command routing,
isolated release/publication adapters, strict development Clippy, warning-denied
floor Rustdoc, formatting, focused ShellCheck and workflow lint pass.
Full release gates and native macOS qualification remain separate. No consumer
migration, release or publication is claimed. Before-images and focused logs are
retained in `/tmp/ic-build-tooling-host-20261017/`. Earlier dirty work, root
manifest/lock selections, index and refs are preserved; own version stays 0.15.0.

## Preparation refusal, admitted outputs and canonical schedules — pending 0.16

This batch extends the incoming dirty source over `8b85c74`. Artifact preparation
returns the existing before/after input-change error on the first detected
mutation, without a hidden retry loop or exhaustion assertion. Consumers retain
source-custody and retry decisions. Retained outputs survive refusal and explicit
fresh preparation can recover. Wasm output admission groups target facts and
paths in one success/refusal; generic non-Wasm snapshots still support validation,
with refusal deferred until outputs or Wasm acquisition are requested. Invalid
selections retain no partial output inventory. These cuts do not change existing
pending recipe keys, stamp bytes, namespaces or artifact lifetime.

Exact-entry maintenance stores one optional limits/interval configuration.
Zero means always; absence disables maintenance. Always-run paths skip due-marker
reads and still record attempts. Both builders return `None` from the Wasm
interval getter; its policy getter distinguishes disabled maintenance. Positive
intervals, marker bytes, nonfatal errors and the separate shared-target
success-marker rule remain intact.

Actual Rust 1.91 passes 128 affected units, 29 enabled integration parents and
the library doctest. The new deterministic input-race regression fails against
an isolated before-image with the former retry loop and passes against current
source, checking digest evidence, retained bytes and explicit recovery. Both
always-run builders pass corrupt-marker cases for transactions and Wasm. Existing
real Cargo/Wasm feature, output, source-race and prepared-reader controls pass,
alongside synthetic producer retention/process cases. Four explicit workers pass;
parent harnesses retain one ignored opt-in measurement and two ignored worker
entries. Floor all-target/all-feature compilation, strict current Clippy,
warning-denied floor Rustdoc, formatting and documentation links pass. Full
release gates and native macOS qualification remain separate.

[Build #69](https://github.com/dragginzgame/ic-build/issues/69) owns delivery
acceptance. Read-only Canic inspection confirms it already treats this error as
input mutation; no interval-getter callers were found in the inspected Canic,
Testkit and IcyDB source scopes. No consumer migration is claimed. Before-images,
the counterfactual source and focused logs are retained in
`/tmp/ic-build-preparation-admission-schedule-20261017/`. Incoming dirty work,
root manifests/lockfile, staging, refs and normal sibling checkouts are preserved;
own package version remains 0.15.0.

## Shared retention limits, tagged failures and typed metadata — pending 0.16

This batch extends the incoming dirty source over `8b85c74`. Exact-entry and
shared-target maintenance use one `CacheRetentionLimits` value. Thresholds and
maintenance-marker identity bytes stay unchanged; LRU/retained-entry pruning and
whole-target clearing remain distinct operations. No compatibility type is kept.
Artifact batch failures identify `Preparation`, `Callback` or `Commit` directly;
the separate phase accessor is removed while primary/cleanup errors, collect-all
order and partial/report timing agreement remain intact.

Cargo observations decode complete bytes once into the canonical schema. Wasm
resolution borrows those package/node facts without retaining a parallel raw JSON
tree or joining array positions. Typed semantic fields use schema defaults and
version normalization. Dependency aliases, features, kinds, platforms, package
metadata and selected lock checksums remain significant; unsupported dependency
kinds refuse. An independent key fixture qualifies the new
`ic-build-wasm-fingerprint-v2` domain for all recipes, including conservative
fallbacks. Old retained entries and v1 stamp formats remain intact; recipes build
under new keys without a migration reader or automatic cleanup. Raw workspace
manifest/lock mutation validation remains independent of semantic reuse.

Actual Rust 1.91 passes 132 affected units, 29 enabled integration parents and
the library doctest. Typed projection controls cover normalization, relocation,
aliases, known kinds/platforms and checksum selection. Real Cargo/Wasm tests cover
feature isolation and source-lease invalidation; synthetic producer fixtures cover
retention and process failures. Four explicit subprocess workers pass. Parent
harnesses retain one ignored opt-in measurement and two ignored worker entries.
Floor all-target/all-feature compilation, strict current Clippy, warning-denied
floor Rustdoc, formatting and documentation links pass. Full release gates and
native macOS qualification remain separate.

[Build #68](https://github.com/dragginzgame/ic-build/issues/68) tracks delivery and
consumer acceptance. Read-only Canic inspection finds the former limits in
`crates/canic-testing-internal/src/pic/artifacts.rs`; IcyDB's
`testing/build/src/canister_build_cache.rs` needs the limit and failure-variant
updates. Coordinate those with the existing 0.16 spawner/flag adoption in #64;
no consumer migration is claimed. Before-images, an independent key fixture and
focused logs are retained at `/tmp/ic-build-retention-failures-metadata-20261017/`.
Incoming dirty work, root manifests/lockfile, staging, refs and normal sibling
checkouts are preserved. Own package version remains 0.15.0.

## Exact depfiles, configuration digests and batch results — pending 0.16

This batch extends the incoming dirty 0.16 source over `8b85c74`. Depfile inputs
now have one explicit-record reader; directory-wide scanning is removed without
an adapter. Consumers select graph records, combine inputs and decide whether
`NotFound` permits a cold inventory. Root fixtures select real Cargo records;
strict parsing, no-follow/bounded reads, resolved exclusions and pre-work input
authority remain intact. Read-only Canic inspection confirms its dependency
adapter already selects exact records and handles cold absence; no consumer
migration or qualification is claimed.

Configuration observations retain Host's `Sha256Digest` instead of unpacking raw
arrays. Independent SHA-256 comparison verifies the same bytes; config contents
remain excluded from returned facts and diagnostics. Wasm batch internals return
their existing result directly, retaining inline failure details, partial timings
and source-lease invalidation. Narrow size-lint expectations document that choice.

Actual Rust 1.91 passes 123 focused cases: 20 input-evidence, four configuration,
13 batch and 62 cache units, plus 24 real Cargo/Wasm integration parents. Failure
collection, progress, retained artifacts, feature isolation and source-race
invalidation controls pass. All-target/all-feature floor compilation, strict
development Clippy, warning-denied floor Rustdoc, formatting and documentation
links pass. Full release gates and native macOS qualification remain separate.
Before-images and focused logs are retained in
`/tmp/ic-build-depfile-batch-config-20261017/`. Incoming dirty work is preserved;
root manifests, lockfile, staging, refs and normal sibling checkouts are unchanged.

## Explicit Wasm spawning, pure flags and Host cache tags — pending 0.16

This batch extends the incoming 0.16 source over `8b85c74`. Standalone builds,
batches, source-lease sessions and prepared readers now require a Host child
spawner. It selects direct/group custody and cleanup independently of progress
and may override the default null stdin. Probe policy, recipe identity, retention,
feature isolation and the absence of a default compilation deadline remain intact.
Rust-flag composition accepts explicit encoded/plain inherited values. The caller
selects its command environment; the six-process dispatcher is replaced with pure
precedence cases and one real Cargo remap check.

Actual Rust 1.91 passes 62 Wasm-cache units, 13 batch units, two flag cases,
the affected admitted-Host caller, 29 enabled integration parents and the library
doctest. Three integration cases
remain ignored in their parent harness (one opt-in measurement and two child
workers); the process parents explicitly execute the child workers. Native
process checks cover progress on/off, direct/group custody and supplied stdin;
existing panic, descendant, feature and input-race controls pass. All-target/
all-feature floor compilation, strict current Clippy and warning-denied floor
Rustdoc pass. Full release gates and native macOS qualification remain separate.

[Host PR #183](https://github.com/dragginzgame/ic-host-tooling/pull/183) extracts
cache-tag recognition and durable replacement. Four focused Host cases, floor
all-target compilation/Rustdoc and strict Clippy pass on its declared Rust 1.88
graph. A frozen Build caller passes ten cache-filesystem cases and six retained/
process integration parents on Rust 1.91, plus floor all-target checking and strict
Clippy. Its lock differs only in the three coherent Host source/checksum rows.
The normal Build graph retains its local implementation until reviewed publication
permits a real minimum. The apply-checked deletion patch has SHA-256
`5a30df64c67766625e161a0cf2ef991bab2c72777c3c8d71eeba05caa05fad70`.

[Build #64](https://github.com/dragginzgame/ic-build/issues/64) owns delivery,
Host adoption and caller coordination. Canic's embedded-root cache build,
Testkit's fixture/benchmark builds and IcyDB's testing/build flag/cache adapters
need the new arguments when adopting Build 0.16; no consumer migration is claimed.
Before-images, the isolated Host contribution, frozen caller, patch and focused
logs are retained at `/tmp/ic-build-execution-flags-tags-20261017/`. Earlier dirty
work is preserved. Root manifests, lockfile, staging, refs and sibling checkouts
are unchanged; the isolated Host contribution alone is committed and pushed.

## Exact-cache reuse, native selection and verification TOML — pending 0.16

Local changes over `8b85c74` remove exact-entry reconstruction from mutable public
Wasm outputs. Missing or invalid entries now run Cargo, while valid entries still
repair public outputs and stamps. Retention locks still prevent replacement of
damaged entries held by live records. Existing cache/digest/stamp bytes and
namespaces stay unchanged; no migration or automatic artifact cleanup is added.

Native executable specifications now require `cargo::native::NativeTargetKind`
(`Binary` or `Example`), without a string-selector adapter. Repository callers and
the README example use it. The read-only Testkit checkout still has one string
caller in `crates/ic-testkit-bench/src/fixture_reuse_driver/mod.rs`; it needs
`NativeTargetKind::Binary` when adopting this Build release. This does not claim
consumer migration. Package verification manifests use TOML serialization;
exact generated-manifest checks and independent graph/lock admission remain intact.

Actual Rust 1.91 passes all-target/all-feature compilation and 124 focused cases:
61 Wasm-cache units, 44 Cargo units, real native and archive qualification, and
17 real Cargo/Wasm integration parents. Synthetic cache units cover all three
cache modes, a valid stamped public replacement, oversized stamps and retained
entry refusal. The real batch fixture checks mixed rebuild/reuse after cache loss.
Strict development all-target/all-feature Clippy, warning-denied floor Rustdoc,
formatting and documentation links pass. Full release gates and native macOS
qualification remain separate. Before-images and focused logs are retained in
`/tmp/ic-build-three-cuts-20261017/`. Own version remains 0.15.0; the breaking
selector and cache semantics select undated 0.16.0 notes. Manifests, lockfile,
staging, refs and normal sibling checkouts are unchanged.

## Canonical hashes, typed inventories and Cargo arguments — pending 0.15

The local source over `fbb732b7` now stores raw file facts directly as Host's
`Sha256Digest`. `InputDigest` remains the framed inventory/cache identity;
consumers render file hashes through `Display`/`to_string()`. No alias, digest
format change, namespace reset or retained-data migration is introduced.

Archive qualification retains typed digest/mode facts and serializes only when
writing evidence. The serializer preserves the former JSON bytes, including
dependency-selected field ordering; comparison retains no JSON objects or hex
strings.
Traversal, byte/depth/entry bounds, mode checks and no-follow admission stay intact.
Observations and recipes now use one graph-argument encoder. Recipe argument/key
bytes stay unchanged; observations use the same lock/network/feature flag order.
Their public specifications and process/error policy remain unchanged.

Actual Rust 1.91 passes all-target/all-feature compilation, 20 input cases,
19 Cargo selection/observation/package cases, real archive qualification and
18 native/Wasm integration parents. Strict development all-target/all-feature
Clippy, warning-denied floor Rustdoc, formatting and documentation links pass.
Existing encoding controls and a byte-for-byte source-inventory JSON comparison
qualify the retained formats. During the caller rehearsal, a concurrent manifest
and lock update selected Host 0.26.1. That incoming update is preserved, and all
58 focused tests, compilation, strict Clippy and floor Rustdoc above pass again
on that graph. Own package version stays 0.14.0; notes select pending 0.15.0
for the public hash field and exposed Host identity changes.
Full release gates and native macOS checks for this dirty source remain separate.

[Build #60](https://github.com/dragginzgame/ic-build/issues/60) owns delivery.
Canic's three raw file-hash callers are pushed in its existing isolated
[PR #593](https://github.com/dragginzgame/canic/pull/593) at
`83976fd140891f6df0051a07be45199a90c645e8`. Registry Build 0.14 / Host 0.25
passes actual Rust 1.91 libraries and strict development all-target Clippy.
A separate frozen candidate with this hash cut on Host 0.25 passes those checks
and all 44 enabled reuse cases; its opt-in measurement remains ignored. Only
Build's source/checksum lock row differed during rehearsal. The registry lock is
restored byte-for-byte; no path override or unpublished requirement is committed.
That rehearsal does not qualify Canic's future Host 0.26 adoption. Match the
actual published Build/Host pair when updating its dependency graph.

Normal sibling checkouts and incoming README/handoff work remain untouched.
Before-images, frozen sources, graph receipts and focused logs are retained in
`/tmp/ic-build-representation-cuts-20261017/`. Root staging, refs, own versions
and release flow remain user-owned and unchanged.

## Published 0.14 and Canic registry adoption — 2026-10-17

Build 0.14.0 is published from `fbb732b7ef567b210d971d55af58b7cc1268cf59`.
The cached registry index and archive agree on SHA-256
`957e87919fc46aed675896ee99cd01d0e18baca91c459b52ee2211d38d670f09`;
the archive VCS receipt names that source and `crates/ic-build`.
The [matching release CI](https://github.com/dragginzgame/ic-build/actions/runs/39031820789)
passes Linux x86-64, native macOS 15 Intel and Apple Silicon, and lint/security
for that exact published source. These producer gates do not establish consumer
adoption or complete-application qualification.

[PR #53](https://github.com/dragginzgame/ic-build/pull/53) and
[PR #55](https://github.com/dragginzgame/ic-build/pull/55) are closed as superseded:
their complete producer additions are present in the published source.
[Host PR #174](https://github.com/dragginzgame/ic-host-tooling/pull/174) remains
open with all required checks passing. Its shared descriptor-lock extraction is
still unpublished; Build retains its current guard and
[#54](https://github.com/dragginzgame/ic-build/issues/54) remains open.

Canic registry adoption is pushed in the isolated
[PR #593](https://github.com/dragginzgame/canic/pull/593) at
`dc55c01b690988e5e29862df28c8b72a01f5958e`, extending `5181a159` from public
main `53f1f22`. Normal Canic's dirty unpublished batch is excluded.
The published requirements are Build 0.14 and matching Host 0.25; no production
path override, package-version bump or compatibility implementation is introduced.
Canic's committed install-limit policy is reused from `56f8d65c1` at its canonical
path, over Host's inspected facts. Complete-release authority stays with Canic.
Actual Rust 1.91 production-library checks, strict development all-target Clippy,
44 enabled reuse cases and affected Wasm/artifact/Root/feature and internal artifact
cases pass. Rejection-only recapture preserves the primary typed failure. The PR
records the existing opt-in ignored measurement, older-compiler test-source limit
and failures owned by Canic PRs #569/#519. Consumer integration, required hosted
checks and installed qualification remain separate from producer publication.
Before-images and focused logs are retained in
`/tmp/ic-build-canic-014-20261017/`.

A concurrent [PR #57](https://github.com/dragginzgame/ic-build/pull/57), at
`77be04d9236e8b671cb74e5728020a6d8443533c`, prepares the separate exposed Host
0.26 boundary for pending 0.15 notes and Testkit. Its required native checks are
in progress. Keep that active owner separate; registry Build 0.14 consumers must
still align direct exposed Host values to 0.25 until the newer Build is published.

## Historical producer and caller receipts

The sections below retain evidence for their named source and release state;
“pending 0.14” describes those earlier preparations, not current publication.

Development Clippy 1.99 rejected two default-selection emptiness assertions
with `assert_is_empty`; they now compare explicit empty `OsString` arrays.
`make clippy` and both selection tests pass on development Rust 1.99; both tests
also pass on Rust 1.91. No release was retried. Focused logs are retained under
`/tmp/ic-build-empty-assertions-20261017/`.

## Canonical feature selection, label errors and phases — pending 0.14

The current working tree over `ba1cff6`, including the incoming 0.14 cuts, now
owns `CargoFeatureSelection` in `cargo::selection`. Its `all_features` flag joins
explicit names and default-feature selection there; observations and builds use
one encoder. No old observation-path alias remains. The existing typed recipe
argument bytes are preserved, including flag order and empty-feature rendering.
Real Cargo metadata and tree cases qualify explicit and all-feature graphs.

`batch::BatchLabelError` is the canonical typed empty/duplicate label failure.
Artifact batches return it directly; Wasm contract errors use `Labels(error)`
and expose that value through `Error::source`. Cargo grouping returns the same
downcastable error. Its generic diagnostic wording replaces Cargo/artifact-specific
wording; Wasm keeps its contextual prefix. Labels still reject before callbacks,
progress, source validation or command effects. No lease/preparation error is
collapsed into a label failure, and individual failures retain their batch entries.

`artifacts::WasmBuildPhase`, owned in `artifacts/phase.rs`, now serves command
errors, progress heartbeats and failure details. Progress/failure enum aliases and
translation are absent. Exact/shared lock phases use `ExactCacheCoordination` and
`SharedTargetCoordination`. Identity failures retain `CargoIdentity` or
`RustcIdentity`; both still contribute to the aggregate `tool_identity` timing.
Specification, publication, maintenance and cleanup phases remain distinct.

Actual Rust 1.91 passes all-target/all-feature compilation, strict Clippy,
warning-denied rustdoc, 43 Cargo cases, 13 Wasm-batch cases, six artifact-batch
cases, 64 Wasm-cache cases, 24 real native/Wasm integration tests and both README
examples. Synthetic producer failure/race cases supplement actual Cargo builds.
The new regressions cover distinct tool phases in silent/observed batches,
aggregate timing preservation and typed label causes. Formatting, documentation
links and the unchanged shared snapshot are checked separately. Full release
gates and native macOS qualification remain unrun.

[Build #54](https://github.com/dragginzgame/ic-build/issues/54) records the local
cut; [Canic #530](https://github.com/dragginzgame/canic/issues/530),
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307) and
[Testkit #94](https://github.com/dragginzgame/ic-testkit/issues/94) retain consumer
adoption. Their normal checkouts remain unchanged. Pending notes stay 0.14.0;
own version stays 0.13.0, and the incoming registry Host 0.25.1 lock, index and refs
remain untouched. No upstream Host API or retained format/namespace changed.
Before-images and focused logs: `/tmp/ic-build-model-cuts-20261017/`.

## Typed Cargo recipes, direct Host streams and shared boundary facts — pending 0.14

The current working tree over `ba1cff6`, including the incoming 0.14 evidence and
batch cuts, now replaces the partial Cargo CLI parser with public
`cargo::selection::CargoSelection`. Wasm recipes use `with_cargo_selection`;
native executable recipes use `selection`. Metadata and builds render the same
feature/lock/network choices. Reuse `CargoFeatureSelection`; represent frozen
policy with both `locked` and `offline`. Profiles remain explicit names, while
package, target, output and message-format choices stay structural. Native builds
apply mandatory locked operation before rendering, preventing duplicate flags.
There is no raw-argument compatibility path. Default keys and canonical
`--release --locked` argument bytes remain unchanged; normalization of other old
spellings/orderings may select new keys, without deleting retained entries.

Cargo output progress now carries `ic_host_process::tool::OutputStream` directly.
The per-validation fact cache lives in `artifacts/inputs/boundaries.rs`; transaction
exclusion-first lazy policy and shared-target eager input admission remain separate.
Both retain their original typed errors and pre-effect ordering. Shared-target
validation reuses the admitted directory types, retaining original input spellings
when constructing generated-directory candidates and observing every candidate.

Actual Rust 1.91 passes all-target/all-feature compilation and strict Clippy,
warning-denied rustdoc, 63 Wasm-cache units, typed/native/boundary regressions,
the generated-build-script observation case, 24 affected integration tests and both
README examples. Real native and Wasm Cargo fixtures complement synthetic producer
failure/race coverage. The lock stays byte-identical to the incoming Host 0.25.1
selection. One generated-source fixture now uses positional formatting to avoid
Rust 1.91 Clippy misreading its nested captured-format braces; behavior is unchanged.
Full release gates and native macOS qualification were not run.

Canic's recipe adapters, IcyDB's testing build adapter and Testkit's benchmark
recipes still use the old published API. Migrate after 0.14 publication through
[Canic #530](https://github.com/dragginzgame/canic/issues/530),
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307) and
[Testkit #94](https://github.com/dragginzgame/ic-testkit/issues/94).
[Build #47](https://github.com/dragginzgame/ic-build/issues/47) records this local
boundary consolidation and the still separate Host publication work. Earlier
frozen Host caller patches below remain receipts for their recorded source;
these new imports change their application context, so refresh and requalify them
before registry adoption. No pending Host API is imported into normal Build.
The incoming source/index/refs, own version and lock are preserved. Before-images
and focused logs: `/tmp/ic-build-typed-selection-20261017/`.

## Fresh input validation and one batch API — pending 0.14

Over `ba1cff6` and the incoming registry Host 0.25.1 lock, input validation now
accepts a post-work specification, observes original and selected roots together,
and returns the selected inventory. Already observed roots are not rehashed;
projection preserves existing path/value and digest bytes, including missing
inputs and explicit roots beneath excluded child names. Dropped original roots
still receive validation and newly discovered bytes gain no retroactive authority.
Independent batches, sessions and prepared snapshots each retain one entrypoint
with explicit configuration and optional progress. Five wrapper variants retire
without aliases. Existing callers and the API guide migrate together.

Actual Rust 1.91 passes all-target/all-feature compilation, 20 input and 12 batch
units and 28 affected integration parents; strict current Clippy and floor
warning-denied rustdoc pass. Links, snapshot and changed-file formatting pass.
Source custody, release admission, diagnostics and feature isolation remain with
their existing owners. [Canic #530 — Converge build-input reuse](https://github.com/dragginzgame/canic/issues/530#issuecomment-6233192667)
records the post-publication migration; normal Canic is unchanged. These public
cuts require a new pre-1.0 minor, so the notes select 0.14.0; own versions stay
0.13.0. Full release/native macOS and published consumer acceptance remain separate.

[Host PR #179](https://github.com/dragginzgame/ic-host-tooling/pull/179),
`a29281c`, adds single-attempt owned-directory cleanup from public `4086a17`.
It complements the existing allocator in PR #135 (`b1d7c71`), without a second
allocator implementation. The frozen Build staging-wrapper deletion passes 18
custody units, six Cargo integration parents, floor compilation and strict lint;
Host's floor passes 14 directory-filter cases as an ordinary user. The patch and
publication boundary are attached to [Build #45 — Directory allocation](https://github.com/dragginzgame/ic-build/issues/45#issuecomment-6233192457).
Normal Build retains its published staging API until both Host APIs are published;
no dependency override or guessed minimum is installed. The incoming lock and
root index/refs remain untouched. Logs and before-images:
`/tmp/ic-build-nextcuts-20261017/`.

## Combined Host caller cuts — source qualification, not registry adoption

The combined deletion over Build `ba1cff6` uses upstream shared descriptor
retention, logical directory size and newline framing directly. Cargo JSON/text
and EOF policy stay local. The candidate passes read-only apply checking against
normal source; [Build #54 — Retire the local shared-lock guard](https://github.com/dragginzgame/ic-build/issues/54#issuecomment-6232843798)
contains the full patch and source receipt, also covering #42 and #47.

The isolated Host source is public `4086a17` plus the producer changes from open
PRs #173 (`700b40b`), #174 (`74ef1db`) and #172 (`2de898dd`). Actual Build
Rust 1.91 passes all-target/all-feature compilation, 16 focused units and seven
integration parents with explicit workers. Strict current Build and affected
Host Clippy pass. Host Rust 1.88 passes two size, eight lock and six framing units.
Synthetic Wasm producers qualify orchestration, not native Wasm compilation.

All 48 non-Host lock entries remain exact in qualification; only the three
coherent Host source identities change temporarily. Normal source, manifests,
index/refs and the incoming registry 0.25.1 lock remain unchanged. The selected
registry packages lack these APIs: reviewed integration/publication, actual first
published FS/Process minima and normal registry caller qualification still gate
activation. No pending imports or source overrides are installed in normal Build.
Full/native macOS validation and consumer migration remain separate. Evidence:
`/tmp/ic-build-hardcuts-followup-20261017/`; caller patch SHA-256:
`89bdfdb5f2fcd1bbcaa3c5b60b29cf2d9ea01ace005a3dd69d4ef061ea460849`.

## Host 0.25 admission caller repair — pending 0.13

The incoming `2f4783a` selects registry Host 0.25.0 but retains five old tool
admission calls in the transaction example and Host/Candid fixtures. Those calls
now pass the expected version separately from `VersionSpec`, preserving their
exact versions, digest authority, executable bounds, environment and deadlines.
No compatibility adapter, dependency selection or own version change is added.
The existing pending 0.13 notes now cover the exposed Host minor-family boundary.

The reported `make check-package` gate passes. Actual Rust 1.91 all-target/all-
feature compilation and all 11 affected Host/Candid cases pass on Linux; strict
current all-target/all-feature Clippy, changed-file formatting, documentation
links, snapshot and diff checks pass. Logs: `/tmp/ic-build-host025-admission-20261017/`.
Full release validation and native macOS qualification remain separate. Root
index/refs, manifests and the selected lock remain unchanged.

## Canic caller PR, producer contributions and issue acceptance

[Auth PR #68](https://github.com/dragginzgame/ic-auth/pull/68), head `9309204`,
adopts published Build 0.12.1 in private native tooling and deletes the duplicate
yq/jq external-lock comparison. It starts at public `8ef596db`, excluding the
normal checkout's unpublished batch. No-follow bounded reads retain an explicit
8 MiB per-lock allowance; Auth retains package/feature/dependency and release
policy. All 368 original lock identities/sources/checksums remain selected; ten
packages are added for the tooling edge, while public library graphs exclude
Build. Actual Rust 1.91 build, seven native tooling cases, package-consumer
refusal fixtures and dependency boundaries pass, with strict all-target tooling
Clippy, formatting, shell lint, dependency inheritance, links and snapshot checks.
Fresh Linux/native macOS CI is running; archive/MSRV-matrix acceptance remains
separate. Auth #66 stays open for review/integration and consumer acceptance.

[Canic PR #593](https://github.com/dragginzgame/canic/pull/593), head `5181a159b`,
is ready for review. Canonical-entry reporting now includes the minimal Testkit
public-field correction needed to compile against pushed `53f1f22`. Actual Rust
1.91 library checking and strict current library Clippy pass; the exact-source
pending-Build projection harness retains its ordering/phase/timing coverage.
Manifests and lockfile are unchanged, and the normal unpublished integration
batch remains excluded. Fresh hosted CI is running; PocketIC/native Wasm and
published Build 0.13 consumer adoption remain separate.

Build [PR #53](https://github.com/dragginzgame/ic-build/pull/53) (`5cd31f7d`) and
[PR #55](https://github.com/dragginzgame/ic-build/pull/55) (`6ead7812`) are reconciled
locally into pending 0.13. Producer/log admission now retains successful output
before producer cleanup, leaving tools, callbacks and failure recovery local.
Native Cargo execution requires explicit child spawning and cancellation during
both metadata and compilation; the README caller migrates with that API cut.
Actual Rust 1.91 passes 18 staging cases, five native units and the real Cargo
executable integration. Strict all-target Clippy, MSRV all-target compilation,
warning-denied floor rustdoc, formatting, links and snapshot checks pass.
No PR merge, root commit, own version change or publication has occurred.

[Host PR #173](https://github.com/dragginzgame/ic-host-tooling/pull/173),
`700b40b`, [Host PR #174](https://github.com/dragginzgame/ic-host-tooling/pull/174),
`74ef1db`, and [Host PR #176](https://github.com/dragginzgame/ic-host-tooling/pull/176),
`2b1eea8`, now pass Linux, both native macOS architectures and actual MSRV.
PR #176 targets integration PR #172's HTTP fixture race. All remain open;
review/integration/publication still gate Build's filesystem/lock/framing caller
deletions. Build #42, #47 and #54 record the source-bound hosted evidence.

Build #25 is closed as producer-complete: registry 0.12.1's VCS receipt names
`1836512`, archive SHA-256 agrees with the selected checksum, and exact release
[CI](https://github.com/dragginzgame/ic-build/actions/runs/39013061639) passes Linux,
both native macOS hosts and lint/security. Blob #69 and Auth #66 retain consumer
acceptance; #54 and #47 retain pending cuts and Host publication/adoption.
Task before-images, source identities and logs: `/tmp/ic-build-next-20261017/`
and `/tmp/ic-build-canic-followup-20261017/`. Only this handoff changed in normal
Build during the consumer follow-up; incoming source, root index/refs, versions
and selected registry graph remain unchanged.

## Batch views, Unix archive admission and upstream lock guard — pending 0.13

Over released `1836512`, batch success/failure iterators now borrow their stored
entries, while integrated shared-target maintenance returns `(entry, outcome)`
pairs. The five separate result-view types have no compatibility aliases.
Labels, order, collect-all failures, partial phase timings and retained records
stay intact; pointer-identity assertions cover canonical iterator borrowing.
Tests and API-guide examples migrate together. Archive qualification now lives
behind one Unix module gate with Host's descriptor-based source traversal;
the distinct non-Unix recursive walker is removed. The pure external-lock
subset verifier and its structural admission tests remain platform-independent.

On actual Rust 1.91/Linux, 29 selected batch/package units and 28 retained-handoff,
Cargo-example, archive and Wasm integration parents pass, including explicit
process workers. Strict all-target Clippy, MSRV all-target compilation,
warning-denied floor rustdoc, formatting, links and snapshot verification pass.
Synthetic Wasm producers qualify orchestration, not native Wasm compilation.

[Host PR #174](https://github.com/dragginzgame/ic-host-tooling/pull/174), head
`74ef1db`, extracts the existing explicit-unlock guard from pushed `2f3976d`.
Eight descriptor-lock cases, all-target checking and warning-denied rustdoc
pass on actual Rust 1.88/Linux, with strict current Clippy, formatting and links.
The frozen Build caller deletion passes ten cache-filesystem units, six retained-
handoff/cache-process integration parents and explicit workers on Rust 1.91,
plus strict all-target Clippy. Only three coherent Host source identities change;
all unrelated lock identities/checksums remain fixed. Build retains its existing
guard and registry graph until reviewed Host publication permits direct adoption.
[Build #54](https://github.com/dragginzgame/ic-build/issues/54) retains that
publication boundary and the tested caller patch, SHA-256
`6e63ca055a7987287bbdd8b8e63363e4ba3be96bd23caa36a6679865a6ff43c4`.

Own versions, root index/refs, the selected lock and normal sibling checkouts
remain unchanged. Full release validation, native macOS and published consumer
adoption remain separate. Sources and logs: `/tmp/ic-build-cuts-20261017/`.

## Shared Tooling snapshot repair

The 2026-10-17 release validation rejected local API prose added to the
snapshot-owned `docs/cargo-publication.md`. That guide now matches the adopted
0.15.2 digest exactly; the prose lives in the local
[artifact API guide](../artifacts.md#external-lock-selection-admission), linked
from README. `make shared-tooling-check` verifies all 101 snapshot files.
This documentation repair does not establish completion of release validation.

## Logical directory-size producer delivery — Host PR #173

The previously prepared filesystem extraction for
[#42](https://github.com/dragginzgame/ic-build/issues/42) is now delivered as
[Host PR #173](https://github.com/dragginzgame/ic-host-tooling/pull/173), head
`374506e`, from pushed Host `2f3976d`. It excludes the normal Host checkout's
unpublished changes. Host's read module gains metadata-only logical-size
observation with the existing Build walker and two filesystem regressions;
cache selection, pruning, resource admission and lifecycle stay with consumers.
Directory entries contribute no metadata bytes, sparse files contribute logical
lengths, final symlinks contribute their own lengths, totals saturate, and native
I/O errors propagate. Namespace custody remains caller-owned.

Both Host cases pass on actual Rust 1.88/Linux, with all-target/all-feature FS
checking, strict current Clippy, warning-denied floor rustdoc, formatting and
links. A frozen current Build rehearsal removes its walker and duplicate generic
tests, switches callers directly to Host and passes 109 cache/pruning/transaction/
Wasm-cache tests on actual Rust 1.91. Caller strict all-target Clippy and MSRV
all-target checking pass. Only the three coherent Host source identities differ
in that temporary graph; all other external selections/checksums remain intact.
These tests are native filesystem/process evidence; synthetic Wasm producers do
not establish native Wasm compiler or complete application qualification.

Normal Build still uses its existing implementation and published dependencies.
The apply-checked caller patch omits temporary overrides and requires reviewed
Host publication plus its actual dependency minimum before activation. It is
retained in #42 with SHA-256
`2c767344c5be0610291a71bfeefe512e79c631ef7693932e57a69964da9032ba`.
Required hosted/native macOS checks and maintainer integration remain separate.
Root's pending lock-subset source, incoming graph, versions, index/refs and all
normal sibling checkouts are preserved. Source snapshots, raw checks and graph
deltas remain under `/tmp/ic-build-logical-size-20261017.Zpx7LU/`.

## Shared external-lock subset admission — pending 0.12.1

This batch extends the incoming documentation work over released `2598a41` and
preserves the incoming libc 0.2.191 lock selection byte-for-byte. The pure
`cargo::packages::verify_external_lock_subset` API exposes the existing archive
engine's external-selection check with an explicit per-document byte bound and
typed errors. Parser/comparison ownership is shared; archive qualification keeps
its independent nonempty-external-graph refusal. The generic API accepts empty
subsets and extra valid original selections. Both reject malformed records,
duplicate identities and changed optional checksum fields, including Git rows;
valid registry checksums retain their canonical lowercase SHA-256 admission.
Unsourced identity, feature/dependency policy, source custody and release
admission stay with consumers. No process, filesystem or network effects occur.

On actual Rust 1.91/Linux, all 11 selected package units and the real Cargo archive
qualification integration pass. Strict all-target/all-feature Clippy, actual
MSRV all-target compilation, warning-denied MSRV rustdoc, formatting and links
pass. The archive case qualifies native Cargo packaging and retained evidence,
not an application release or synthetic Wasm compiler proof.

Auth's existing native tooling command is rehearsed in a disposable snapshot
of `18b1850` plus incoming source. Its script deletes the yq/jq lock comparison
and delegates to Build while retaining Auth's metadata/library policy and explicit
8 MiB file allowance. Actual Rust 1.91 tooling build, strict all-target tooling
lint and package-consumer fixtures pass, including duplicate/malformed lock
refusals. All original package identities/checksums remain selected; only the
candidate Build source and missing dependencies are added. The normal Auth
checkout is unchanged and has no unpublished import or source override.

[Build #25](https://github.com/dragginzgame/ic-build/issues/25) retains producer
publication and archive acceptance; [Auth #66](https://github.com/dragginzgame/ic-auth/issues/66)
owns published caller adoption and carries the apply-checked source patch.
Full release/native macOS gates and real registry adoption remain separate.
Own versions, the Git index/refs and retained artifacts are unchanged. Before-
images, the frozen producer/consumer, exact graph delta and logs remain under
`/tmp/ic-build-lock-subset-20261017.gihOWp/`.

## Released 0.12.0 acceptance and registry adoption — pending 0.12.1 docs

Registry 0.12.0 is verified at source
`2598a417a96834d3139751adf549126add666c4b`. The cached sparse-index checksum
and archive SHA-256 agree on
`37ce052cd95c403415a36fafbe33c3ea872fcef02a29a65b871bfd2effcde425`;
archive VCS metadata and the remote annotated `v0.12.0` tag identify that source.
Its exact [release CI](https://github.com/dragginzgame/ic-build/actions/runs/38947150972)
passes Linux, native macOS 15 Intel/Apple Silicon and lint/security. Producer
issues [#49](https://github.com/dragginzgame/ic-build/issues/49),
[#50](https://github.com/dragginzgame/ic-build/issues/50) and
[#51](https://github.com/dragginzgame/ic-build/issues/51) are closed with that
receipt. [PR #52](https://github.com/dragginzgame/ic-build/pull/52) is closed as
superseded without merging or deleting its historical branch. The prior pending
entry below retains development evidence; its unpublished/native blockers are
resolved by this exact release, separately from application acceptance.

IcyDB [PR #369](https://github.com/dragginzgame/icydb/pull/369), head `bacd69be0`,
adopts registry Build 0.12.0/Host 0.24.0 and moves the extraction import/removes
the retired Tools feature. It starts at pushed `27035c5`, excluding the normal
checkout's active Memory/schema work. All unrelated external package identities
remain unchanged, including that base's Memory 0.44.0. Build-helper all-target
checking, strict Clippy and the installed real-extractor artifact/publication
control pass; formatting and diff checks pass. The PR joins existing 0.277 notes
without changing own versions or adding source overrides. Hosted CI cannot fetch
the existing private IC Wire pin before compilation, reproduced on unchanged
main and the PR. The commit exists; runner authentication is tracked in
[IcyDB #370](https://github.com/dragginzgame/icydb/issues/370). Required native
acceptance and maintainer integration remain separate; no IC/PocketIC lifecycle ran.

Canic's current local Memory fixes allow `canic-host --lib` compilation with
registry Build 0.12.0/Host 0.24.0. This uses a frozen snapshot of local `93c69ada7`
plus incoming edits, not pushed main `53f1f22`; no dependency substitutions or
normal-checkout mutations are made. Strict Host library Clippy, all 16 focused
declaration/cache tests and the explicitly selected installed real-extractor reuse
test pass. The manifest/lock bytes remain unchanged. These cover the current
cache/process integration, including failed/changed inputs, optional cache I/O,
corruption, batching and cold/warm output parity. Canic still owns its direct
extractor invocation; adopting Build's admitted-tool extraction API remains
separate from these results. No installed release, full/native macOS or lifecycle
acceptance is claimed. Build
[#9](https://github.com/dragginzgame/ic-build/issues/9) retains consumer acceptance.

README, the artifact guide and ownership design now describe the published
0.12/Host 0.24 boundary. Own versions, this repository's index/refs and normal
sibling checkouts remain unchanged. Evidence and frozen sources are retained at
`/tmp/ic-build-next012.hjDv0F/`; root before-images are in its `before/` directory.

## Host 0.24 and native Candid extraction — pending 0.12.0

This batch starts at released 0.11.1 `4fa4fc0`, preserving the incoming
Host 0.24.0 manifest/lock selection byte-for-byte. The exposed Host type change
requires a 0.12 boundary; own package versions remain unchanged. Build now calls
published Host directory custody/removal and fixed-buffer digest encoding
directly, removing its local guard/remover and per-file evidence hex allocation.
Retained fingerprints, cleanup policy, source leases and publication stay intact.

The native Candid implementation and eight tests from
[PR #52](https://github.com/dragginzgame/ic-build/pull/52), head `c2322a5`, are
reconciled into this working tree over Host 0.24. Host has already removed its
former extraction module/feature. Build retains raw bounded output, admitted
source/tool identities and post-success source-change/reinspection failures.
Consumers retain tools, budgets, environment, normalization, grammar and custody.
The original PR's green Linux/both-macOS CI qualifies its original head only;
its branch remains conflicting and unmerged. This local reconciliation is not
a PR merge or a published API.

On Linux, actual Rust 1.91 passes 139 focused units and six integration parents
(including explicitly dispatched process workers), covering native extraction,
staging, transaction cleanup, Wasm-cache cleanup, retained cache handoff and
real Cargo depfile evidence. All-target MSRV checking, strict all-target current
Clippy, warning-denied MSRV rustdoc, formatting and documentation links pass.
Synthetic Wasm producers do not establish native Wasm compiler qualification.

Disposable consumer snapshots use this frozen Build source and registry Host
0.24, preserving all unrelated external package selections. IcyDB `27035c5`
passes build-helper all-target checking, strict lint and its real-extractor
artifact/publication test after the import/feature move. Testkit `34eb530`
passes its `canister_benchmark` caller check; Query `69881eb` passes its
`governance_smoke` example check. These last two are compile-only evidence;
no PocketIC or IC lifecycle ran. Canic `53f1f22` (including its incoming Host
0.24 selection) fails first in Core's unchanged Memory 0.45 pool/policy/diagnostic
calls; no Canic acceptance or dependency substitution is claimed.

Publish Build's aligned owner before consumers switch to it. Full release and
combined-source native macOS gates remain unrun. GitHub
[#49](https://github.com/dragginzgame/ic-build/issues/49),
[#50](https://github.com/dragginzgame/ic-build/issues/50),
[#51](https://github.com/dragginzgame/ic-build/issues/51) and
[#9](https://github.com/dragginzgame/ic-build/issues/9) retain delivery/acceptance.
Before-images, frozen source, exact consumer lock deltas, focused logs and the
IcyDB import/feature patch are under `/tmp/ic-build-host024.Fasjsb/`.
Sibling source/index/refs and this repository's index/refs are untouched.

## Manifest comparison, batch finalization and argument formatting — pending 0.11.1

This compatible batch extends incoming changes over released 0.11.0 `dff188f`.
Cold publication and cache-hit comparison share one byte encoder; the latter uses
Host's published MatchingWriter instead of allocating another full manifest.
Bounded reads and the early header check remain before output inspection. Short,
altered and trailing retained bytes refuse, and encoder sink errors propagate.
The existing batch finalizer owns source-change invalidation for silent and
observed acquisitions, sampling total elapsed time after invalidation as before.
Cargo feature and ordinary Rust-flag strings are built without temporary join
vectors; canonical order, empty names/arguments and environment precedence remain.

On actual Rust 1.91, 51 focused units and three integration parents pass, including
the explicitly dispatched process workers and six isolated Cargo flag scenarios.
Evidence covers retained manifest bytes/bounds, truncation, encoder write failure,
cross-process cache reuse, silent session invalidation and observed prepared-reader
invalidation during actual Wasm compilation. Synthetic Wasm batch producers remain
separate from those actual Cargo/Wasm cases. Actual MSRV all-target checking and
strict all-target Clippy pass after splitting the expanded flag fixture's parent
driver and correcting its test-only lint findings. Formatting and links pass.
Full release/native macOS gates remain unrun for these unpublished changes.
Own versions, selected graph, Git index/refs, unrelated edits and artifacts are
preserved. Task before-images remain under
`/tmp/ic-build-manifest-finalizer-20261016.BjWKCT/before/`.

## Batch discovery, archive identities and projection indexing — pending 0.11.1

This compatible batch extends the incoming working changes over released 0.11.0
`dff188f`. Batch discovery populates its success list and indexed failure slots
directly within the existing discovery phase, removing the intermediate result
vector while retaining collect-all order, independent failures and timing ownership.
Archive verification retains Host's binary digest until writing evidence; archive
hash lines and source-inventory JSON keep their original hexadecimal representation.
Lockfile version projection indexes selected names and unsourced package rows once,
then applies changes in caller order and checks matching rows in original order.
Missing/ambiguous selection, prior-version and parse refusal precedence is preserved.

On actual Rust 1.91, 31 focused units and the real Cargo archive qualification
integration pass. Coverage includes malformed/shared metadata, mixed batch results,
session/prepared reuse, complete fresh/batched snapshot parity, retained fingerprint
bytes, lockfile error precedence and exact archive/source evidence. Package and
lockfile fixtures execute native Cargo; synthetic Wasm producers qualify batch
mechanics without establishing native Wasm compiler coverage. Strict all-target
Clippy, actual MSRV all-target checking, formatting and links pass on Linux.
Full release and native macOS gates remain unrun for these unpublished changes.
Own versions, dependency selection, index/refs, unrelated edits and artifacts are
preserved. Task before-images remain at
`/tmp/ic-build-batch-lock-digests-20261016.rdgLxg/before/`.

## Delivered issue/PR reconciliation and consumer guidance — pending 0.11.1

Registry 0.11.0 is verified from `dff188fe27136054a91044900ae1cd607669d870`:
the sparse index and downloaded archive agree on SHA-256
`260e3b9871c838623dfd6b85554274289a981d1c11e1087752560349c679d122`, and its
VCS receipt names that source. The exact
[release CI](https://github.com/dragginzgame/ic-build/actions/runs/38916316699)
passes Linux, native macOS 15 Intel/Apple Silicon and lint/security. The maintained
producer implementations and tests satisfy issues
[#22 — Caller-bounded input probes](https://github.com/dragginzgame/ic-build/issues/22),
[#28 — Strict lockfile projections](https://github.com/dragginzgame/ic-build/issues/28),
[#30 — Native Cargo executable selection](https://github.com/dragginzgame/ic-build/issues/30),
[#39 — Shared release preflight](https://github.com/dragginzgame/ic-build/issues/39) and
[#40 — Relocatable source fingerprints](https://github.com/dragginzgame/ic-build/issues/40); they are closed with
release-bound evidence and separate consumer trackers. Conflicting PRs
[#31](https://github.com/dragginzgame/ic-build/pull/31) and
[#48](https://github.com/dragginzgame/ic-build/pull/48) are closed as superseded,
without merging or deleting their branches.

README and the artifact guide now select published Build 0.11/Host 0.23 and
distinguish this delivered producer matrix from consumer/application acceptance.
The current unpublished 0.11.1 source below retains only its focused Linux proof.
This documentation/issue pass preserves those incoming Rust edits, own versions,
the selected dependency graph, index/refs and artifacts. Documentation links and
diff checks pass; no compilation or release runs for these prose-only changes.

## Canonical labels, environment collection and shared inventories — pending 0.11.1

This compatible batch starts from clean release commit `dff188f` (0.11.0).
Transaction validation checks adjacent canonical labels instead of building three
temporary sets; output lookup uses binary search over the admitted sorted list.
Input/tool labels and destination uniqueness retain their separate checks, and
per-entry validation/error ordering is preserved. Effective environment collection
populates its final ordered map directly, observing each unique ambient name once
and retaining explicit override precedence, unset and OS-native values.
Acquisition record clones share immutable artifact inventories through Arc slices;
public slice accessors, equality, Debug lists and last-owner retention remain intact.

On actual Rust 1.91, 40 distinct focused units and four process handoff integration
parents pass, including their dispatched consumer workers. Evidence covers sorted
duplicate admission, known/unknown output lookup, environment identity, retained
fingerprints, corruption refusal, clone equality/Debug and cross-process pruning
through final drop or consumer termination. Synthetic Wasm producers exercise
native process/filesystem mechanics, not native Wasm compiler qualification.
Actual MSRV all-target checking, strict all-target Clippy, formatting and links
pass on Linux. Full release gates and native macOS qualification remain unrun.
The compatible draft is 0.11.1; package versions, dependency graph, Git index/refs
and retained artifacts are unchanged.

## Snapshot assembly, JSON scratch and Debug projections — pending 0.11.0

This compatible cleanup extends the existing dirty work over `95567eb` and its
selected registry Host 0.23.1 graph. Fresh and batched Cargo resolution share one
private snapshot assembler. The fresh timing callback runs after path conversion;
batch attribution still charges one successful entry and leaves peers at zero.
Fresh input discovery and mutation guards remain independent. Package projection
reuses one JSON byte buffer across raw fields, retaining exact compact encoding,
missing/null distinctions and framing. Redacted Debug lists format borrowed
iterators directly through a private Debug adapter; values stay hidden and both
ordinary/pretty formatting retain the former list representation.

On actual Rust 1.91, 20 focused units and eight real Cargo integration parents
pass. Coverage includes complete fresh/batch snapshot equality apart from timing,
the retained raw-JSON fingerprint vector, semantic dependency changes, independent
batch errors, Debug redaction/layout, named examples, concurrent prepared readers
and transactional source guards. Strict all-target Clippy, actual MSRV all-target
checking, formatting and links pass on Linux. Full release and native macOS gates
remain unrun. Own versions, manifests/lock, index, incoming edits and retained
artifacts are preserved. Before-images remain under
`/tmp/ic-build-snapshot-assembly-20261016.3hcztc/`.

## Artifact collection, diagnostics and incoming Host repair — pending 0.11.0

This batch starts over `95567eb` plus the existing cleanup work and an incoming
manifest/lock selection of registry Host 0.23.1. That graph initially fails at
four retired APIs. The required callers now explicitly preserve `usize::MAX`
missing-target symlink depth, open/no-follow plus nonblocking exclusive locking,
caller-selected probe cleanup and observed-build KillAndWait group cleanup.
The example's owned group selects the same explicit policy. Incoming manifest
and lock bytes are preserved. Exposed Host types make the combined draft 0.11.0;
earlier 0.10.1 draft evidence below belongs to this pending minor release.
Own package versions and Git operations remain user-owned and unchanged.

Cargo artifact collection finds the unique admitted target, then moves its owned
filename while retaining exact package/kind/name matching, absolute-path checks,
ambiguous filenames and duplicate-record refusal. Error-list formatting streams
directly with unchanged text, ordering, Unicode and empty-list output. Shared
target boundaries fold generated roots into a boolean, consuming every candidate
and keeping all resolved-input admission eager before considering generated roots.

On actual Rust 1.91 with the incoming Host graph, 58 distinct focused units and
eight integration parents pass, including their explicitly dispatched process
workers. Evidence covers protocol refusal after child completion, lock contention
and redirection, source drift, retained outputs, cancellation/cleanup, real Cargo
examples and custom multi-package library paths. Strict all-target Clippy, actual
MSRV all-target checking, formatting and links pass on Linux. Full release gates
and native macOS qualification remain unrun. Before-images remain under
`/tmp/ic-build-artifact-formatting-20261016.Bz83PK/`.

## Borrowed include paths, tree roots and artifact directories — pending 0.10.1

This compatible batch extends incoming dirty work over `95567eb`. Configuration
include parsing borrows paths from the decoded TOML; absolute paths stay borrowed
and relative paths allocate only their resolved lookup path. All include entries
are still validated before recursion. Tree parsing borrows its current root name
from captured text while retaining owned map keys and normalized tree contents.
Wasm output discovery chooses the common profile/example directory once per set,
preserving target names, paths and retained cache identities.

On actual Rust 1.91, 17 focused units and two real Cargo/Wasm integrations pass.
Coverage includes absolute/relative/optional includes, distinct aliases, malformed
include refusal before missing-file reads, feature-safe grouping, profile bounds,
library/example selection, named-example reuse and distinct custom-library outputs.
Strict all-target Clippy, actual MSRV all-target checking, formatting and links
pass on Linux. Full release and native macOS gates remain unrun. Existing edits,
own versions, registry graph, index and retained artifacts are preserved. Task
before-images remain under `/tmp/ic-build-borrowed-paths-20261016.8E5J8r/`.

## Cargo package selection — pending 0.10.1

This compatible batch extends the incoming dirty work over `95567eb`. Parsed
Cargo contexts retain a borrowed workspace-name index instead of a membership
set used for repeated package scans. Missing and ambiguous names remain distinct,
nonmembers cannot shadow workspace members, duplicate member IDs are deduplicated,
and requested order is preserved. Ambiguity is refused only when that name is
selected. Native executable selection consumes decoded metadata and moves its
package ID and target directory. Archive qualification checks its first two
matching packages without allocating a match vector. Public APIs, selected
dependencies, fingerprint inputs and refusal diagnostics stay unchanged.

On actual Rust 1.91, 32 focused units and three real Cargo integrations pass:
native custom-profile execution and failed-build diagnostics, package archive
qualification, and isolated/shared Wasm cache publication/reuse. Units cover
selection order, membership, absent/ambiguous identities, interleaved feature
contexts, standalone fingerprint parity and independent batch failures. Strict
all-target Clippy, actual MSRV all-target checking, formatting and documentation
links pass on Linux. Full release and native macOS gates remain unrun. Own
versions, index, registry graph and existing artifacts are preserved. Task
before-images remain under `/tmp/ic-build-package-selection-20261016.UJNxCH/`.

## Batch grouping, depfile accumulation and stamp bounds — pending 0.10.1

This compatible batch extends the incoming dirty work over `95567eb`. Batch
resolution indexes complete context keys without scanning prior groups; the
group/member vectors retain first-seen order. Workspace, tools, metadata arguments
and effective environment remain part of equality. Depfile readers insert into
the caller's canonical-path set directly, preserving complete syntax admission
before filesystem inspection and refusing the entire discovery on any error.
Wasm stamp reads calculate the exact bound from the existing header, shared
artifact label, digest width and newline, without rendering a placeholder stamp.
The writer's retained format, fingerprints and cache namespaces stay unchanged.

On actual Rust 1.91, 37 distinct focused units and two real Cargo integrations
pass. Evidence covers interleaved feature contexts with one probe sequence per
context and standalone fingerprint parity; duplicate depfile inputs/records,
syntax refusal before symlink-loop inspection; exact 184-byte stamps, one-byte
overflow/truncation, reconstruction and publication failures. Strict all-target
Clippy, actual MSRV all-target checking, formatting and documentation links pass.
These are Linux checks; full release and native macOS gates remain unrun. Own
versions, registry graph, index and existing artifacts are preserved. Task
before-images remain under `/tmp/ic-build-batch-depfiles-20261016.WMrFga/`.

## Indexed lock identity and input allocation cuts — pending 0.10.1

This compatible batch extends the existing dirty 0.10.1 work over `95567eb`.
Semantic projection builds one borrowed index of external lock identities,
preserving the first exact name/version/source row, including absent checksums.
Package versions are formatted once per identity. Lock parsing still inspects
every row, and registry/git qualification, fallback and fingerprint fields stay.
Transaction boundary caches borrow path keys on hits and allocate keys on misses;
filesystem facts remain lazy and scoped to one validation call. Relative input
evidence appends validated path components into one String, preserving empty-root
encoding, slash separators, UTF-8 refusal and the required lexical sort.

On actual Rust 1.91, 55 distinct affected units and two real Cargo integrations
pass. Coverage includes duplicate lock rows, absent checksums and decoy identities;
exact absolute/relative digest encoding with empty-root, Unicode and absent paths;
fresh-call boundary failures, exclusions and transaction input drift. Strict
all-target Clippy, actual MSRV all-target checking and formatting pass on Linux.
Full release gates and native macOS qualification remain unrun. Existing edits,
own versions, selected lock, index and retained artifacts are preserved. Task
before-images remain under `/tmp/ic-build-indexed-inputs-20261016.SbHL7m/`.

## Borrowed grouping, snapshot metrics and fixed hex — pending 0.10.1

[#50](https://github.com/dragginzgame/ic-build/issues/50) owns this compatible
batch over the clean 0.10.0 release commit `95567eb`. Feature-safe grouping moves
the isolated tree out of its map and borrows candidate package names. Greedy
input/command ordering, exact feature-tree comparisons and typed failures stay.
Prepared reader metrics use the standard saturating atomic update with relaxed
ordering and matching-only increments. Rust 1.99 deprecates fetch_update, while
try_update is unstable on actual Rust 1.91; the one MSRV-supported implementation
has a scoped, explained deprecation allowance rather than a second code path.

All six Cargo grouping cases and a concurrent/saturating reader case pass on
actual Rust 1.91, together with the real concurrent prepared-reader integration.
Strict all-target Clippy, actual MSRV all-target checking and formatting pass.

[Host PR #163](https://github.com/dragginzgame/ic-host-tooling/pull/163), from pushed
main `798fd42`, adds fixed-buffer lowercase SHA-256 output and makes Display use
that sole encoder. Two focused cases on actual Rust 1.88 cover known vectors,
every byte at every digest position, leading zeroes, parser round trips and
existing formatting flags. Strict affected lint, Rustdoc, formatting and links pass.

A frozen Build caller removes the inventory hasher's per-file temporary String.
All 37 affected units, real Cargo depfile integration, strict all-target Clippy
and actual Rust 1.91 all-target checking pass. Four inventories with 256 varied
files, Unicode paths, absence and exclusions retain all eight absolute/relative
digest values against the released source. Ordering and big-endian field framing
stay unchanged. Only three Host registry identities become coherent temporary
path selections; every other lock selection stays. Normal Build keeps published
APIs until reviewed Host publication permits adoption. The apply-checked caller
patch has SHA-256
`ea532e8ac3c048823d0441ed179f06531593fac5168b5a7ec313e10ef253164e`.

Own versions, registry graph, index and retained artifacts are preserved. No full
suite, native macOS qualification, normal sibling mutation or release ran.
Before-images, the caller patch, frozen sources and focused evidence remain under
`/tmp/ic-build-borrowed-grouping-20261016.P9JwS3/`.

## Cargo artifact evidence and directory custody — pending 0.10.0

[#49](https://github.com/dragginzgame/ic-build/issues/49) owns this batch over
`60e00c2` plus incoming native/boundary/inspection work and the registry Host
0.22.1 selection. Six byte writers now explicitly preserve Replace/0666 under
the new Host signature; manifests, selected lock and own versions are untouched.
The exposed Host boundary and reserved Cargo message format select pending
0.10.0 notes, including the earlier unpublished compatible 0.9.1 work. Host
alignment is also tracked by its existing owner in
[#48](https://github.com/dragginzgame/ic-build/issues/48).

Fresh and batched resolution share one validation/progress completion sequence;
post-build and non-immutable reuse checks still resolve fresh inputs. Native and
Wasm builds share Cargo message decoding and exact package/target matching. Wasm
captures Cargo-reported source paths, admits custom cdylib names and derives
declared/retained paths from metadata-selected targets. Existing recipes retain
their path layout, namespaces, fingerprints and stamps. Target-name collisions
still refuse; custom multi-package names cannot overwrite another output. Wrappers
must emit artifact records and cannot override the engine's JSON message format.
The caller-adjustable per-message allowance defaults to 16 MiB. Protocol refusal
does not terminate Cargo: diagnostics drain and child/status/cleanup failures keep
precedence before publication is rejected. No build deadline is introduced.

All 75 selected Wasm/native/message/selection units pass on actual Rust 1.91.
Real Cargo integrations pass: 17 Wasm, six example, one native and four retained
handoff parents; ignored worker entrypoints run through their parents. Six frozen
default/dev/test/release/bench/custom recipes preserve exact fingerprint, input,
validation and Wasm SHA-256 values and reuse every incoming cache entry. Strict
all-target Clippy, actual Rust 1.91 all-target checking, warning-denied Rustdoc,
formatting and documentation links pass on Linux.

[Host PR #160](https://github.com/dragginzgame/ic-host-tooling/pull/160), from pushed
main `798fd42`, extracts directory custody and final-entry removal. Four producer
cases pass on actual Rust 1.88, including non-root failure/retry, with strict
affected lint/docs/format checks. A frozen Build caller deletes both local
implementations and passes 122 affected units, cache/retention process integrations
and strict all-target Clippy on actual Rust 1.91. Only the three Host identities
become temporary coherent path selections; every other locked selection stays.
Normal Build keeps published APIs until reviewed Host publication permits the
deletion. Its apply-checked patch has SHA-256
`28224dadefbaf3989fd2dc800ea6c978501d27dc75a977f82d8efa5329aa09b9`.

No full suite, native macOS qualification, normal sibling mutation or release ran.
Before-images, frozen sources, the caller patch and focused logs remain under
`/tmp/ic-build-cargo-custody-20261016.Bc30HA/`. Source overrides exist only in the
disposable consumer qualification; the normal registry graph remains selected.

## Transaction inspection allocations — pending 0.9.1

Over released `60e00c2` plus the incoming native/boundary/lock work, transaction
directory inspection now visits undeclared entries once. Cache-validity callers
retain only a boolean; staging collects the complete diagnostic list directly.
The scan still consumes every directory entry so later read failures remain
errors. Manifest admission computes the same maximum length with checked
arithmetic instead of allocating a placeholder document. The encoder, retained
format and public APIs are unchanged.

The proposed removal of the absolute input-evidence sort was rejected during
implementation: PathBuf orders components, while the retained consumer encoding
orders UTF-8 strings. The existing encoding regression already covers this
difference; explicit ordering assertions strengthen that proof. The required
sort and both absolute/relative digest implementations remain intact.

On actual Rust 1.91, all 32 transaction and 18 input-evidence unit cases pass,
along with the cross-process artifact-cache parent and real Cargo depfile
integration. The ignored process worker runs through its parent. Coverage
includes complete multi-directory staging diagnostics, malformed/oversized cache
entries and manifest bounds across decimal widths, both validation tokens and
multibyte names. Strict all-target Clippy, formatting and documentation links
pass. These are focused Linux checks, without full release or native macOS
qualification. Own versions, selected lock, real index and incoming work are
preserved. Before-images and focused logs remain under
`/tmp/ic-build-cache-cuts-20261016.VSdg77/`.

## Boundary observations, lock decoding and Host framing — pending 0.9.1

[#47 — Consolidate boundary observations, lock decoding and native framing](https://github.com/dragginzgame/ic-build/issues/47)
owns this approved batch over released 0.9.0 plus the incoming PR #31 working
source. Transaction admission now reuses successful Cargo input/exclusion path
observations only within that invocation. Reads remain lazy and exclusion-first;
later invocations re-observe the filesystem. Entry/referent checks and subsequent
Cargo-input revalidation retain their existing owners and custody requirements.

Fingerprinting and archive qualification share borrowed external lock-row fields.
Wasm keeps its optional checksum/fallback and original raw identity bytes; package
qualification keeps strict checksums, duplicate rejection, nonempty external graph
and subset acceptance. Full-document version projection remains separate. On actual
Rust 1.91, all 31 transaction, 65 Wasm-cache and seven package cases pass, together
with real Cargo archive qualification. Temporary comparisons match original rows
and error text on 2,453 documents for each reader. Six real Cargo/Wasm recipes retain
exact fingerprints, input/validation digests and artifact bytes; all reuse their
incoming-source cache entries. Strict all-target Clippy, actual MSRV all-target
checking, Rustdoc and formatting pass on Linux.

Existing [Host PR #128](https://github.com/dragginzgame/ic-host-tooling/pull/128)
is extended in an isolated topic from pushed main `a0a969a` with chunk-fed framing.
Six producer cases pass on actual Rust 1.88, with strict affected lint/docs and
all-target compiler-floor checking. A frozen Build candidate deletes its local
line-buffering loop and passes the four native protocol cases, a temporary caller
case covering EOF tails/overflow/parse/sink failures, and real native Cargo binary
and example execution on actual Rust 1.91. Only the three Host registry identities
become coherent temporary source selections; every other locked selection stays.

Normal Build retains published Host APIs until reviewed publication permits the
caller deletion and exposed-boundary qualification. The incoming native selector,
own versions, registry lock, real index and retained artifacts are preserved. No
full suite, native macOS qualification, release or normal sibling mutation ran.
Before-images, original failed checks, frozen sources, caller patch and focused
logs remain under `/tmp/ic-build-three-cuts-20261016.wnMuH7/`.

## Native executable selection and PR reconciliation — pending 0.9.1

Released 0.9.0 is `60e00c2593163a41109bd097306f1574ae10b7e3`. PRs
[#29](https://github.com/dragginzgame/ic-build/pull/29),
[#37](https://github.com/dragginzgame/ic-build/pull/37) and
[#41](https://github.com/dragginzgame/ic-build/pull/41) are closed as duplicate
proposals: their implementations are already delivered. Their producer issues
retain current-release/native acceptance until the complete current gate passes;
consumer adoption remains with its existing owners. Older successful source-bound
results are preserved rather than relabelled as current acceptance.

[PR #31](https://github.com/dragginzgame/ic-build/pull/31) is reconciled with
released main under compatible 0.9.1 notes for
[#30 — Select native Cargo executables](https://github.com/dragginzgame/ic-build/issues/30).
The Unix operation returns the exact selected package/target's Cargo executable,
using the existing canonical Cargo metadata/message schema and argument selection.
Build-only package/target/path/configuration overrides refuse before dispatch;
feature/network arguments reach metadata as well as build. Caller-owned command,
source exclusion, artifact custody and in-flight signal policy remain explicit.
Diagnostics stream, message buffers are caller-bounded and no default elapsed-time
deadline, installation, cache or release admission is introduced.

Four protocol/refusal cases and a real native Cargo integration pass on actual
Rust 1.91. The integration builds and runs both a binary and example with a custom
profile and spaced target path, checks refusal before the second dispatch and
retains typed compiler failure plus streamed diagnostics. Strict all-target Clippy,
Rustdoc and formatting pass on Linux. This is producer qualification, without
Testkit adoption, full release validation or native macOS acceptance. Own package
versions, selected lock and the real index remain unchanged. Source before-images,
PR comparisons, isolated topic and focused logs are retained under
`/tmp/ic-build-maintenance-20261016/`.

## Derived Cargo profile directories — pending 0.9.0

From released 0.8.0 `0a8b9667ff9e2d7927a6ca8961b7d463de396a77`, the Wasm
specification constructor takes workspace, target directory and package selection.
The existing Cargo argument selection derives the profile output directory;
replacing profile arguments updates artifact paths as well. Malformed/repeated
profiles and unsafe derived path components still refuse before tool execution
or acquisition. Raw argument bytes, fingerprint field encoding, retained v1
formats and cache namespaces remain unchanged for equivalent recipes.

This is a breaking constructor cut, so pending notes select 0.9.0. Callers must
remove the fourth constructor argument and keep profile selection in
`with_cargo_profile_args`. All in-repository callers and active guide examples
use the new API. Consumer adoption follows publication; Canic's artifact test is
a known external constructor caller. No sibling source was changed or qualified.
The package version and selected Host 0.21 graph remain at their incoming state.

Artifact identifier and label length checks share one validator with the same
limits and diagnostics. Configuration observations use Host's existing raw-byte
SHA-256 API, preserving returned bytes and independent expected-hash coverage.

All 102 focused selection, Wasm-cache, transaction and configuration cases pass
on actual Rust 1.91, together with 28 example, handoff, real Wasm and process
integration cases. Worker entrypoints remain harness-ignored and run through
their parent cases. Strict all-target Clippy, actual MSRV all-target checking,
Rustdoc and the crate doctest pass. Six real Cargo recipes compare frozen 0.8.0
source with a frozen current candidate: default, dev, test, release, bench and
custom profiles retain exact fingerprints, input/validation digests and Wasm
bytes. Every candidate acquisition reuses its original 0.8.0 cache entry.

The reported release-validation failure exposed a missed batch fixture migration:
removing its invalid output-directory argument made the second specification
valid. The fixture now selects an explicitly empty Cargo profile, retains both
indexed specification failures and their timing/counter assertions, and uses a
temporary workspace with a nonexistent Cargo command. It also verifies no target
directory is created. No production implementation changed. All 12 batch cases
pass on Rust 1.99 and actual Rust 1.91; all four related profile cases pass on
Rust 1.91. Strict all-target Clippy and formatting pass. The failed reproduction,
before-images and focused logs remain under
`/tmp/ic-build-batch-refusal-fix-20261016.OzTeYD/`.

Own manifests/lock, the index and release state remain untouched. No full suite,
native macOS qualification or release ran here. Before-images, the frozen
qualification workspaces and logs remain under
`/tmp/ic-build-profile-cut-20261016.6nTpbD/`.

## Metadata, configuration and release preparation cuts — pending 0.8.0

The incoming manifests/lock select registry Host 0.21.0 while IC Build's own
version remains 0.7.0. Public Host limits and typed failures have different Rust
crate identities from the released Host 0.20 boundary. The complete pending notes
therefore select 0.8.0; consumers crossing that boundary must align direct Host
dependencies and rebuild. This cleanup changes no dependency selection or own
package version.

Shared-target coordination and scheduled-maintenance process tests now create
their own dependency-free Cargo/Wasm workspace under the test fixture root and
pass it to both workers. They no longer depend on the absent Testkit perf package
or return before the process assertions. The real Wasm tests use the same fixture
writer. Host owns marker polling, with explicit ten-second test allowances and
child-aware readiness checks. Unit and integration targets include one shared
fixture-directory wrapper; existing naming and caller-owned retention remain.

On actual Rust 1.91, all 21 selected integration cases pass, including both
restored process parents and all 16 real Wasm cases. Their three harness-ignored
worker entrypoints execute through the parent process cases. Twelve staging and
four configuration cases also pass through the shared unit fixture owner.
Strict all-target Clippy, actual MSRV all-target compilation and formatting pass.
These are focused Linux checks, without full-suite, release or native macOS
qualification. The incoming manifests, lock selection and index remain unchanged.
Before-images and logs remain at `/tmp/ic-build-process-fixtures-20261016.7GEKzb/`.

Configuration observations skip already recorded lookup paths before reading or
expanding their include trees again. Distinct aliases retain their own relative
include directories; active-cycle suppression, first-observation order and fresh
calls' raw-byte checks remain intact. Depfile parsing retains target presence
instead of discarded target strings and inserts dependencies directly into its
ordered set. Package qualification uses Host's existing strict SHA-256 parser
while retaining Build's checksum diagnostic and exact external-selection checks.

On the selected Host 0.21.0 graph, all 30 focused configuration, input/depfile,
package-admission and real Cargo archive cases pass on actual Rust 1.91. A
temporary comparison against the saved depfile parser matches results, error
kinds and diagnostics on 597,871 generated inputs. Strict all-target Clippy,
actual MSRV all-target checking, Rustdoc, formatting and documentation links pass.
No full suite, release or native macOS qualification ran for these cuts. Own
manifests/lock and the real index are byte-identical to their incoming state.
Before-images and logs are retained under
`/tmp/ic-build-config-depfile-cuts-20261016.pTiXky/`.

From released 0.7.0 `f07f4032d1dcac669135a0bdba0857e9b2a2a3c0`, standalone
Cargo observations and archive qualification decode directly into the canonical
schema. Wasm fingerprints retain their raw JSON projection and original
identity/encoding. All five real Cargo observation and typed-refusal cases pass
on actual Rust 1.91. The seven archive-admission cases and real Cargo archive
qualification also pass on that compiler, preserving retained metadata, graph
admission and source inventories.

Input digests delegate lowercase hexadecimal rendering to Host 0.20.1's existing
`Sha256Digest` formatter. All 12 digest cases pass on actual Rust 1.91, including
leading zeroes and exact digest bytes; framing, public types and cache identities
are unchanged. No new dependency selection or upstream publication is required.

Shared Tooling 0.15.2 is adopted from clean committed revision
`f5080b812489654eaa04d368a09f460aea07d4ce`, adding the shared release preflight
entrypoint through canonical distribution. All 101 payloads verify. Build's
adapter delegates locked fetch and ordered tool/pin checks while retaining source
and recovery admission. The unused Cargo editor installer, extra catalog, Make
hooks and editor admission are removed: Shared's workspace-version renderer
already owns version preparation. Existing installed bundles are preserved.
Its actual release fixtures pass on current Bash and genuine Bash 3.2.57,
including common-catalog pin mutation, preparation-authority
consumption, offline/preparation order, original failure statuses and refused
invalid selection. Metadata preparation/recovery/publication substitutes also
pass. Shared owns the helper implementation; no local fork is introduced.

[Host PR #99](https://github.com/dragginzgame/ic-host-tooling/pull/99) is normally
refreshed on released 0.20.1 at `5591efb158db71ab89f7b34d719275e2df9023c1`.
The existing capture implementation passes all 68 selected tool/internal-engine
cases on actual Rust 1.88 and strict affected lint/MSRV/docs/static checks. A
frozen current Build caller deletion passes all 65 Wasm-cache cases, strict
Clippy and actual Rust 1.91 against the coherent temporary Host candidate family;
all other lock selections remain unchanged. The replay patch is retained as
`/tmp/ic-build-cuts071-20261015/build-capture-adoption.patch`. Registry activation
still requires reviewed Host publication under #43; normal Build dependencies
and its local composition remain intact until then. New hosted/native checks
must qualify the refreshed PR head, not its older green receipts.

Root strict Clippy, actual MSRV, Rustdoc, formatting, pins, workspace metadata,
links and snapshot checks pass. Own versions, selected lock, real index and
release state remain unchanged; no release or native qualification ran here.
The separate clean clones exclude sibling dirty source. Before-images, candidate
graph, patches and logs remain under `/tmp/ic-build-cuts071-20261015/`.
The additional archive/digest/editor cuts have before-images and focused logs
under `/tmp/ic-build-three-cuts-20261015/`.

## Host 0.20 admission repair — pending 0.7.0

The maintainer's `8eb59e1d256677b6c1d2fba741258a7ae70a041b` selects registry
Host 0.20 while own package metadata remains 0.6.1. The attempted patch release
stopped at all-target compilation before version preparation: the example and
Host integration tests still used removed `ToolSpec` admission. They now pass
`VersionSpec` and the separate trusted digest to `AdmittedTool::admit`.
The example still consumes caller-supplied authority rather than trusting a hash
of the selected executable. Execution limits, failure evidence and cache
admission remain unchanged.

All-target compilation, strict all-target Clippy and actual Rust 1.91 checking
pass. All three Host integration cases pass on Rust 1.91, retaining changed-tool
invalidation, independent Wasm/post-link caching and failed-producer refusal.
The real example rejects a wrong digest before creating a cache or output, then
builds and reuses the exact same artifact key with matching output bytes under
the supplied correct digest. Rustdoc, formatting and local links pass. These are
focused Linux checks, not a resumed release or native macOS qualification.

Host 0.20 changes the Rust identities exposed through public limits and errors.
The complete pending batch therefore selects minor 0.7.0 instead of patch 0.6.2;
the changelog and current guide requirements agree. Consumers crossing that
boundary must align direct Host dependencies and rebuild. Own versions, lock
selection, the real index and release state remain untouched. Earlier 0.6.2
candidate evidence below retains its original graph and source identity.
Before-images, original failure references and repair logs are retained under
`/tmp/ic-build-host020-fix-20261015/`.

## Canic producer handoff — pending 0.6.2

The inherited-flag implementation from [PR #37](https://github.com/dragginzgame/ic-build/pull/37)
is applied to the normal working tree on released 0.6.1. Its helper and harness
match the reviewed topic source. `cargo::flags::combined_encoded_rustflags`
preserves encoded precedence, explicit emptiness and argument boundaries;
consumers retain remap selection and the command's effective environment.
The guide records adoption, including Canic's three caller-selected remaps.
Compatible notes select 0.6.2; package versions and lock selection stay 0.6.1.

The batch also applies [PR #29](https://github.com/dragginzgame/ic-build/pull/29)'s
strict workspace lockfile projection under [#28](https://github.com/dragginzgame/ic-build/issues/28).
`cargo::lock_projection::verify_workspace_version_projection` projects only
explicit unsourced package versions and compares complete parsed lockfiles.
Dependencies, sources, checksums and unrelated values cannot drift; version
selection, rendering, byte limits, saved inputs and receipts remain caller-owned.
It performs no I/O or release effects. Memory's published adoption remains
[#96](https://github.com/dragginzgame/ic-memory/issues/96); Canic's broader fixture
normalization may rewrite dependency references and is not this strict contract.

Five projection cases pass on actual Rust 1.91, including a self-contained real
Cargo workspace update with two members and a nonmember path dependency. The
fixture confirms permitted version changes and rejects an unselected dependency
change without fetching a new registry graph. Strict all-target Clippy, actual
MSRV all-target checking, Rustdoc, formatting and links pass on the combined
source. The older PR's native results remain bound to its own head; this local
addition has no hosted/native result or publication. Own metadata, selected lock
and the real index are unchanged. Evidence and before-images are retained at
`/tmp/ic-build-projection062-20261015/`.

The same batch applies [PR #41](https://github.com/dragginzgame/ic-build/pull/41)'s
relative input projection under [#40](https://github.com/dragginzgame/ic-build/issues/40).
`InputEvidence::digest_relative_to(root)` hashes captured facts with relative
UTF-8 labels and the shared framed encoder, without rereading the filesystem.
It leaves the absolute digest and cache formats unchanged. Blob retains complete
source selection, rebuild directives and descriptive-provenance adoption; this
API does not attest an executable or admit Canic's release.

All 18 input cases and 12 digest cases pass on actual Rust 1.91. The relative
regressions independently restore each content/name/membership/absence mutation,
cover Unicode and Linux invalid-UTF-8 names, and project after source removal.
A dual-source consumer preserves exact absolute digest bytes against registry
0.6.1 across 30 inventories/identities, including framing boundaries, excluded
trees, missing inputs, renames and native string ordering. Strict all-target
Clippy, actual MSRV, Rustdoc, formatting and links pass on the current graph.
These are focused Linux checks; the local combined batch has no hosted/native
result or publication. Evidence is at `/tmp/ic-build-relative062-20261015/`.

All five flag child scenarios, 16 input/depfile cases and five Cargo observation
cases pass on actual Rust 1.91 on Linux. A separate real-Cargo probe composes
Canic's source/Cargo-home/sysroot remaps in four inherited-environment cases;
all three tokens survive, source paths remap, and caller cfg flags retain their
precedence. Strict all-target Clippy, actual MSRV checking, Rustdoc, formatting
and local links pass. The topic's exact `cff0603406000c4ae020afce34e55d49b6295f14`
also passes its complete [hosted matrix](https://github.com/dragginzgame/ic-build/actions/runs/38756845888),
including native macOS 15 Intel/Apple Silicon. These hosted results qualify that
topic source, not the dirty working tree's documentation changes or a publication.

Producer issues [#12](https://github.com/dragginzgame/ic-build/issues/12) and
[#33](https://github.com/dragginzgame/ic-build/issues/33) close against published
0.6.1 and its complete release matrix; their maintained implementation/tests
remain unchanged. Canic #530/#544/#553 retain consumer release, input/recipe and
application acceptance. Its current handoff names active Auth 0.6 integration
as the release blocker; that work remains with its owner. No Canic checkout,
Root index, version or release ref changed. Flag delivery remains #36; unpublished
Host prerequisites remain #43/#45. Before-images, probes and logs are retained at
`/tmp/ic-build-canic-unblock062-20261015/`.

The additional copy-adapter deletion under #24 still needs Host PR #93: exact
registry Host 0.19.0 has the bounded reader and durable writer, but no
`durable::copy_from`. The current adapter remains unchanged; no prospective
dependency or source override is adopted. Canic's active Auth 0.6.1 transaction
and Root enrollment work remains with its owner.

## Published 0.6.1 acceptance and inherited-flag PR refresh

Published 0.6.1 is `f5ee4bed73d8a93c5bfc5c3d0d31712f597a8bad`, also GitHub main.
The registry index and archive agree on SHA-256
`dc5d2d07e331000117fc7a4d9cff6e825bc4070e7bf0b39d471dac196ca2e50c`;
archive VCS provenance identifies that source and the index records Rust 1.91
without a yank. The complete [release-source CI matrix](https://github.com/dragginzgame/ic-build/actions/runs/38751375358)
passes Linux, macOS 15 Intel/Apple Silicon and lint/security. Issues #35/#46 close
against this delivered evidence. PR #38 is closed as already implemented, without
another merge or branch deletion; its configuration module/tests and Wasm caller
match the release. Memory's adoption remains at
[Memory #110](https://github.com/dragginzgame/ic-memory/issues/110).

[PR #37](https://github.com/dragginzgame/ic-build/pull/37) is refreshed at
`cff0603406000c4ae020afce34e55d49b6295f14` by a normal topic merge of 0.6.1.
Its helper and real-Cargo harness are unchanged. All five child scenarios,
strict all-target Clippy, actual Rust 1.91 checking, strict Rustdoc, formatting
and links pass on Linux. New hosted checks must qualify that exact head.
The PR selects compatible 0.6.2 notes while preserving 0.6.1 package metadata
and the Host 0.19 graph. It is not integrated or published.

Host PRs #135 and #99 pass their complete native/MSRV matrices and are mergeable,
but remain open. Build's prepared deletions still require a reviewed Host
publication under #45/#43. Canic changes are maintainer-owned for this pass;
no sibling checkout is changed. Root's index, version and lock selection remain
untouched. Evidence is retained at `/tmp/ic-build-pr37-refresh-20261015/`.

## Cargo selection, exact-entry preparation and fixtures — pending 0.6.1

Cargo arguments are interpreted once when the specification is constructed or
its argument builder is replaced. Validation, metadata commands and batch grouping
consume those retained facts; batching borrows the metadata argument slice.
Original OS-native arguments still feed command execution and fingerprinting.
The public builder and conservative malformed-argument refusal behavior remain.
Cold Wasm builds and exact-entry reconstruction share retention-safe preparation
and cleanup ownership. Private unit/integration fixture scaffolding has one source
owner; scenario-specific producers and process controls remain with their tests.

All three selection, 65 Wasm-cache, 12 Wasm-batch and four handoff cases pass on
actual Rust 1.91 on Linux; the handoff worker remains intentionally ignored by
the test harness and is selected by its parent process case. Strict all-target
Clippy, actual MSRV checking, Rustdoc, formatting and local links pass.
A temporary comparator using saved original validators/projection agrees on
287934 validation combinations and metadata for every sampled argument sequence,
including native invalid UTF-8. Five real Cargo recipes preserve exact digests
against registry Build 0.6.0. Equivalent release flag spellings still have
distinct identities, as required by the existing raw-argument fingerprint.

[#46](https://github.com/dragginzgame/ic-build/issues/46) records implementation
and qualification. Compatible cleanup extends pending 0.6.1 without changing
package versions, manifests, lock selection or the real index. Incoming source
and the earlier unpublished Host adoptions remain intact. Before-images and
logs are retained at `/tmp/ic-build-internal6-20261015/`; local full-suite,
native macOS and release execution remain separate.

## Streaming capture and shared Host mechanics — pending 0.6.1

Captured outputs now stream through published Host 0.19's bounded copy and
durable writer, removing the complete in-memory buffer. Regular-file/initial
size admission precedes staging; stream limits, original input errors, durable
cleanup evidence and capture lifetime remain. Twelve staging, 30 transaction and
six transaction-batch cases pass on actual Rust 1.91. Strict Clippy, actual MSRV,
Rustdoc, formatting and documentation links pass on Linux. This compatible work
extends 0.6.1 notes without changing the package version or dependency selection.

[Host PR #135](https://github.com/dragginzgame/ic-host-tooling/pull/135) delivers
the bounded visitor and production directory allocator from isolated source
`b1d7c71` on released Host 0.19.0. Host fixture allocation uses that sole engine;
Build cleanup/lifetime, framing and exact-size admission remain here. Host's
actual Rust 1.88 selected tests, strict affected lint/docs, MSRV, formatting,
links and snapshot checks pass. Normal sibling checkouts remain untouched.

The two Build caller deletions are implemented in an isolated consumer. All 125
selected digest/staging/transaction/batch/Wasm-cache cases, strict Clippy and
actual Rust 1.91 checking pass against the coherent Host candidate family. Four
dual-source Cargo recipes preserve exact fingerprints against registry Build
0.6.0. The normal checkout retains published dependencies and its existing loops
until a reviewed Host release supplies the new APIs. The checked replay patch,
before-images, exact graph and logs are retained under
`/tmp/ic-build-cuts6-20261015/`; [#45](https://github.com/dragginzgame/ic-build/issues/45)
tracks publication and activation. Incoming configuration work and the real
index/manifests/lock are preserved. Native macOS and release execution remain
separate.

## Published 0.6.0, Canic acceptance and shared configuration — pending 0.6.1

Published 0.6.0 is `d0eaf2d95fc4191eab1b59a80d19c43816580f08`. The cached
registry archive and sparse-index checksum agree on SHA-256
`40afef44b47e796ce55c412e20cf96a27ab2f7d04b6bef6c9eb1c66e42528fde`;
its provenance identifies that commit and its normalized manifest selects Host
0.19 and Rust 1.91. The redundant Host-alignment PR #44 is closed as implemented
by this release, without another merge or branch deletion.

Canic's owner has adopted registry Build 0.6 and consolidated workspace discovery
under explicit fixture command/limits policy. Its default/governed compilation,
strict fixture lint and selected installed checkpoint case pass on Build 0.6,
Host 0.19 and Testkit 0.45. Recorded root/private graph digests still match the
normal checkout at inspection. These are the owner's retained receipts under
`/tmp/canic-build06-adoption/`, not a fresh full Canic validation or live Backup
acceptance claim. Its active source and staging are untouched.

The exact implementation from PR #38 `aaff82542b899b4781d4cd22e863101107a707e9`
is applied locally on released 0.6. Cargo configuration discovery/include traversal
now has one owner at `cargo::environment`; the Wasm engine consumes its facts.
The three public cases and all 65 Wasm-cache cases pass on actual Rust 1.91.
A dual-source consumer comparing registry 0.6 with this source preserves exact
fingerprints for direct, string/array include and precedence recipes. Cargo 1.91
itself rejects optional-table include syntax before either resolver; the public
observation case separately covers optional missing includes. Original failed
probe evidence remains retained rather than being relabelled as acceptance.

Strict all-target Clippy, actual MSRV, Rustdoc, formatting, links, workspace
metadata, pins and snapshot verification pass on Linux. README/guide requirements
now describe published Build 0.6 with Host 0.19. Notes select compatible 0.6.1;
own package version, lock selection and staging remain unchanged. Canic's
configuration caller deletion still needs producer publication and consumer
qualification under #35; the guide records its absence/input-authority boundary.
Host PR #99 remains open and unpublished, so #43's controlled-capture deletion
is still gated. Native qualification and release execution remain separate.
Before-images, source patch and logs are at `/tmp/ic-build-061-issues-20261015/`.

## Directory cleanup, workspace observation and controlled capture — pending 0.6.0

Released source is 0.5.0 `4feef850010ed725edb9b9e9adeaed140baf5106`.
Owned-directory fields now perform Drop cleanup directly; transaction staging
still drops before both locks, while explicit close/abort recovery is preserved.
Workspace discovery moves to `cargo::observation::workspace_root_for` with the
caller's Command and OutputLimits. The old artifact export is gone. Offline
membership, auto-install refusal and complete-output admission remain; failed
commands retain typed Host status, diagnostics and cleanup evidence. In-repository
callers and the guide use the new contract. This selects 0.6.0 notes without
changing package metadata.

All 47 affected observation/staging/transaction units, real Cargo membership
cases and the cross-process artifact-lock parent case pass on actual Rust 1.91.
Strict all-target Clippy, MSRV compilation, Rustdoc, formatting, links, workspace
metadata, pins and snapshot checks pass on Linux.

Host candidate `a9fe39d6eed16b302e2f929b331c101da818c69a` extends existing
PR #99 through normal merges of released 0.18.6 and then 0.19.0 main.
The unpublished cancellation API accepts explicit cleanup and a shared absolute
deadline through the existing capture engine. All 68 selected tool/resolution/
internal process cases, strict affected lint/docs and actual Rust 1.88 checks pass.
The temporary Build caller deletion passes all 65 Wasm-cache cases, strict Clippy
and actual Rust 1.91 all-target checking against that coherent Host family, with
external selections unchanged. Normal Build retains its published implementation
until a reviewed Host release permits a real minimum and the prepared caller
deletion.

[#43](https://github.com/dragginzgame/ic-build/issues/43) tracks delivery and
consumer adoption; native macOS qualification remains separate. Normal sibling
checkouts, Root staging, package version and lock selection are preserved.
After an initial approval-review rejection, the maintainer explicitly delegated
the upstream delivery. Candidate `a9fe39d6` is pushed to existing Host PR #99;
its description records the updated API and fresh validation. No merge or
release ran.
Before-images, candidate source, the adoption patch and raw logs are retained
under `/tmp/ic-build-cleanup5-20261015/`.

## Shared Tooling 0.15.0 and registry Host 0.18.6 — 2026-10-15

The canonical refresh selects Shared Tooling's verified current main revision
`3bef3f2daec7f64f875397ed4d53d5b9e2e68a08` (0.15.0). All 96 payloads verify,
including the newly required exact version/Gregorian-date admission helper for
the shared release runner. Local agent and README baseline references agree.
Host's latest published tag is 0.18.6 at
`7f291760ac1184e5235d08f75e3c1aee0fa60cc3`. All three registry Host packages
were already selected at 0.18.6 in the incoming lock; a scoped online update
confirms that selection without changing other packages or API minimums.
The unpublished filesystem extraction remains separate.

All 84 affected filesystem, batching, host-composition and Wasm-cache cases pass
on actual Rust 1.91. Strict all-target Clippy, MSRV compilation, Rustdoc,
formatting, links, workspace metadata, pins, tools and snapshot checks pass on
Linux. Exact release-input fixtures, shared runner command-stub cases and isolated
local release adapters pass.
Host/IC/release executable pins, incoming source, lock selection, package version
and staging are preserved. Native qualification and release delivery remain
separate; no commit, release or publication ran. Before-images and validation
logs are retained in `/tmp/ic-build-latest-20261015/`.

## Environment composition and compiler identity — pending 0.5.0

`WasmBuildSpec::with_extra_env` now merges overrides across calls: later values
win for duplicate names, and an empty call preserves prior configuration.
The compiler probe setter is now `with_rustc_identity_program`, with no old-name
alias. It remains identity-only; explicit `RUSTC` still selects Cargo's compiler
and takes precedence for its identity probe. Callers and guidance use the new
name. Historical replacement-map receipts below describe the earlier contract.

All 65 Wasm-cache cases and three host-composition cases pass on actual Rust
1.91, including child delivery, chained/combined cache parity, changed-input
identity and explicit compiler precedence after another environment call.
Strict all-target Clippy, MSRV compilation, Rustdoc, formatting, links, workspace
metadata, pins and snapshot checks pass on Linux. Native qualification remains
separate. Own metadata, incoming lock selection and staging are unchanged;
no commit or release ran. Before-images are retained in
`/tmp/ic-build-env-hardcut-20261015/`.

## Batch phases, filesystem accounting and fixture paths — pending 0.5.0

[#42](https://github.com/dragginzgame/ic-build/issues/42) owns the three approved
cleanups. A private per-entry batch phase function replaces nested result/timing
construction. All six cases pass, including simultaneous callback/abort failure
with original causes, exact failure/report timing and later-entry completion.
The public test-directory helper and its mirrored assertion are removed; guides
select explicit paths. This removal selects 0.5.0 notes without changing 0.4.0
package metadata. Strict all-target Clippy, actual Rust 1.91, Rustdoc, formatting,
workspace metadata, pins, snapshot and documentation links pass on Linux.

Host's isolated `feat/logical-tree-size` candidate `5f2fbd7` moves the metadata
accounting primitive and two existing filesystem cases into `ic-host-fs::read`.
Rust 1.88 cases/all-target checks, strict Clippy and Rustdoc pass. Build's prepared
caller deletion passes Rust 1.91, strict lint and cache/pruning/maintenance cases
against that coherent candidate family. Normal Build keeps the existing owner
until reviewed Host publication permits a real dependency minimum; the prepared
patch introduces no production override or prospective dependency.

Canic's isolated `test/own-artifact-target-paths` topic switches all upstream
path-helper imports to unpublished fixture support. Default/governed caller
compilation, strict all-target lint, actual Rust 1.91 and formatting/links pass
on unchanged pushed-main dependencies. A separate patch applies cleanly to its
active unpublished source without importing that dirty work into the topic.

Automatic approval review rejected topic-branch publication and PR creation for
lack of explicit push/PR delegation. Both contributions and PR descriptions are
ready locally; no branch push, merge or release ran. Native and live PocketIC
acceptance remain separate. Normal sibling checkouts, Root staging and incoming
lock selections are preserved. Before-images, patches and logs are retained in
`/tmp/ic-build-cleanup4-20261015/`.

## Published 0.4.0 and delivered consumer acceptance — 2026-10-15

Published 0.4.0 identifies `d223747a003c35954d02e43ed110dd23707affa1`;
the registry archive matches sparse-index SHA-256
`17bc5e5c137a0055ce069d2d244017aafcddcfe47fd92bf2141bf7e7154c1010`.
Its normalized manifest and index declare Rust 1.91.0. A fresh registry consumer
selecting exact 0.4.0, cargo-platform 0.3.3 and Host 0.18.5 passes a locked/offline
check on actual Rust/Cargo 1.91; no source override or producer lock is used.
This closes [#34](https://github.com/dragginzgame/ic-build/issues/34).

Delivered Jobs 0.11.0 `7e6c643c50dfe3f67252e51537f44391d92db2c7` delegates
capture/finalization to registry Build 0.3.3. Its five capture-tool cases and
complete Linux/Intel/Apple Silicon [CI](https://github.com/dragginzgame/ic-jobs/actions/runs/38702118931)
pass, closing Build [#14](https://github.com/dragginzgame/ic-build/issues/14) and
[#26](https://github.com/dragginzgame/ic-build/issues/26). No live recovery claim
is made. Jobs' incoming Build 0.4 selection still needs host-harness Rust 1.91
admission; the delivered receipt does not qualify that graph.

The README now selects registry 0.4 and the actual Shared Tooling 0.14 snapshot.
Compatible notes select 0.4.1; own metadata remains 0.4.0. The incoming Host 0.18.5
lock update is preserved. Open producer implementations remain in their PRs;
the configuration PR's Intel failure is a pre-compilation Rust-download DNS
failure, while its reconciliation with released main is separately required.
Release 0.4.0 [CI](https://github.com/dragginzgame/ic-build/actions/runs/38704810434)
passes Linux, Apple Silicon and lint/security; Intel is still running at this
inspection. Evidence and original source snapshots are retained under
`/tmp/ic-build-issues040-20261015/`. No staging, branch update or release ran.
The pending sections below retain their original source-bound historical receipts.

## Rust 1.91 MSRV — pending 0.4.0

The maintainer selected Rust 1.91.0 to admit `cargo-platform` 0.3.3. The workspace
and inherited package floor, local agent guidance and README now agree; pending
0.4.0 notes call out the consumer compiler requirement. The development compiler
remains independently pinned to 1.99.0. CI and `make msrv` already read the floor
from the root manifest.

Actual Rust/Cargo 1.91 passes `make msrv` (all workspace targets/features), all
18 Cargo tests and the existing v1 fingerprint regression on Linux. A fresh
cache-backed consumer resolves `cargo-platform` 0.3.3 and Host 0.18.4 independently
of the producer lock and passes both initial and locked checks. Only the selected
platform package/checksum changed in the producer lock, now SHA-256
`9a77c5dc69bd356fe3c5cf71d86a5d0cf0e7a8877d1ca99f9c4d727de4a5f431`.
Formatting, snapshot, pins, workspace metadata and documentation links pass.

The release gate exposed eight MSRV-dependent duration-unit lint failures. Tests
and the cache example now use equivalent hour/minute constructors; strict
all-target/all-feature Clippy and actual Rust 1.91 MSRV checking pass. Formatting
and documentation links pass; logs are in `/tmp/ic-build-duration-clippy-20261015/`.

Published 0.3.3 still advertises Rust 1.88; this change does not rewrite that
release. [#34 — Dependency graph MSRV](https://github.com/dragginzgame/ic-build/issues/34)
retains release delivery. Native macOS qualification remains a hosted gate.
Own package version and incoming staging are preserved; no release ran. Evidence
is retained under `/tmp/ic-build-msrv191-20261015/`.

## Canonical decoding, source traversal and helper retirement — pending 0.4.0

All metadata consumers now decode through the canonical Cargo schema at
`cargo::observation`. Wasm input discovery retains only typed indexes and the
original JSON needed for its existing v1 fingerprint projection. Independently
captured pre-change digest fixtures match, including missing/null package fields
and unknown dependency-kind fields. Controlled probes and archive command/log
retention remain at their existing policy owners.

Package source inventory delegates Unix descriptor traversal to published Host
0.18.3, hashes the opened file and preserves byte/mode evidence and aggregate
entry/depth budgets. The Unix visitor owns qualification policy only; the existing
non-Unix fallback remains because Host's descriptor walker is Unix-only.
This batch adds the package's direct Serde/Rustix dependency edges and declares
the filesystem minimum as 0.18.3. The Host family advanced to 0.18.4 during
closeout; the final graph is qualified below and its lock remains unchanged at
SHA-256 `401c3fd39fca6c1a7f310468e66aed53a8fa8b14ed7bb4157bf3c8a056ac1180`.
Earlier 0.18.3 receipts and the original incoming selection remain retained.

The public Wasm path/readiness helpers are removed. Fixture path construction is
in test support, and the guide uses retained build records. This public removal
selects pending 0.4.0 notes, retaining the earlier Shared Tooling note. Own package
metadata stays 0.3.3; retained artifacts are not migrated or cleaned.

All 119 affected Linux cases pass, including the real Cargo package qualifier,
actual Wasm/examples, source-budget regressions and fingerprint compatibility.
Strict all-target/all-feature Clippy, warnings-denied Rustdoc and actual Rust 1.88
library compilation pass on that Host 0.18.4 graph. Native macOS and release gates
remain separate; [#34 — Fresh-resolution MSRV compatibility](https://github.com/dragginzgame/ic-build/issues/34)
is still open. Source,
notes and earlier dirty work remain uncommitted; no release or publication ran.
Before-images, original failures and logs are retained under
`/tmp/ic-build-cleanup3-20261015/`.

## Published Cargo observations adopted in Canic — 2026-10-15

Canic's authorized working-tree integration now uses published Build 0.3.3 for
metadata and tree observations. Its private metadata schema and unbounded capture
are removed; discovery, role graphs and selected export inventories use canonical
Cargo types and IDs. The existing #553 producer/recipe authority is preserved.
Canic owns the 16 MiB stdout/1 MiB stderr budgets, commands and sanitized findings;
no elapsed deadline, compatibility schema or source override is introduced.

The 41 role, 44 reuse/export, seven Fleet, 18 bootstrap, ten artifact and four
canonical-selection checks pass; the reuse lane leaves its opt-in matrix ignored.
Strict Host library/test Clippy passes. Actual Rust 1.91 library checking passes
in a frozen source copy on the unchanged incoming registry graph, lock SHA-256
`1fb060c8029f246f1841f67d2d6c61c8e8bda51cc3c07ef03fa10d35db64372f`
(Build 0.3.3, Host 0.18.3, Testkit 0.43, Memory 0.38, Metrics 0.10, Timers 0.22.6).
The opt-in real-Cargo cold/warm/relocation/source-change matrix also passes in
that copy; its synthetic release sealing is not installed Fleet acceptance.

Canic independently raised its Memory requirement to 0.39 during validation;
the resulting live locked-check refusal is retained separately, without altering
that owner's work or claiming the old graph qualifies it. Build #33/Canic #530
retain delivery, native and current-graph/application acceptance. Changes and
pending 0.112 notes remain uncommitted; no own version/release or live effect ran.
Task before-images, failed attempts and frozen-source evidence are retained under
`/tmp/ic-build-canic-adopt033-20261015/`.

PR #29 `d26a76a` and #31 `bd38b51` now pass their complete hosted Linux,
lint/security and native macOS Intel/Apple Silicon matrices. Review, integration
and their respective published consumer adoption remain open.

## Shared Tooling 0.14.0 and delivered cleanup — 2026-10-15

The canonical refresh adopts clean Shared Tooling 0.14.0 at
`5f70392ccc15085cdba9e6c7c596fc29b0dce341`. All 95 payloads verify and a repeat
preview reports no changes. Tool pins and executable helpers are unchanged;
pending upstream helper proposals are not claimed as delivered. Pending notes
select 0.3.4; own package metadata remains 0.3.3. Documentation links pass.

Published 0.3.3 names `f6f22590287d36277b68df9692282d69edb25176`; its registry
archive matches sparse-index checksum
`e27a0e5fc995b05319faafa062bd2a6adc12edaadf5e8fb5bc76c3e482686394`.
[Release CI 38633020109](https://github.com/dragginzgame/ic-build/actions/runs/38633020109)
passes Linux, lint/security and native macOS 15 Intel/Apple Silicon. This clears
the remaining native gates for delivered cleanup issues #18–#21, #27 and #32,
which are closed with evidence. #26 and #33 retain Jobs/Canic adoption acceptance.
Canic's owner has local #553 export-scoping work; no duplicate repair is added.
Host's published 0.18.3 still lacks the stream-copy API required by #24.

PR #29 `d26a76a` and #31 `bd38b51` are normally reconciled with released 0.3.3;
their original projection/native implementations are unchanged. Focused units,
the real native integration, strict Clippy/docs, actual Rust 1.88, formatting,
snapshot and link checks pass on the normal Host 0.18.2 registry graph. Pending
notes select 0.3.4; new-head hosted/native and published consumer gates remain.

Root edits remain uncommitted and the incoming Host 0.18.3 lock selection is
preserved byte-for-byte, outside the released graph's qualification. No Root
staging, version bump, release or publication ran. Evidence is retained under
`/tmp/ic-build-shared014-20261015/`; live sibling source is untouched.

## Bounded Cargo observation and Canic removal rehearsal — pending 0.3.3

Build #33 now has catalog/resolved-graph and selected-tree observations with
caller-selected commands, environments, feature/target/lock/offline choices and
complete-output limits. Metadata uses the upstream `cargo_metadata` 0.23.1 schema;
feature-safe grouping shares the tree observer. Typed Host failure/status and
truncation evidence remain intact; no default elapsed deadline or cache is added.
All 18 affected Cargo cases, strict all-target/all-feature Clippy, warnings-denied
docs and actual Rust 1.88 observation tests/library checks pass. The incoming
Host 0.18.2 lock selection is preserved; own metadata remains 0.3.2.

The isolated Canic rehearsal at committed `644d4b13012b395d6211a6907e2e6aafdf391db9`
removes its local metadata schema/feature builder and unbounded command capture.
Callers use canonical package IDs, target/dependency kinds and paths. Canic retains
observation budgets, discovery/role recipes and bounded sanitized diagnostics.
All 41 role-package cases, seven generated-package cases, 19 reuse cases (one
optional measurement ignored), strict Host library/test Clippy and actual Rust
1.91 library compilation pass. The
producer path override and Host graph alignment exist only
in that disposable rehearsal. Normal registry adoption requires producer
publication and reconciliation with Canic's current input-selection/changelog
work; its live source, index, graph and refs are untouched.

A real shared-target Cargo probe confirms distinct same-name workspace package
IDs can share one build-script output directory. Cached structured messages remain
available with an absent uplifted depfile and ignore unrelated malformed records.
Canic #553 must bind retained exports to exact package/recipe and record bytes;
directory/prefix filtering cannot establish that authority. This probe and the
new schema boundary do not implement that consumer receipt/admission repair.

Canic #519's job `115969175140` now passes embedded-peer preflight and reaches
PocketIC suites before cancellation. Full PocketIC/native/review acceptance stays
open. Before-images, failed attempts, source patch and scoped evidence are under
`/tmp/ic-build-cargo-observation/`. No broad gate, version bump, commit, release or
publication ran for this task.

## Capture finalization and peer source paths — pending 0.3.3

Build #26's additive `CapturedArtifact::retain_to` and `CaptureRetentionError`
are applied to the working tree, using the delivered directory-custody guard.
Custody transfers before rename; failure reports the actual retained path and
staging child, and cleanup removes only an empty child without hidden retry.
All 11 staging cases, four handoff cases (one controlled worker ignored), strict
Clippy/docs and actual Rust 1.88 pass on the normal registry graph.

The proposed Jobs caller removes its manual rename/cleanup sequence and retains
native error kind plus structured custody. Its two real capture tests and strict
lint pass on source `37937543a58cd1fd54eea8dbd9bb0382f658a194`, with an isolated
owner override only for rehearsal. The patch applies to the active caller; its
live source and incoming application changes are untouched. Published owner
adoption, native acceptance and Jobs review remain open at Build #26/Jobs #42.

Canic CI run `38590334477` retained peer artifact `11909600044`. Its compiler and
lock match local selection; the bytes reveal installed-rust-src-dependent
standard-library locations. The selected compiler's virtual source root now
normalizes those locations. The locally refreshed peer matches retained CI Wasm
byte-for-byte at `ce25919429b494a761db6f8838e349a47df2dd809a4d6a7f83de4225af775ab2`.
Canic #519 `ad69834` / #546 `fce62e2` carry the fix; #551 retains CI/native
acceptance. Root source, index and own versions/refs remain uncommitted.
Before-images, actual CI bytes, comparisons and caller proofs are retained under
`/tmp/ic-build-retain033/`; earlier handoff/changelog edits are preserved.

## Released 0.3.2 and PR reconciliation — 2026-10-14

The maintainer published 0.3.2 from `da344beb8a951d020249a83746456399b5c5b6ef`.
The downloaded archive matches its sparse-index checksum
`9376d6ab7241be995d669f1b60674e435c669f41478a24ab29313970e2b9ad4b`.
Release CI `38586895793` passes Linux/lint; native Intel/Apple Silicon are queued.
This delivers #27/#32; their explicit native acceptance remains open.

PR #29 `e2e7230` and #31 `f791954` are normally reconciled with that release.
Only changelogs conflicted; pending notes select 0.3.3 and producer Rust is
unchanged. Projection/native protocol units, the real native integration, strict
Clippy/docs, actual Rust 1.88, formatting and links pass on Host 0.18.1.
Both PRs are mergeable; new required checks and registry consumer adoption remain.
Host stream-copy PR #93 remains unpublished; Build #24's reviewed rehearsal is
retained and the live graph continues to use the published API.

Canic's security, ordinary, MSRV and check jobs pass. PocketIC runs
`38578657429`/`38578893452` still refuse peer preflight before server startup;
both build `ce25919429b494a761db6f8838e349a47df2dd809a4d6a7f83de4225af775ab2`
against expected `969b5a08c22a5fe1ec7749475e7e3b7993c1f9c8230781639eba346c451caa67`.
The local real-registry-root Cargo-home reproduction preserves expected bytes.
Canic #519 `51783a0` and normal topic merge #546 `8c6b81a` now retain actual
mismatch Wasm/provenance in CI without relaxing refusal. Six focused cases on
each topic, strict fixture-owner Clippy, actual Rust 1.91 libraries and workflow
lint pass; normal peer verification passes locally. Current CI acceptance is pending.
Evidence is under `/tmp/ic-build-after032/`; source assets stay unchanged.

## Fixture adoption and directory custody — pending 0.3.2

Both Build test allocators use published Host 0.18.1's opt-in fixture API;
local collision removal and counters are gone. The normal registry graph selects
the coherent 0.18.1 family. A private owned-directory guard now serves Wasm,
artifact transactions and captures, preserving public Drop contracts, protected
lock lifetime, failed-build retry and explicit-close recovery. 159 artifact units,
the added paired-error case, affected process/helper/handoff integrations, strict
Clippy/docs and actual Rust 1.88 pass. Focused final guard regressions also pass.

Host copy PR #93 is normally reconciled with published 0.18.1 at `60ceef4`, with
pending 0.18.2 notes; its unchanged copy API passes 68 durable cases, strict lint,
docs and Rust 1.88. It remains unpublished. Build's current-source stream-copy
adoption patch is rehearsed under `/tmp/ic-build-cleanup-20261014/`; the live
consumer retains its published implementation until that API is released.
Native macOS and reviewed delivery remain open at Build #24/#27/#32. The earlier
documentation edits are preserved; source/index/own versions/refs stay uncommitted.

## Published 0.3.1 and remaining PR repairs — 2026-10-14

The maintainer pushed/published 0.3.1 from `259e8c8c1bd8576bbfb3e306dba3fb2151b7fa90`.
Its sparse-index/archive checksum is
`671361f0f18d3d592e9ae652dc29325a986d2a0472c88c4627706132a997f986`;
the VCS receipt names that source. The selected release CI `38567150403` passes
Linux/lint; both native jobs are queued. Cancelled 0.3.0/Jobs 0.10.0 runs are
historical; Jobs' active 0.10.2 run passes Linux with native acceptance queued.
Publication clears #22/#25 delivery, not their native/consumer gates. Closed
policy PR #23's obsolete queued workflow was cancelled without touching release CI.

PR #29 is reconciled with 0.3.1 at `5271356`; only release notes conflicted.
Its unchanged projection API passes four focused cases, strict Clippy/docs,
actual Rust 1.88, formatting/snapshot/links on the current graph. The prior head's
complete native matrix retains its identity; current Linux/lint pass and native
checks are queued. Compatible pending notes select 0.3.2. Local guidance also
corrects 0.3.1's exposed Host boundary to 0.18; those docs remain uncommitted.

Canic's completed Apple run exposed Binaryen temporary-path expectations; all
seven cases now pass through a real parent alias. Fresh Linux CI still differed
because remapped source strings retained Cargo's cache-directory hash. The
recipe normalizes registry source roots; a compiler-backed location regression,
five peer cases and explicit real-Wasm path/version reproduction pass, with
strict Host/fixture-owner Clippy and Rust 1.91 libraries. Byte comparison remains
strict and mismatch errors retain both digests. The path-repair heads
#519 `88009a0` and #546 `f77819a` are retained as separate evidence;
combined verification uses the rebuilt owner API.
Current hosted/native/application acceptance remains open. Original failures,
qualification logs and before-images remain under `/tmp/ic-build-pr-next-20261014/`.

The restarted Canic security jobs found four new 2026-10-14 RustSec advisories
against rustls 0.23.45. [Canic #560](https://github.com/dragginzgame/canic/issues/560)
tracks the isolated PR graph repair to 0.23.46; the live Canic checkout already
selects that patch. The real risk gate passes against database revision
`caee11f41fab07d676c0a4655a2a5dd1f94da97c`, preserving advisory refusal and the
existing informational warnings. This is separate from native/application proof.
The repair is pushed at #519 `b60c3c6` and normal topic merge #546 `bcc3cbc`.
Peer bytes remain identical and graph provenance is refreshed. Both real risk
gates and current-source peer qualification pass; current hosted acceptance is pending.

## PR and issue reconciliation — 2026-10-14

PR #23 is closed as superseded by the pending canonical Shared Tooling 0.12.7
adoption, without merging or deleting its branch. Exact released 0.3.0 CI
`38546827916` passes Linux, lint/security and native Intel; Apple Silicon remains
queued. Explicit native acceptance at #18–#21 is still outstanding. The old Host
type/permission producer blockers are delivered; #9/#12 retain their separate
consumer/application/native acceptance. Jobs' retained-artifact adoption is
delivered in 0.10.0, with Linux CI passing and both native jobs queued at #14.

Canic PRs #519/#546 fail embedded-peer preflight before PocketIC startup. The
earlier compiler repair called `with_extra_env` twice; the second call replaced
the first map and dropped path remaps and fixture settings. The isolated repair
selects all settings together and extends real-Wasm reproduction to reject
embedded Cargo/sysroot paths. Logs, before-fix failures and source/graph evidence
are retained under `/tmp/ic-build-pr-reconcile-20261014/`.

Both topic updates are pushed: #519 `836e6daf616787b52de91f25b0e508b429d45b63`
and #546 `060b6a06e5a7f04231f14c2adfe78cfce554eab2`, through normal topic merges.
Four peer cases, real path/version reproduction and wrapper/unwrapped admission
pass; the combined selector graph also passes forced fresh-build qualification.
New CI exposed yanked num-modular 0.6.6, tracked at
[Canic #556 — Repair yanked dependency selection](https://github.com/dragginzgame/canic/issues/556).
Separate commits select nonyanked 0.6.5, with byte-identical published Rust
sources and peer Wasm. Both real risk gates, 51 Candid cases (one ignored), strict
Host/fixture-owner Clippy and Rust 1.91 libraries pass on that graph. Graph
provenance is refreshed; advisory refusal and all other selections are preserved.
Required CI, native/app acceptance and owner integration remain open. No Root
staging, own version/ref changes or release ran; live sibling work is untouched.
An incoming Root catalog update to Host 0.18 appeared during closeout and is
preserved. This task's Canic topic-graph evidence and documentation checks do
not qualify that separate dependency-adoption graph.

Host PR #103's documentation conflicts with released 0.18.0 are resolved through
normal topic merge `9d2f0fe`; unpublished fixture notes now target 0.18.1.
Its allocator/caller Rust is unchanged. Current-graph 94 filesystem and 94 process
units, collision, strict lint/docs, default-feature and Rust 1.88 checks pass.
The PR is mergeable; required native CI and publication remain open at Host #98
and Build #27. Producer proof does not qualify the incoming Root Host 0.18 graph.

## Shared release metadata, streamed hashes and fixture extraction — pending 0.3.1

Shared Tooling 0.12.7 is exported from immutable source
`9364ffe3f770a6d5d94809f0bf82c33202054bf8`, with the canonical Cargo release
metadata entrypoint explicitly selected. All 95 payloads verify; repeat preview
reports only KEEP and tool pins are unchanged. The local release adapter delegates
metadata transactions while retaining source admission, selected Cargo editor,
idempotent recovery and local commit checks. Existing isolated adapter fixtures
pass rollback, lost-response recovery, frozen external selections, exact-commit
validation, release routing and publication refusal/success substitutes.

The pending archive qualifier now uses Host's published bounded no-follow
streaming hash, preserving raw SHA-256, limits and caller source custody.
All seven focused package units pass on the normal registry graph, as do strict
all-target/all-feature Clippy and actual Rust 1.88 checking. Other owners' pending
archive/resolver work and the incoming Host lock selection are preserved.

[Host PR #103 — Fixture directory allocation](https://github.com/dragginzgame/ic-host-tooling/pull/103)
adds the opt-in filesystem allocator and removes both upstream fixture allocation
loops in an isolated clone based on released 0.17.13. Host's existing cleanup and
process serialization remain local. Linux affected filesystem/process units,
collision/alias/permission cases, strict lint/docs and Rust 1.88 pass.
[Build #27 — Adopt published fixture allocation](https://github.com/dragginzgame/ic-build/issues/27)
owns the remaining consumer switch. Its isolated rehearsal passes 157 artifact
units and 15 integration cases (one environment-dependent case ignored), strict
Clippy and Rust 1.88. The patch is retained under `/tmp/ic-build-cleanup-followup/`;
the live Build helpers remain on published APIs until producer publication.

Native macOS evidence, producer review/integration/publication and registry
consumer qualification remain separate. Build source, index, versions and refs
remain uncommitted; only the authorized Host topic branch was committed/pushed.
The adopted canonical focused-check policy supersedes PR #23's separate 0.12.5
reference once these local changes are integrated; that PR is not merged.

## Coordinated archive qualification — pending 0.3.1

[Build #25](https://github.com/dragginzgame/ic-build/issues/25) owns the
maintainer-authorized generic Cargo archive engine. Consumers select packages,
versions, tools, temporary path patches, source custody and budgets. Qualification
retains an exact extracted graph and frozen external selections, uses a fresh
compilation target, runs all-target/doc tests and checks original inputs,
archives, source bytes/modes and admitted verification inputs afterward.
Traversal is bounded by entries/depth and file bytes. Command failures retain
original Host execution evidence separately from failed diagnostic writes.
No retry, cleanup, installer, publication or release admission is supplied.

Full Linux `make ci lint check-secrets` passes on the actual registry graph,
including 170 units, real Cargo/Wasm/process suites and Rust 1.88. A subsequent
malformed-lock correction passes all 170 units, strict lint/docs and Rust 1.88;
unaffected full-gate evidence is retained. Blob's real same-version/refusal
regression passes against the prospective producer in isolation. Source
before-images, failed attempts, exact graphs and raw logs remain under
`/tmp/blob-extract3.lPJtvW/`. Native macOS and registry consumer activation
remain outstanding under Blob #69 / archive qualification #56.
Real Build metadata remains 0.3.0; prospective 0.3.1 is confined to the isolated
graph. No release, publication, staging or Git effects ran.


## Shared-target export qualification — 2026-10-14

Canic's current exact-depfile caller selects its Cargo graph, while its retained
build-script export scanner still opens every shared-target `build/*/output`.
The scanner/path helpers match committed Canic `644d4b13012b395d6211a6907e2e6aafdf391db9`.
Two disposable probes compile those unchanged functions with the existing Host
reader and IC Build evidence guard: a foreign export adds an unobserved input,
and foreign invalid UTF-8 fails before selected-package admission. This is
function-level evidence, with a qualification-only error projection; it is not
a complete Canic build, PocketIC or native proof. Source comparisons and logs
are retained under `/tmp/ic-build-next-issues/`.
[Canic #553 — Scope retained build-script exports](https://github.com/dragginzgame/canic/issues/553)
owns the remaining consumer recipe boundary. The live checkout is unchanged.

Host PR #93 remains open, and the authoritative registry index ends at 0.17.13;
the durable-copy API is not yet adoptable under Build #24. Released Build 0.3.0
now passes native Intel CI; Apple Silicon remains queued. These inspection
results do not qualify the pending bounded-resolver source or close acceptance.

## Caller-bounded Cargo resolution and Canic CI repair — 2026-10-14

The starting checkout is clean published 0.3.0 source
`84cc436ddacf3e9ef21b62167bb9a50a52f871ff`. Its registry archive names that
source and checksum `415c50c37611014ab87016292bffb7ca366498c52062d226a0c9e886346683f8`.
Its Linux CI passes; both exact native macOS jobs remain queued at inspection.
Earlier cleanup issues remain open through that required native qualification.

[#22 — Caller-bounded Cargo input resolution](https://github.com/dragginzgame/ic-build/issues/22)
now has an additive working-tree implementation and compatible 0.3.1 notes.
The same resolver accepts an optional caller absolute deadline, cancellation and
Host process-group cleanup for its three probes and repeated revalidation.
Default probe execution, output limits, discovery, admission and fingerprinting
are preserved. Filesystem discovery/hashing are outside the probe deadline.
Query retains build execution, receipt framing, target custody and lifecycle;
this producer change does not establish published adoption or native delivery.

Full Linux `make ci lint check-secrets` passes: 163 unit tests, actual Cargo/Wasm
examples and existing integration/retention checks, strict Clippy/docs, actual
Rust 1.88 and isolated release adapters. Initial Clippy and fixture failures and
the final gate are retained under `/tmp/ic-build-bounded-probes/`. Source, manifest
versions and the index remain uncommitted; no root release or push ran.
An incoming lock-only update to Host 0.17.13 arrived after the first passing gate.
It is preserved separately from this task; the full gate passes on that graph too.

Authorized Canic CI repairs use isolated PR checkouts. Worker custody now uses
physical temporary paths, tool-resolution fixtures compare the physical
namespace, and the normal allocation-peer owner refreshes stale bytes and
provenance ([Canic #551 — Refresh embedded allocation peer](https://github.com/dragginzgame/canic/issues/551)).
The live Canic checkout and its release graph remain untouched. Focused Linux
worker, resolution, peer verification/freshness and Host lint checks pass;
required native CI and review remain delivery boundaries. PR #23's revised
immutable Shared Tooling 0.12.5 policy reference is consistent; its native checks
are queued, and it is not merged or applied to this checkout.

Canic [PR #519](https://github.com/dragginzgame/canic/pull/519) is updated at
`a25f4fd22eab23e1a2a5fa9947a9bbfa064e25a0`; [PR #546](https://github.com/dragginzgame/canic/pull/546)
includes that prerequisite through normal merge `f76ea5bf3835462920f8f4ff383cb6b979aecd95`.
The peer recipe binds Cargo to the compiler whose sysroot it remaps. Implicit
and explicit compiler entry points, actual path/version reproduction, fixture
lint and Rust 1.91 pass. Updated required CI is in progress; neither PR is merged.

## Shared 0.12.4 and published Host 0.17.11 adoption — 2026-10-14

The maintainer authorized both adoptions and dependent repairs. Shared Tooling
0.12.4 is exported from clean committed source
`8b2dbd70a24330fdc6c102a01772f4d3e5b628d7`, matching remote main at inspection.
Uncommitted sibling work is excluded. Canonical preview/refresh
retains existing roots and adds the publication guide explicitly for the missing
companion at [Shared #176 — Export the publication guide](https://github.com/dragginzgame/shared-tooling/issues/176).
All 89 payloads verify and repeat preview reports only KEEP. Host/IC/Cargo and CI
tool pins are byte-identical to the starting snapshot; explicit common setup and
offline checks pass. The actual collector's local archive, compaction, corruption
and original-failure retention fixtures pass using isolated synthetic tool stores.

The artifact/fs/process catalog now requires published registry Host 0.17.11,
retaining disabled defaults and explicit features. All three registry VCS receipts
name `afb05ee53c5ade59dad25fc4a9264a5200963e9c`. The incoming dirty lock already
selected those packages and is preserved except for removing fs2, its three
winapi-only dependencies and Build's direct edge. Remaining versions/checksums
are unchanged. [#21 — Adopt Host descriptor locks](https://github.com/dragginzgame/ic-build/issues/21) now uses
the published shared/exclusive descriptor locks and final-owner unlock in the
actual checkout, with no path override. Namespace policy, cloned-record retention,
duplicate-descriptor release, typed errors and lock timing remain local.

Full Linux `make ci lint check-secrets` passes with 161 units, actual Cargo/Wasm,
process/retention/repair/handoff and example cases, strict Clippy/docs, actual
Rust 1.88 and isolated release adapters. The first missing-guide failure and the
successful final gate are retained under `/tmp/ic-build-adopt-0123/`, alongside
before-images, exporter previews and structured lock comparison. Own metadata
remains 0.2.7; the accepted public removal keeps the combined draft at 0.3.0.
The index and incoming cleanup source are preserved. No sibling mutation, staging,
commit, push, release, publication or CI dispatch ran. This consumer source has
no exact native CI result yet; Host's release CI passes Linux/MSRV and Apple
Silicon, with Intel still running at inspection.

## Accepted cleanup and breaking 0.3.0 draft — 2026-10-14

The maintainer approved all three reviewed reductions over source
`5f1b00e1ba6b60a3d78e8f43c16d24df78cd9de1`. The unused watched-input stamp API
and its separate protocol/tests retire at
[#19](https://github.com/dragginzgame/ic-build/issues/19). Local Rust searches and
organization code search find no production callers outside Build; search results
include historical sibling documentation, not proof about external consumers.
Use `InputEvidence` for input guards and `ArtifactCacheSpec` for verified retained
outputs. Existing sidecars stay untouched; no compatibility reexports or unused
Canic replacement are introduced. Public removal makes the complete pending
batch breaking, so its notes move from 0.2.8 to 0.3.0 and retain the fixture adoption
notes. Own metadata remains 0.2.7; no release/version operation ran.

[#20](https://github.com/dragginzgame/ic-build/issues/20) gives the verified
destination-copy decision one private owner. Cold publications still copy;
verified warm outputs preserve matching independent destinations under the same
ownership/link/permission checks. Digest domains, typed caller error context and
the all-copies-before-Wasm-stamps barrier remain unchanged. Full Linux
`make ci lint check-secrets` passes with 161 units, existing real Cargo/Wasm,
process/retention/repair/handoff cases, strict Clippy/docs and actual Rust 1.88.
The initial missing-import failure and corrected complete gate are both retained.

The authorized [Host #89](https://github.com/dragginzgame/ic-host-tooling/issues/89)
adds regular-descriptor shared/exclusive acquisition and explicit unlock to the
existing owner. Its focused tests and full Linux/MSRV gate pass, preserving
incoming private-parent and concurrent tooling work. The fs2-free Build candidate
passes the complete package checks with the same 161 units and integration cases.
Producer overrides and prospective Host 0.17.11 metadata exist only in that
disposable copy. [#21](https://github.com/dragginzgame/ic-build/issues/21) owns
normal registry adoption after publication; the actual checkout retains published
dependencies and fs2 until then. Reviewable patch and all evidence are under
`/tmp/ic-build-cleanup3/`. Native macOS and producer publication remain distinct
from Linux candidate proof. No staging, commit, push or publication ran.

## Published Host fixture adoption and isolated Canic PR — 2026-10-14

IC Build HEAD is published 0.2.7 source
`68d21a913f41927be952acd68ec52bc941a82900`. Its
[exact CI](https://github.com/dragginzgame/ic-build/actions/runs/38535022329)
passes Linux, native macOS 15 Intel/Apple Silicon and lint/security. Those
released results do not qualify the new Host fixture graph described below.

Host 0.17.9 is published from `6fd078d9b914fff530b1efe521dc08ce18b97cc0`.
For [#18](https://github.com/dragginzgame/ic-build/issues/18), IC Build now imports
its canonical executable-script writer directly in unit and integration fixtures
and deletes both local implementations and their module declarations. Fixture
contents and assertions stay local; fallible writes retain explicit test-call
failure. Only the Unix dev dependency enables `test-support`; production feature
inspection shows the process dependency with no enabled features. The process
catalog minimum and selected lock advance to 0.17.9; other selections stay intact.

Compatible pending notes select 0.2.8; own metadata remains 0.2.7. Full Linux
`make ci lint check-secrets` passes with 164 units, actual Cargo/Wasm, process,
handoff and example fixtures, strict Clippy/docs and actual Rust 1.88. Evidence is
under `/tmp/ic-build-018/`. The first manifest-ordering failure and the successful
final gate are both retained. Delivery and exact native qualification of this
new fixture graph remain outstanding; no producer Git or release operation ran.

The maintainer requests a PR for
[Canic #542](https://github.com/dragginzgame/canic/issues/542), superseding the
earlier queue decision for that isolated delivery. Remote Canic main remains
`25b73cd403b20b4e148c6e340f8c760a26d97f6e` (0.111); its local 0.112 source and
large concurrent batch are preserved. The isolated contribution includes the
library-only command prerequisite and binds that same selector into complete-build
identity. It preserves record encoding/namespace and retained artifacts, without
a second reader or cleanup. [Canic PR #546](https://github.com/dragginzgame/canic/pull/546)
delivers commit `d616547a1643edac1b2823b581a3bd19ddce6e06` for review against
main. Its 95 build/reuse cases (three governed cases ignored), 18 bootstrap
cases, strict Host library/test Clippy and actual Rust 1.91 checking pass.
PR evidence is under `/tmp/ic-build-canic-542-pr/` and `/tmp/canic-542-*.log`;
native and complete application acceptance remain distinct from these checks.

## Published 0.2.6 and registry Canic qualification — 2026-10-14

Registry 0.2.6 names `8afa2d66c4d49a84154f1e1c1904dc033622b585`, checksum
`226c5e59d1226d272d5940155f8ed58abf4630fbd4cbd6bed83ca5143608e5a9`.
Its [exact CI](https://github.com/dragginzgame/ic-build/actions/runs/38530184826)
passes Linux x86-64, native macOS 15 Intel/Apple Silicon and lint/security.
[#15](https://github.com/dragginzgame/ic-build/issues/15) and
[#16](https://github.com/dragginzgame/ic-build/issues/16) close on those native
results. The selected-record API at #17 is published and natively qualified.

The local workspace now requires compatible Host 0.17.8 for artifacts/fs/process,
preserving features and the already-selected lock byte-for-byte. This prevents
older retained-staging source from satisfying the advertised minimum. Compatible
pending notes select 0.2.7; own metadata remains 0.2.6. Full Linux
`make ci lint check-secrets` passes with 164 units and actual Rust 1.88.

The concurrent Canic owner has applied the registry 0.2.6 requirement and exact
record adapter to its existing dirty 0.112 batch. This task preserves those
incoming files and qualifies a frozen current source with actual registry Build
0.2.6/Host 0.17.8, with no producer override or lock change. All 91 build-family
cases pass (three governed cases ignored), plus strict Host library/test Clippy
and actual Rust 1.91 all-feature library compilation.

Both dedicated and embedded managed Blob build-only workers complete four-artifact
builds, then verify exact release-ID reuse under matched launcher environments.
A new malformed foreign depfile was added to the shared output target during
the dedicated cold build and retained through both compositions. It does not
contaminate their selected input graphs. The first embedded repeat is retained
in the evidence: the harness changed launcher environment, and the cache
correctly rebuilt. Matched repetition then reused all four verified artifacts.
No PocketIC server or deployment ran. Evidence is under `/tmp/ic-build-canic-026/`.

The bounded producer #17 closes; Canic #544 retains current-source delivery and
complete application/native acceptance. Larger #1/#9/#12 and Canic #528/#529/#530
remain open. The explicit #542 release hold remains; Canic 0.112 is not delivered.
Host 0.17.8's unrelated Darwin FIFO-test compilation defect is reported on
[Host #85](https://github.com/dragginzgame/ic-host-tooling/issues/85); its full
fixture acceptance is distinct from the successful IC Build native gates.

## Selected-record input boundary and Canic handoff — 2026-10-14

Pending compatible 0.2.6 adds `read_cargo_depfile_inputs` at the existing input
owner ([#17](https://github.com/dragginzgame/ic-build/issues/17)). Exact-record
and directory discovery share bounded no-follow reads, strict parsing, alias
resolution and generated-root exclusion. Consumers choose target records and
cold-inventory absence; malformed selected records and late inputs still refuse.
Directory discovery's original contracts and retained evidence remain unchanged.
Own metadata remains 0.2.5; the incoming Host 0.17.7 lock is preserved byte-for-byte.
Full Linux `make ci lint check-secrets` passes with 164 units, Cargo/Wasm/process
fixtures, strict Clippy/docs and actual Rust 1.88.

Canic's stopped 0.112 release attempt rejected an unobserved foreign App manifest
while Blob's worker shared the suite target. Retained diagnostics prove the
before/after mismatch. A concurrent owner now scopes records to the selected
Cargo graph; [Canic #544](https://github.com/dragginzgame/canic/issues/544) tracks
that repair and shared-owner adoption. This task changes no live Canic source.
The isolated adapter replaces repeated record interpretation with the new API;
Canic retains target selection, generated-export recipes and release admission.
Its 91 build-family checks pass, with three governed cases ignored. Strict Host
library/test Clippy and actual Rust 1.91 all-feature library compilation pass.
The temporary producer override is confined to
the qualification copy. Registry package versions/checksums stay unchanged;
Cargo also normalizes some existing Windows dependency edges there, retained in
`adapter-lock-delta.json`. The real consumer graph is untouched.

Evidence and adapter patch are under `/tmp/ic-build-canic-025-integration/`.
Host 0.17.8 remains unpublished; its native repair must be delivered and adopted
before exact macOS qualification. Publish the producer boundary before Canic's
registry switch. The explicit #542 hold remains until release completion; the
failed attempt and new concurrent source edits do not constitute delivery.

[#7](https://github.com/dragginzgame/ic-build/issues/7) closes on published
IC Build 0.2.5 and Testkit 0.38.0 helper consolidation. Testkit's exact source
`35938ee0c8fc2c3e5efa2cf198e39d6e8c486b2e` passes all four Linux CI lanes;
the separate Host/native and broader extraction acceptance issues remain open.

## Authorized Host repair prepared — 2026-10-13

The maintainer authorized [Host #84](https://github.com/dragginzgame/ic-host-tooling/issues/84).
The repair is applied to Host's released 0.17.7 source
`2bdc5f1cae2c12040bf08d03b5c68502291599cd`, with compatible pending 0.17.8 notes.
Retained staging reuses its existing checked native-permissions helper; invalid
high-bit permissions refuse before creation. Full Linux `make ci msrv-workspace`
passes, including 82 filesystem tests, strict Clippy/docs, tooling/release fixtures
and actual Rust 1.88 workspace compilation. Host's lockfile is unchanged.
Evidence is under `/tmp/ic-build-host84/`. Native qualification and publication
remain outstanding; IC Build's minimum requirement/locked graph adoption must
follow the published repair. Own IC Build versions and dependency inputs remain unchanged.

The completed 0.2.5 CI now fails on both macOS architectures at the same Host
0.17.5 call, with Linux and lint/security passing. Apple Silicon's job log is
`/tmp/ic-build-host84/ic-build-025-apple.log`. Canic's independent 0.112.0 release
is still running governed tests on admitted `644d4b13012b395d6211a6907e2e6aafdf391db9`;
its explicitly queued #542 patch remains unapplied. This follow-up performs no
staging, commit, version bump, release, publication or CI dispatch.

## Pushed 0.2.5, native dependency blocker and Canic hold — 2026-10-13

The maintainer pushed finalized 0.2.5 source
`1ae67242b73e002414105c8de55b05f9364c1e0a`. Its
[exact CI](https://github.com/dragginzgame/ic-build/actions/runs/38441416124)
passes Linux and lint/security; Intel fails during package compilation against
locked Host 0.17.5, and Apple Silicon is queued at inspection. The previous
0.2.4 run now has both native jobs failed against Host 0.17.4. These results
supersede the running/queued observations in the historical sections below.

Both completed native 0.2.4 jobs and the 0.2.5 Intel job report E0308 in Host's
retained staging: its public `u32` permissions reach a Darwin `u16` mode
constructor. The existing checked native-permissions helper already handles
Apple narrowing in the same filesystem owner. The repair and native acceptance
are tracked at [Host #84](https://github.com/dragginzgame/ic-host-tooling/issues/84).
Registry 0.17.6/0.17.7 and current Host source retain the offending call; source
inspection does not qualify those releases. A published Host repair and its
minimum requirement/locked graph adoption must precede new exact native
producer qualification. No downstream workaround or Host source edit was made.
Local logs are `/tmp/ic-build-024-intel.log`, `/tmp/ic-build-024-arm.log` and
`/tmp/ic-build-025-intel.log`; the hosted jobs retain portable evidence.

Canic's independent release remains active on admitted source
`644d4b13012b395d6211a6907e2e6aafdf391db9`, preparing 0.112.0 from 0.111.0.
Its release validation is running governed PocketIC tests and infrastructure
Cargo builds at inspection. The maintainer's explicit hold still applies to
[Canic #542](https://github.com/dragginzgame/canic/issues/542). The retained
selector/identity patch remains unapplied until that release finishes; IC Build's
push does not end Canic's source admission. No sibling source, Git refs, package
versions, release, publication or workflow dispatch were changed in this follow-up.

## Published 0.2.4, helper cleanup and Canic integration — 2026-10-13

Registry IC Build 0.2.4 names source
`b3eba0f4b45fbca8a2a8cba6afcc3cfd88be5348`; checksum is
`68c9379fb1224f483d1896cb91448d0bf17f46a2c98277c1402d71867adb0a23`.
Its [exact CI](https://github.com/dragginzgame/ic-build/actions/runs/38424234824)
passes Linux/lint, with Intel running and Apple Silicon queued at inspection.
The cancelled 0.2.3 run does not establish its unfinished Intel result.

The maintainer authorized Canic integration and continued issue work. For
[#7](https://github.com/dragginzgame/ic-build/issues/7), IC Build now uses standard
optional-duration reduction in both timing aggregates, preserving absent and
measured-zero values plus saturation. The duplicated timing module retires and
its regression follows the Wasm aggregate; artifact aggregation has its own
boundary coverage. Testkit's separate owner has already removed its generic
label/timing modules in the current dirty draft, with contract-specific label
admission and phase accumulation retained there. That source inspection is not
Testkit delivery or a new consumer qualification. No Testkit source was edited.

Compatible pending notes select 0.2.5; own metadata stays 0.2.4. Full Linux
`make ci lint check-secrets` passes with 163 units, actual Cargo/Wasm/process and
retention cases, strict Clippy/docs, actual Rust 1.88 and isolated release adapters.
The incoming Host 0.17.5 lock selection remains byte-for-byte unchanged.

Canic's catalog and lock advance only registry IC Build 0.2.3 to 0.2.4; other
selected packages/checksums are unchanged. Configured and infrastructure Cargo
commands now select libraries, preserving named-library JSON selection, profiles,
features and qualification/publication policy. A new actual Cargo collision case
fails on the original command and passes after the repair. The redundant generic
subcommand helper retires into the existing build command owner. The maintainer
committed these applied files while restarting the Canic release; this agent
performed no staging, commit, release or publication action.

The further shared selector/complete-build identity patch is qualified in the
isolated consumer and retained at `/tmp/ic-build-canic-024/canic-selector-followup.patch`.
The maintainer explicitly requests queueing it until the restarted release
finishes. Its application must preserve that admitted source. Record
format and namespace remain unchanged; the selected library recipe contributes
to identity without reading a parallel cache format or deleting retained records.
Evidence and before-images are under `/tmp/ic-build-canic-024/`; producer validation
is `producer-full-validation.log`. Final consumer checks and application status
are recorded on Canic #528/#529/#530 and IC Build #9/#12. The queued follow-up
is [Canic #542](https://github.com/dragginzgame/canic/issues/542), by explicit
maintainer instruction; it is not applied to the active release source. Native and complete
application acceptance remain open. Historical sections keep their source identities.

## Shared Tooling 0.11.2 and Host 0.17.4 adoption — 2026-10-13

The maintainer authorized both upgrades. Shared Tooling 0.11.2 is exported from
clean committed source `81bdf144482b21f146acf542f8013eefe1dfeba8`, matching remote
main at review, through its canonical preview/refresh workflow. The snapshot
verifies all 86 payloads. Local CI setup uses `install-ci-tool.sh TOOL`; obsolete
snapshot-owned forwarding commands retire through the exporter. Existing common
host/IC/Rust and CI versions/digests are unchanged. Shared release recovery uses
the annotated-tag checker; selected evidence and setup fixes arrive with the
same reviewed snapshot. No live release or hosted evidence round trip ran.

The workspace catalog now requires compatible artifact/fs/process Host 0.17.4
releases, preserving explicit features and disabled defaults. Cargo registry
search confirms 0.17.4 is the newest stable release for all three; their registry
VCS receipts agree on `28f3a6d5771ecb941b86cd3c6ea2507de82b101e`.
The incoming dirty lockfile already selected these versions and is preserved
byte-for-byte. No unrelated dependency upgrade or sibling edit ran.

Explicit locked fetch, `make install-tools tools-check` and actual unified CI
installer preparation pass. Full Linux `make ci lint check-secrets` passes with
162 unit cases, real Cargo/Wasm and retained/process checks, strict Clippy/docs,
actual Rust 1.88 and isolated release-adapter fixtures. Final local links and
snapshot integrity also pass. Evidence and before-images are retained under
`/tmp/ic-build-tooling-upgrade/`; `full-validation.log` is the complete gate receipt.

This compatible consumer adoption joins the existing pending 0.2.4 notes; Shared
Tooling's CLI retirement does not break IC Build's published library boundary.
Own version stays 0.2.3. No staging, commit, push, package-version change, release,
publication or CI dispatch ran. Native qualification of this source/graph remains
pending at #1/#9. Prior 0.2.3 CI passes Linux, Apple Silicon and lint/security;
Intel is running at inspection. Those results do not qualify this new snapshot
or Host selection. Earlier sections preserve their original evidence.

## Published 0.2.3 and pending 0.2.4 consumer boundaries — 2026-10-13

Registry 0.2.3 resolves to source
`23e047b7672aae0d5ba442c4b384c4b782835c87`; its archive SHA-256 is
`9f5baf0449153f28de324d37dc9787298d77788940e0bce88f3d0c252b018ca5`.
Its [exact CI](https://github.com/dragginzgame/ic-build/actions/runs/38416043907)
passes Linux and lint/security; both native macOS 15 jobs are running at inspection.
[#15](https://github.com/dragginzgame/ic-build/issues/15) remains open for that
source's native qualification. The older 0.2.2 run was cancelled.

The maintainer accepted the #16/#14/#13 producer batch. Pending notes select
compatible 0.2.4: additive APIs and corrected default Cargo filenames, with
existing default-library identity, v1 formats, namespaces and execution policy
preserved. Own manifests and lock remain 0.2.3.

- [#16](https://github.com/dragginzgame/ic-build/issues/16): package hyphens become
  underscores in default cdylib output paths across cache modes. Renamed/non-cdylib
  libraries and colliding normalized filenames remain refused before acquisition.
- [#14](https://github.com/dragginzgame/ic-build/issues/14): adopt the prepared
  `CapturedArtifact::retain` transfer. Normal close/Drop still clean only their
  owned child. The Jobs adapter preserves original/O3/Os/Oz bytes, including
  retained capture after a failed rename and process exit. Published producer
  dependency and caller admission still precede Jobs adoption.
- [#13](https://github.com/dragginzgame/ic-build/issues/13): the existing resolver
  structurally selects one named cdylib example, fingerprints that selection and
  admits its output. The existing Cargo command factory is public for uncached,
  caller-executed builds. Query keeps deadlines, cancellation/cleanup, receipts,
  DID/metadata publication and smoke/deployment lifecycle. Shared input inventory
  projects Query's unchanged relative-path/raw-byte framing; it does not replace
  that receipt with `InputEvidence::digest()`.

Complete Linux `make ci lint check-secrets` passes: 162 units, six named-example
integration cases, 16 actual Cargo/Wasm cases, retained artifacts/process/input
checks, strict Clippy/docs, actual Rust 1.88 and isolated release adapters. The
new cases cover isolated/shared reuse, normalized example/library filenames,
changed/missing target admission, configured target-directory override, prepared
snapshot selection, source changes, late depfiles, capture lifetime and caller
failure/deadline/cancellation evidence. A frozen 796-file Query inventory gives
identical old/shared-projection source hashes and rejects mutation. Current Query
locked/offline metadata admits its named probe and selected output; a repeated
resolution confirms identity. These are
producer and adapter checks, not Query smoke or new native qualification.

Evidence is under `/tmp/ic-build-024/`: `full-validation.log`,
`jobs-adapter-proof.log` and `query-receipt-final.log`. Sibling repositories stay
read-only. No staging, commit, version change, push, release, publication or CI
trigger ran. Issues stay open for delivery/native acceptance and actual adoption;
historical sections below preserve their original identities.

## Published 0.2.2 and Canic input-engine integration — 2026-10-13

Version 0.2.2 is published from
`e9be97fa356c14cb5431b879e304a0f8cc5077c3`; archive SHA-256 is
`d408281094f1ae6928f02102131a904b8edcf5693b6c8f01b105e6488d0c4d82`, and
its registry VCS receipt names that release. The newer
[0.2.2 CI](https://github.com/dragginzgame/ic-build/actions/runs/38407403272)
passes Linux and lint/security; Intel fails the new native-path fixture before
capture because its `0xff` filename is inadmissible (`Illegal byte sequence`).
Apple Silicon is queued. Exact
[0.2.1 CI](https://github.com/dragginzgame/ic-build/actions/runs/38394499212)
passes Linux x86-64, native macOS 15 Intel/Apple Silicon and lint/security.
This closes the delivered setup/compiler/Clippy/catalog/editor repairs at
#5, #6, #8, #10 and #11; it does not qualify the new 0.2.2 evidence APIs.
[#15](https://github.com/dragginzgame/ic-build/issues/15) records the bounded
fixture repair: valid Unicode native files on every Unix host, with invalid-byte
files only on Linux, following the existing digest fixture. Actual capture,
digest and descendant symlink/path-error checks remain active on macOS. Engine
semantics are unchanged; exact delivered native qualification is still needed.

The maintainer authorized Canic integration after publication. Canic's catalog
requires registry 0.2.2; its incoming lock already selected that release. Its
complete-build production path now consumes `InputEvidence` directly for capture,
digest and before/after validation, and the strict shared depfile discovery API
for explicitly selected records. Its old recursive scanner, snapshot engine and
parser are removed. Canic retains role/family/generated-export/fixture recipes,
source custody, tool/environment identity, complete-release record format and
namespace, output projection, release/role admission, locks and diagnostics.

Two remaining generic regression declarations move from Canic to IC Build:
source edits/renames/exclusions and generated-root alias/external-input discovery.
They use the existing shared APIs. Complete Linux `make ci lint check-secrets`
passes with 161 units, integration/real Cargo/Wasm/process cases, strict Clippy,
docs and actual Rust 1.88. Own metadata stays 0.2.2; compatible pending 0.2.3
notes cover this producer fixture/test ownership cleanup. Canic's focused reuse suite
passes 40 cases, with its governed opt-in synthetic-sealing case ignored. These
include real Cargo generation, source/tool/environment drift, disappearing
inputs, cold/repeat release reuse, tampered outputs, diagnostics, cancellation and
required-tool absence retaining the original filesystem failure.

Receipts and task-owned before-images are under `/tmp/ic-build-canic-022/`:
`producer-final-validation.log`, `canic-final-clippy.log` and
`canic-final-reuse-tests.log`. Initial receipts remain alongside them. Concurrent
Canic Backup/query/dependency work is preserved. Explicit locked preparation
admits its incoming graph; final strict Host library/test Clippy and reuse checks
pass with Host 0.17.3/Query 0.55.4. Lock SHA-256 is
`0b30060e6d39ff08ac2c54b2d07de74b21f10863e19426ebba89a5630d78bc10`, unchanged
through those checks. The query owner resolves its duration lint. New-source
native and complete consumer acceptance remain at #12/#530/#1/#15.
The wider Canic 0.112 Auth/Backup/Blob batch remains open. No staging, commit,
package-version change, push, release, publication, CI dispatch or broad Canic
gate ran. Historical sections below retain their original source identities.

## Published 0.2.1 and pending 0.2.2 input evidence — 2026-10-13

Version 0.2.1 is published from
`b797ed7cdfc3303987d8c02af5aad8dadd22eddd`; archive SHA-256 is
`a52c3b1a7c80eb24899cd38bbf6767e792d1236cf6f9522f30dd075ab48fae79`.
The archive's VCS receipt names that release. Shared Tooling 0.10 is delivered
in this source. Its [CI](https://github.com/dragginzgame/ic-build/actions/runs/38394499212)
passes Linux, Apple Silicon and lint/security; Intel is running
at inspection. The prior 0.2.0 run was cancelled, so its unfinished hosts do not
establish native acceptance.

The maintainer reports Canic #530 is in progress with its own owner and asks for
continued IC Build issue work. This batch addresses the remaining current-guide
corrections in [#2](https://github.com/dragginzgame/ic-build/issues/2): generic
resolution, source leases and transactions now name IC Build; the public type
boundary selects Host 0.17. This guide and the implemented public Rustdoc own
the supported contracts. Original Testkit designs/migrations remain historical
references, pinned to extraction source
`b906b1a44f8f5f0f2f23d2413a76103d99e4fbb3` with their paths/anchors verified.
Retained `.ic-testkit` namespaces and stamp/cache bytes are untouched.

The ownership design separates implemented consumer routes from the original
audit/proposals. README and agent instructions refer to the canonical version
and handoff instead of carrying a stale workspace-version assertion. Pending
notes are compatible 0.2.2; own metadata remains 0.2.1. The initial documentation
checkpoint passed link/diff checks with executable inputs unchanged.

The active Canic #530 owner then filed
[#12](https://github.com/dragginzgame/ic-build/issues/12) for standalone input
authority. `InputEvidenceSpec`, `InputEvidence`, `InputEvidenceValue` and
`InputEvidenceError` now capture and validate per-path bytes, directory membership
and explicit absence independently of an artifact-cache transaction. Strict
`parse_cargo_depfile` / `discover_cargo_depfile_inputs` use caller-selected
directories, exclusions and record bounds. Original inputs are rechecked even
when depfiles disappear or replace their inventories; late bytes cannot acquire
pre-build authority. The existing traversal is shared, preserving old cache
fingerprints and the consumer's string-sorted, big-endian length-framed encoding.
No second production scanner, recipe, release admission or cache namespace is
introduced. Source/parent custody remains the caller's responsibility.

Thirteen focused cases and a real Cargo external-include depfile fixture pass.
Complete Linux `make ci lint check-secrets` passes, including 159 units, retained
artifact/process/real Wasm integration checks, strict Clippy, docs, actual Rust
1.88 and isolated release-adapter fixtures. Receipts and before-images are under
`/tmp/ic-build-022-docs/`; the final full receipt is `full-validation.log`.
Canic's owner retains its implementation/migration and consumer acceptance; this
batch changes only IC Build. New source is locally verified, unpublished and has
no native macOS qualification yet. Released-source native rows and consumer
delivery acceptance remain at their existing GitHub issues. Manifests, lockfile,
tool selection and retained cache formats are unchanged. No staging, commit,
push, version/release/publication or CI action ran.

## Published 0.2.0 adoption and pending 0.2.1 tooling — 2026-10-13

Version 0.2.0 is published from
`9ddcd4ee0b2ac85f0608b8aaf49493cf015f295a`; archive SHA-256 is
`fb055d1b5649636c09a52f23fdc091417c732cbb87a2125d5a24f93c043c8df8`.
Its [release CI](https://github.com/dragginzgame/ic-build/actions/runs/38389784699)
passes Linux and lint/security, including fresh release-editor setup. Both macOS
jobs remain queued at inspection; earlier cancelled runs do not qualify them.

The authorized continuation adopts the published registry line in IcyDB and
Testkit. Their unused Host 0.16 artifact/fs/process packages leave the lockfiles;
all other selected registry versions/checksums remain unchanged. Canic's owner
had already selected 0.2.0. No real consumer carries a producer path override.
Focused registry qualification passes strict Canic Host all-target/all-feature
Clippy and 45 artifact/output/binding/declaration cases, including the extractor;
one governed PocketIC binding case stays ignored. IcyDB passes selected Clippy,
nine artifact/Candid and five cache cases, including real Wasm flag identity;
the ordinary CLI remains parser-free. Testkit passes library/driver Clippy,
17 driver, six diagnostics and six pure pool tests, plus actual Rust 1.88.
Receipts and consumer before-images are under
`/tmp/ic-build-020-registry-adoption/`. Incoming consumer work is preserved.

Pending compatible 0.2.1 adopts [Shared 0.10](https://github.com/dragginzgame/ic-build/issues/10)
at `ed9c26f015691859f67b50b512a94709dcdc0c6c`: 70 roots, 88 canonical files.
Twenty-seven payloads update, and the newly referenced reporting rule is added
as a canonical entrypoint. Initial link validation caught that omitted rule;
the retained failed log precedes the complete, verified closure. Common tool
versions/digests match; no exception or alternate installation owner is added.
Release-editor selection remains consumer-owned. Explicit setup/offline admission
and full Linux `make ci lint check-secrets` pass with the incoming Host 0.17.1
lock graph, including actual Rust 1.88. Final receipt:
`/tmp/ic-build-020-registry-adoption/shared010-final-validation.log`.
README registry guidance and the local baseline identify the current owners.
KEEP-only repeat refresh and integrity pass. Canonical Make, formatting,
freshness, host/IC store and actual evidence-collector fixtures pass on Bash 5
and genuine Bash 3.2. Corruption cases use only synthetic owned installations;
their receipts are the `shared010-final-owner-*` and `shared010-host-ic-*` logs.

Native/application acceptance remains open on the existing issues; the larger
[Canic #530](https://github.com/dragginzgame/canic/issues/530) engine removal
still requires those prerequisites. Toko's migration remains on
[#1796](https://github.com/dragginzgame/toko/issues/1796). Own metadata stays
0.2.0, IcyDB's compatible draft stays 0.273.1, and Testkit's stays 0.37.1.
No staging, commit, push, version change, release, publication or CI action ran
in this continuation. Historical sections retain their original graph scope.

## 0.2.0 release setup and consumer qualification — 2026-10-13

The authorized continuation fixes [#11](https://github.com/dragginzgame/ic-build/issues/11):
`make install-tools` explicitly prepares consumer-pinned cargo-edit 0.13.13
through the canonical shared selected-tool installer. Offline checks and release
metadata use its admitted absolute executable. Preflight checks both pin files
for input changes; failed adapter fixtures print bounded inner diagnostics and
retain complete logs. The existing Host 0.17 and LTO work remains intact.

Linux `make ci lint check-secrets` passes, including 146 units, real Cargo/Wasm
and process contracts, strict Clippy and actual Rust 1.88. Receipts:
`/tmp/ic-build-020-11-validation.log` and the final shell/documentation/adapter
rerun `/tmp/ic-build-020-11-final-focused.log`. Private Cargo-home/no-global-plugin
adapters pass under `/tmp/ic-build-11-no-global.Um1McV/`. Missing editor selection
refuses offline; explicit setup and adapters pass under
`/tmp/ic-build-11-fresh-selection.COoG9r/`. An injected admission fault prints
the retained inner log. The first fresh harness omitted common-tool link targets;
that harness failure is retained separately from the corrected proof.

Frozen candidate qualification lives under `/tmp/ic-build-020-consumers.GxZq30/`.
Only those disposable producers/manifests select 0.2.0 and local overrides.
Current Canic `0d6779da5d9f1f7e92ef532ef32663f28fff4677` passes strict Host
all-target/all-feature Clippy and 45 artifact/output/binding/declaration checks,
including the real extractor; one governed PocketIC binding case stays ignored.
Its original Blob 0.23.1 selection is preserved after explicit exact cache setup.
Testkit's existing dirty retention work passes driver Clippy and all 17 tests;
initial temporary proofs lacked nested Cargo override/Git provenance and are
retained as fixture failures. IcyDB's corrected `candid-service` integration
profile passes strict selected Clippy and nine artifact/Candid tests on both its
registry graph and the candidate; its ordinary CLI remains parser-free.

Real consumer catalogs remain registry-based. Publish 0.2.0 before adopting its
requirement in Canic, IcyDB and Testkit; Canic's normal Host boundary still needs
that release. Toko's existing [#1796](https://github.com/dragginzgame/toko/issues/1796)
now names direct IC Build/current Testkit ownership. The larger Canic #530
removal remains behind native/application acceptance. Native macOS is unqualified;
0.1.4 CI still has the recorded Linux failure and queued macOS jobs. Package
metadata remains 0.1.4. No live Git, version, release, publication or CI effects.

## Host 0.17 public-boundary alignment — 2026-10-13

The maintainer authorized this sibling change to unblock Canic. IC Build now
selects Host artifacts/fs/process 0.17 together. No Rust API implementation or
cache-format change was needed. Its public grouping `OutputLimits` and typed
Host failures now belong to 0.17, so the pending release is correctly classified
as breaking 0.2.0. Package metadata remains 0.1.4; publication is maintainer-owned.

Linux delivery evidence passes across `/tmp/ic-build-host017-validation.log`
and `/tmp/ic-build-host017-followup.log`: compilation, strict Clippy, rustdoc,
146 unit tests, real Cargo/Wasm and process cases, actual Rust 1.88, release
commands/adapters, shell/workflow lint and secret scanning. The first adapter
fixture failed because it reused the live minor draft for a patch transaction;
it now authors its own disposable patch notes, and its focused rerun passes.
No live Git release effects ran. Native macOS remains unqualified for this graph.

Canic Host compiles against the frozen producer in
`/tmp/canic-host017-proof.oEVphD`, using an override only in that temporary copy.
Its strict all-feature library/test Clippy check also passes
(`/tmp/canic-host017-consumer-clippy.log`). The real Canic catalog/lock remain registry-based; publication and adoption are
still needed to clear its normal build. Concurrent LTO documentation was
preserved. No commit, package-version edit, push or publication was performed.

## Prepared deployable-Wasm guidance — 2026-10-13

Released HEAD is 0.1.4 at `848ae20548527f5595ea68e7edf9cfec4f3188ce`.
The undated 0.2.0 draft adds [LTO guidance](../artifacts.md#deployable-wasm-lto):
IC Timers' linked probes save 18.67–18.85% raw Wasm with full LTO. This motivates
complete-application qualification, including final transforms and instruction
budgets, before a common deployable-Wasm adapter adopts the selection. Existing
`with_extra_env` already makes an explicit override part of build identity.
Engine defaults, public API and Cargo inputs are unchanged. Documentation checks
are scoped separately from builds and product acceptance. Earlier sections retain
their original source and qualification identities.

## Pending 0.1.4 qualification batch — 2026-10-13

The maintainer requested more work for 0.1.4. The existing fresh-runner Wasm
setup correction is extended with [#8](https://github.com/dragginzgame/ic-build/issues/8):
warning-denied Rust Clippy now passes for the library, unit/integration tests and
examples with all features. `make clippy` is a primitive, offline/locked package
check; `make check` and `make ci` run it before rustdoc and tests. The development
toolchain declares both Clippy and the Wasm target for explicit setup.

The 43 initial library/test diagnostics are corrected, together with one source
guard diagnostic exposed when integration targets became reachable. Public
error/panic contracts are documented, private cache-miss inputs are borrowed,
and small test/style corrections preserve behavior. Source guards remain held
through the final session/snapshot operation; borrowed owners explicitly drop
before the guards. Two statement-local lint expectations document why descriptor
inheritance tests must inspect underscore-named RAII fields. The declared lint
policy, public production signatures, dependencies and cache bytes are unchanged.

Full Linux `make ci lint check-secrets` passes with the new gate, including 146
unit tests, 14 real Cargo/Wasm cases, process/retention suites, rustdoc, actual
Rust 1.88.0 and isolated release-adapter fixtures. Receipt:
`/tmp/ic-build-014-full-validation.log`; strict initial/final diagnostics and
before-images are retained under `/tmp/ic-build-014-*`. Cargo.lock remains
`c0c98783fc94d305034cef7788a7558a5629aacef8c8ca039d2a638ab5fcd235`.
Explicit setup in the retained private Rustup fixture also prepares Clippy while
keeping Wasm installed; receipts are `clippy-install.log` and
`clippy-components.log` under `/tmp/ic-build-6-fresh-rustup.FGmFTo/`.

Published 0.1.3 now has the same heartbeat failure on Linux and native macOS
Apple Silicon; Intel remains queued at inspection. The delivered 0.1.4 native
matrix still needs to qualify these working-tree corrections. Metadata remains
0.1.3, the numbered draft remains 0.1.4, and Git/release effects are user-owned.
No sibling source was changed in this batch. Earlier sections retain their
historical source/graph and qualification state.

## Published 0.1.3 and current consumer qualification — 2026-10-13

Version 0.1.3 is published from
`3248364dccc9bdb9b56a666f86cc7682b4dead42`. The cached crates.io archive's
SHA-256 is
`1527d2dfe308fc7c1236b6db42c8c6883635f0d60d1dbbabd28462648058caec`;
its VCS receipt names that release and its staging source/tests match this
checkout. The published dependency graph selects Host 0.16.0. Earlier Linux
receipts below qualified Host 0.15.2 and remain historical evidence.

The authorized Canic #529 replacement is applied against registry 0.1.3, with
no local producer override or parallel staging engine. Canic keeps its recipes,
read/output policy, release binding and qualification-first durable publication.
Its current graph selects Host 0.16.0, Memory 0.36.1, Testkit 0.36.0,
Blob/contracts 0.23.0 and Timers 0.21.4. Concurrent catalog updates are preserved.

All 45 focused consumer checks pass in a frozen checkout with a separate target:
23 artifact cases, two real Cargo-output cases, three release-binding cases and
17 declaration/cache cases including the real extractor. One governed PocketIC
binding case remains ignored. Strict Host all-target/all-feature Clippy and
changed-file formatting pass. A final custody check matches 1,592 owning-package
Rust/manifest/config inputs to live Canic; receipts are under
`/tmp/canic-529-registry-qualification/` and `/tmp/canic-529-registry-*.log`.
Three consumer production adapters shrink by 25 code lines. Canic's handoff and
0.112 draft own consumer evidence; native/application acceptance remains pending.
Package versions, staging, Git delivery and further publication remain user-owned.

The [release CI run](https://github.com/dragginzgame/ic-build/actions/runs/38373448100)
has a failed Linux job and queued macOS jobs at inspection. Linux again fails
the real positive-heartbeat build with a missing Wasm standard library, despite
the released prepared-compiler selection. Rustup 1.29.1's no-channel install
branch reads required targets from the toolchain file and does not apply the
command's target/component flags. The released file omitted the Wasm target.

The working-tree #6 follow-up declares `wasm32-unknown-unknown` there. A fresh
private Rustup home under `/tmp/ic-build-6-fresh-rustup.FGmFTo/` reproduces the
old setup's successful installation without Wasm, then the same explicit setup
installs Wasm with the amended file. Its real positive-heartbeat test passes
with that private compiler and no inherited toolchain override. The user's
installed compilers, global default and targets are untouched.

Full Linux `make ci lint check-secrets` passes on the released Host 0.16 graph,
including 146 units, 14 real Cargo/Wasm cases, process/retention suites and actual
Rust 1.88.0. Receipt: `/tmp/ic-build-013-host016-final-ci.log`. Metadata remains
0.1.3; the current numbered 0.1.4 draft covers this setup correction. Native
macOS acceptance and #6 closure still require delivered CI results. Historical
working-tree/version claims below describe their recorded source state.

## Historical compiler qualification

Version 0.1.2 is published from `24c5f49a1684e62f67a659a931a628ada6cd2cdd`;
package metadata remains 0.1.2 and the next pending notes are 0.1.3. The later
sections preserve prior batch receipts, including historical version and
working-tree references.

The working-tree repair for
[#6](https://github.com/dragginzgame/ic-build/issues/6) selects the prepared
compiler in the `make test` entrypoint and passes its absolute path as `RUSTC`
to the test command. This existing caller-environment contract covers both
identity probes and Cargo builds in all independent fixtures. Heartbeat
zero-interval rejection and the real positive-interval build remain unchanged;
the library API, cache formats and process implementation are unchanged.

The original heartbeat failure was reproduced locally with inherited compiler
selection removed. The ambient stable toolchain lacked the Wasm standard library,
while the repository-selected 1.99.0 toolchain had it. An initial fixture-only
fix exposed the same cause in 11 integration cases; the common entrypoint now
qualifies all 141 unit tests and all 14 real Cargo/Wasm cases with a direct
prepared Cargo executable and no inherited toolchain/compiler selection.
The focused heartbeat invocation also passes from that ambient environment
with the selected `RUSTC`. No global default or installed targets were changed.

The standard full Linux `make ci lint check-secrets` passed, including actual
Rust 1.88.0 and release-adapter fixtures. Evidence remains under
`/tmp/ic-build-6-ambient-qualification.puIRI8/`; `compiler-selection.log`,
`focused-after.log` and `full-validation.log` own the successful receipts.
`ambient-direct-cargo-gate.log` retains the adversarial suite pass and a
qualification-harness-only MSRV dispatch failure: direct Cargo cannot interpret
rustup's `+1.88.0` syntax. Normal MSRV dispatch subsequently passed.
The original failures remain in `/tmp/ic-build-6-ambient-before.log` and
`/tmp/ic-build-6-ambient-suite-before.log`.

The authorized Toko adoption finding was added to existing
[Toko #1796](https://github.com/dragginzgame/toko/issues/1796#issuecomment-6140005110)
without sibling file changes. Git delivery remains user-owned; this repair is
uncommitted and native macOS qualification requires CI against the delivered
change. Local Linux evidence does not establish native macOS acceptance.

The maintainer separately authorized implementing
[Canic #528](https://github.com/dragginzgame/canic/issues/528). Canic now delegates
declaration caching to its existing registry IC Build 0.1.1 transaction API;
private JSON persistence is removed in the same working-tree change. Its recipe
adapter retains optional-cache fallback, input/tool custody and bounded workers.
All 17 scoped extraction/cache tests, including real extractor byte parity, and
strict Host Clippy pass on Linux. Detailed custody and qualification live in
Canic's handoff and the issue; native macOS acceptance remains pending. This
consumer change needs no new IC Build API or dependency update and does not
expand the producer's pending 0.1.3 notes beyond the compiler-selection repair.

## Temporary staging producer and pending Canic adoption

Pending 0.1.3 now also prepares temporary artifact staging and producer-output
capture for [Canic #529](https://github.com/dragginzgame/canic/issues/529).
`ArtifactStagingDirectory` owns an invocation's temporary child and explicit
close/best-effort Drop cleanup. `CapturedArtifact` keeps exact copied producer
bytes alive across later source replacement, with a caller-selected read budget.
These APIs do not publish/reuse cache entries, select recipes, prune retained
artifacts or admit a release. No dependencies or package versions were added.

The full Linux `make ci lint check-secrets` passes on the maintainer's current
staged Host 0.15.2 lock graph, including 146 unit tests, the existing real
Cargo/Wasm/process suites and actual Rust 1.88.0. Its receipt is
`/tmp/ic-build-529-final-ci.log`; the five focused staging tests and updated
docs/format checks are retained separately. Optional strict Rust Clippy still
reports 43 distinct existing diagnostics. A disposable pre-staging baseline on
the same graph produces identical normalized diagnostics; the new module adds
none. [IC Build #8](https://github.com/dragginzgame/ic-build/issues/8) owns that
independent follow-up.

The Canic replacement is prepared as `/tmp/canic-529-adoption.patch` with its
source candidate and independent qualification checkout under
`/tmp/canic-529-candidate/` and `/tmp/canic-529-review/`. Its compiled-output,
artifact qualification and release-binding checks plus strict Host Clippy pass
in isolation (22 passing checks, with the governed PocketIC case still ignored).
That frozen consumer graph selects Host 0.15.1 and Memory 0.35.19; unrelated
live Canic catalog/lock updates are preserved and need qualification when the
patch is adopted. The producer is represented as 0.1.3 only in the disposable
qualification fixture; real metadata stays 0.1.2. This is candidate evidence,
not registry adoption or native macOS acceptance. The live Canic staging path
has not been removed yet: publish the new API, then update its registry
selection/lock and apply the qualified replacement and retirement together.
Canic retains its source-read policy, transform/output limits, release binding,
qualification and durable publication order. No live source override or dual
compatibility pipeline is introduced. User-owned staging and other dirty work
remain untouched; the new API and notes are unstaged.

## CI dependency preparation

Version 0.1.1 is published from `dbb17982557174eed29e1a499a2b1a7bb278d5a7`.
The next pending notes are 0.1.2; package metadata remains 0.1.1. The sections
below preserve earlier extraction/delivery evidence, including their historical
version and working-tree references.

The current working-tree repair for
[#5](https://github.com/dragginzgame/ic-build/issues/5) adds explicit
`cargo fetch --locked` preparation on every CI matrix host before the offline
gate. Fetch output is retained with the existing failed-validation evidence.
Developer setup now documents the same prerequisite; validation remains offline
and does not install dependencies implicitly.

A private checkout of the released source with the workflow/changelog repair and
an empty Cargo home reproduced the missing `fs2` failure before preparation.
Locked fetching then enabled the full Linux `make ci lint check-secrets` gate,
including 141 unit tests, real Cargo/Wasm/process suites, actual Rust 1.88.0 and
release-adapter fixtures. The released lockfile checksum stayed
`c1704c433e54f8bd20c054561570f6e7ed85cf8fbfa165093a6b55eb4429a51e`
before fetch, after fetch and after the gate. Evidence remains under
`/tmp/ic-build-ci-cache-repair.Vn3mPr/`, including `before-fetch.log`,
`dependencies-fetch.log` and `full-validation.log`.

The real checkout's existing Host 0.15.1/syn 3.0.9 lockfile update was preserved
unchanged and was not substituted for the released graph in this validation.
The repair remains uncommitted; no package version, real index, Git refs,
publication, CI dispatch or retained artifacts were changed. Native macOS
qualification still requires CI against the delivered repair; local Linux checks
do not establish it.

## Generic engine extraction

The maintainer authorized moving generic build/artifact code from IC Testkit
and Canic into this repository. The extracted implementation is in
`crates/ic-build/`; Cargo metadata currently selects version `0.1.0` and permits
registry publication (`publish = true`). Version 0.1.0 is published and selected
from the registry by Testkit and IcyDB working trees. Its cached archive checksum
matches their tracked lock entries. Release/publication commands remain
maintainer-operated. The changelog preserves the original 0.1.0 scaffold notes as
historical evidence; the published archive also contains the extracted engine.
Pending 0.1.1 covers subsequent corrections, documentation and tooling work.
The Testkit namespace removal retains its separately documented minor boundary.
The maintainer selected 0.1.1 for these notes; Cargo metadata stays at 0.1.0.
[IC Build #1](https://github.com/dragginzgame/ic-build/issues/1) owns extraction
acceptance and follow-up. The [ownership design](../design/build-ownership.md)
is the architecture and historical audit evidence.

The engine owns Cargo input resolution, exact Wasm caches, transactional
artifact sets, retained records, sessions, batch outcomes, freshness and explicit
maintenance. Canic's feature-equivalent Cargo grouping also lives here; callers
supply the Cargo command/environment, target and output policy. The engine
requires neither IC Testkit nor Canic nor PocketIC.

Consumer migrations were applied as precise static patches against their current
working files after validation in isolated source checkouts and verification of
exact before-images and backups. Testkit removes its artifact module and
moves generic tests/examples to this owner, with direct IC Build imports in its
remaining fixture recipes. Canic migrates direct artifact
consumers and replaces its grouping implementation with a policy adapter.
There are no compatibility reexports or remaining Testkit artifact callers in
the authorized consumer Rust sources.

Testkit keeps PocketIC provisioning, lifecycle, diagnostics and benchmarks.
Canic keeps recipes, generated infrastructure, build environment, source custody,
credentials, release identities and complete-release admission. Its separate
production complete-release reuse/finalization path is retained: a generic cache
hit does not prove a valid Canic release. Converging that path requires preserving
its additional dependency, before/after, Candid and release invariants. IcyDB's
read-only working tree at `bf444afeb4634dacc809396dbdc8c10cc2e984f4` now selects
registry IC Build and imports its artifacts directly. Its owner is qualifying
that dirty migration; source inspection does not establish acceptance or delivery.

## Retained contracts and dependency custody

Existing `ic-testkit-wasm-build-v1`, `ic-testkit-watched-input-v1` and
`ic-testkit-artifact-set-v1` digest/stamp domains, bytes, `.ic-testkit` cache paths
and lock namespaces remain unchanged. No alternate reader, implicit migration or
artifact cleanup is introduced. Records retain artifact ownership through use;
failures, cancellation, input-change rejection, feature-safe batching and explicit
maintenance remain with the extracted engine. Builds have no finite default
elapsed-time deadline. The Testkit #66 lock-test isolation repair is preserved.

The virtual root workspace owns edition 2024, Rust floor 1.88.0, MIT licensing,
shared dependency selections and the lockfile. License metadata inherits SPDX
`MIT`; the package-local regular `LICENSE` preserves the repository's original
notice and is compatible with the shared formatting hook. The redundant
`license-file` field is removed to avoid Cargo's manifest warning.
IC Build selects Host 0.15.0,
matching the independently prepared Canic and
Testkit source boundaries. Consumer lockfiles retain existing dependency versions
while reconciling the explicit source inputs and relocated engine. Both consumer
graphs compile; retained transitive Host 0.14 dependencies do not own the engine.
Canic's incubation source paths and evidence are explicit dependency-pin exceptions;
Testkit now selects registry IC Build 0.1.0 with a tracked checksum.
No workspace package versions have been changed by this extraction. Testkit's
published 0.34.7 history and ongoing Host selection are preserved; the artifact
namespace removal is documented under pending 0.35.0 notes. Canic's existing
0.112.0 draft is extended. Consumer delivery requires the appropriate version
boundary and registry alignment; IC Build publication does not establish consumer
acceptance.

## Qualification and evidence

The extracted package passed `make ci` and `make lint check-secrets` on Linux
x86-64, including the shared snapshot, tool admission, documentation links,
manifest/pin inheritance, formatting, all-target compilation, rustdoc with
warnings denied, behavioral/process tests and compilation with actual Rust/Cargo
1.88.0. The initial extraction gate included 139 unit tests, cross-process cache locking,
retained-artifact handoff, helper coverage, 14 real Cargo/Wasm cases, shared-target
process coordination and a doctest. Ignored worker entrypoints execute through
the parent process tests; synthetic producer fixtures cover failure/race cases.
The real Wasm cases use independent Cargo fixture workspaces, not a silently
missing Testkit perf package. The artifact API guide is [here](../artifacts.md).

The current local issue-repair gate passed `make ci lint check-secrets`, with
141 unit tests and the existing Cargo/Wasm/process suites, plus actual Rust
1.88.0 compilation and release-adapter fixtures. Cargo grouping now returns
Host's original typed nonzero failure with status and bounded output, and rejects
duplicate/empty package inputs before invoking the command factory. Regression
checks cover truncation evidence, malformed forests, typed capture/spawn failures
and real Cargo feature-safe grouping. The README's direct registry/import example
also compiled through rustdoc. Current public documentation and the GitHub
description acknowledge published 0.1.0; its immutable registry README retains
the old text until a subsequent package is published. Evidence is retained in
`/tmp/ic-build-issues-full-validation.log`. These repairs are working-tree changes;
release identities remain unchanged. Native macOS and consumer
acceptance remain with the linked extraction issue.

Testkit all-target compilation passed in both the isolated review and live
workspace against its current 0.34.7 source and selected Host 0.15.0 graph. Its
18 benchmark-driver tests also passed in the live workspace, with six PocketIC
diagnostic unit checks passing in isolation. All-target compilation passed in
both isolation and the live workspace for Canic Host,
Testing Internal and Tests against the extracted engine and changed Testkit
producer. Nine focused Canic tests passed in the live workspace, exercising
retained-artifact handoff, cache timing and generated-input/build-policy custody
without starting PocketIC. Both consumers' dependency-pin and focused
source-format checks passed. Full consumer/PocketIC
suites remain maintainer-owned and were not run.

Review checkouts, exact live before-images, hashes and static consumer patches
are retained under `/tmp/ic-build-extraction-review.18ss90/`; focused logs are
`/tmp/ic-build-testkit-live-check.log`, `/tmp/ic-build-testkit-live-driver.log`,
`/tmp/ic-build-canic-live-check.log`, and `/tmp/ic-build-canic-live-targeted.log`.
The complete IC Build gate is recorded in `/tmp/ic-build-extraction-final-ci.log`.
Earlier engine test/MSRV logs and failed
validation evidence remain retained. These local paths are evidence custody,
not portable CI receipts.

CI is configured for Linux x86-64 and native macOS 15 Intel/Apple Silicon, with
explicit Wasm-target and declared-MSRV preparation. Native macOS extraction
qualification has not run; Linux does not substitute for it. Shared Tooling 0.9.5
is pinned at `b1630b89482cc6071fc928a86962d79c9c11fdb7` with exact snapshot
digests. All common tools and pinned Linux validation tools are prepared offline;
shared helpers have no consumer edits. The development compiler remains Rust 1.99.0.

The 0.9.5 adoption used a clean, detached source checkout and retained the 86-file
entrypoint closure. The sibling's dirty 0.9.6 work was excluded. Existing tool
pins, package metadata and extracted Rust sources were preserved. Snapshot/tool
admission, documentation links, dependency pins, formatting, release routing and
shell/workflow lint passed on Linux. Canonical checksum-refusal and release-runner
stub fixtures passed against the exact adopted helper bytes; curl's transient
recovery, exhaustion, permanent failure and checksum refusal passed through a
bounded loopback fixture. These fixtures perform no real release or tool install.
Evidence is retained in `/tmp/ic-build-shared-tooling-095.hFZahO/` and the
`/tmp/ic-build-shared-tooling-095-*.log` validation logs. The committed 0.1.0
scaffold changelog section is preserved verbatim beneath pending 0.1.1 notes.

`make check`/`make ci` include the actual package tests and isolated consumer
release-adapter checks. The bootstrap refusal is replaced by direct-delivery
metadata adapters using the shared runner, source checker, changelog finalizer
and local-lock writer. The snapshot now records 87 files. The metadata set is
exactly `Cargo.toml`, `Cargo.lock` and `CHANGELOG.md`; package versions and
registry selections are checked across preparation and selected-commit recovery.
Publication requires clean HEAD at the annotated workspace-version tag and is
separate from release delivery. No adapter cleans retained artifacts. The
[release guide](../publishing.md) owns operating instructions; native macOS
qualification remains outstanding. This batch prepares adapters, not a release.
The full Linux `make ci lint check-secrets` passed with these adapters, including
the actual nested validation logger and disposable Git/index/tag fixtures.
The fixtures cover metadata rollback, unchanged external lock entries, saved
candidate replay, selected older-commit checks, publication refusal/failure and
retained artifacts. Logs are retained at `/tmp/ic-build-release-full-validation.log`
and `/tmp/ic-build-release-adapter-test.log`; failed fixture attempts remain in
their printed temporary directories. No live release or registry request ran.
Commit the adapter changes before invoking a normal release: preflight correctly
refuses implementation edits outside the three declared metadata files.
The public repository is established; the maintainer committed the earlier
package layout as `f4ae89c` and extraction/tooling as `d4f6222`. The maintainer
has now requested committing and pushing the validated current batch to
`origin/main`; associated issues record delivery identities. This delivery does
not change package versions, release tags, publication or retained artifacts.
GitHub issues are the follow-up tracker; this handoff records ownership and
qualification evidence.
