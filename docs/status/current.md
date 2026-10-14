# Current handoff

## Portable FIFO fixtures — pending 0.17.10

Released Host 0.17.9 still selects Apple-unavailable `rustix::fs::mkfifoat`
in process admission and regular-opener tests. Exact 0.17.8 hosted macOS failure
logs confirm the process compiler error; the same Apple exclusion is present in
the selected rustix source. Both fixtures now use the established native
`/usr/bin/mkfifo` command with their original 0400/0600 modes. Actual FIFO refusal,
marker nonexecution and the filesystem timeout/unblock guard remain intact
([#85](https://github.com/dragginzgame/ic-host-tooling/issues/85),
[#88](https://github.com/dragginzgame/ic-host-tooling/issues/88)).

Focused Linux admission/opener tests, corrected strict fs/process Clippy and
actual Rust 1.88 checks pass. Full combined `make ci msrv-workspace` passes with
unchanged source inputs. Initial leftover-import lint failure and successful
logs are retained with before-images under `/tmp/host-issues01710.Yi1g2U/`.
The Apple log is recovered through the direct job API after empty standard
retrieval; both native compiler errors are retained. Manifest/lock, pins,
snapshot and index remain unchanged. Current 0.17.9 CI passes Linux and MSRV,
but both macOS lanes remain queued on run `38536901339`; existing producer
acceptance issues stay open. Production APIs and behavior are unchanged;
compatible notes select 0.17.10 without changing package versions or lock inputs.
Native macOS qualification of the corrected source remains separate. No Git,
release, publication, executable download or sibling mutation ran.

## Shared Tooling adoption — delivered 0.17.9

Shared 0.12.2 at `cff186285829481185e932d3c16514022b897de9` is exported
from a clean isolated copy of pushed upstream main. The dirty sibling changelog
and tooling-inventory fixture are excluded. Existing roots remain selected;
the closure now contains 109 payloads, including the failure-evidence collector.
Historical changelog admission and documentation companion export are updated.
Host's publication admission, package APIs, tool pins and release policy remain
local ([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73)).

Full combined offline Linux `make ci` and Rust 1.88 workspace checking pass.
Release-adapter, shared release-runner and actual evidence-collector fixtures
also pass under genuine Bash 3.2 with substituted external effects. Snapshot
integrity, repeat preview (109 KEEP), shared/local documentation links and strict
ShellCheck pass. Recorded Rust, lock, pin and validation inputs stayed unchanged,
and the index is preserved. Before-images, preview and logs are retained under
`/tmp/host-shared0122.UOVeFV/`, including `full-validation.log` and
`msrv-workspace.log`. Incoming publication, process-fixture and regular-opener
changes are included in the combined gate and preserved. Own versions remain
0.17.8 and compatible pending notes remain 0.17.9. Native macOS qualification
stays separate; no commit, release, publication, executable download or sibling
file mutation ran.

## Following regular-file opener for Blob — delivered 0.17.9

The Blob maintainer approved the scoped extraction under
[#88](https://github.com/dragginzgame/ic-host-tooling/issues/88).
`read::open_regular_file` follows selected symlinks, admits the opened regular
file and returns its read-only descriptor. Unix FIFO opening cannot wait for a
writer before admission. Existing following reads/hashes reuse the opener;
limits, expected lengths and separate no-follow/private contracts remain intact.
The two source files match their frozen qualified counterparts.

Seventeen focused read cases, full frozen Linux `make ci` and actual Rust 1.88
fs all-target/all-feature checking pass. Original compile and temporary-selector
failures remain in `/tmp/blob-host-opener-_1fkrkee/`; the frozen incoming process
manifest was sorted only in that copy before its successful gate. The concurrent
real Host gate is excluded from this extraction's qualification. Native macOS
remains separate. A frozen Blob replacement passes 115 CLI tests and strict lint;
its initial unused-import lint failure and corrected retry are retained.
Blob keeps its local opener until a published Host selection exposes the API;
consumer retirement is [Blob #67](https://github.com/dragginzgame/ic-blob-storage/issues/67).
Own versions, lock selection, index/refs and publication remain untouched.

## Publication and initialization cleanup — delivered 0.17.9

Host delegates its admitted five-package Cargo command to the selected Shared
0.12.0 attempt owner. Eligibility, licenses, package provenance, fresh remote
retrieval, final source comparison and the wider publication lock remain local.
Admission evidence links to the fresh command attempt, which owns exact selectors,
dispatch and separate Cargo/logger statuses. Cargo failure takes priority; an
interrupted attempt retains its stable lock and refuses a fresh invocation until
explicit reconciliation. No retry, upload or registry request is added to tests
([#75](https://github.com/dragginzgame/ic-host-tooling/issues/75)).

Spawn paths reuse direct-child state initialization before applying their own
group/cleanup authority. CLI argument fixtures pass a borrowed iterator directly.
Focused child and CLI checks pass; publication rejection/status/interruption
fixtures pass under current Bash and genuine Bash 3.2. Full offline Linux
`make ci msrv-workspace` passes in the task-only frozen checkout and then in the
combined real checkout including the concurrent #87 fixture module. The combined
manifest is sorted and its new files participate in the formatting fixture;
both sets of notes share one 0.17.9 heading. Recorded inputs stayed unchanged
through that gate. Initial formatting/preparation/fixture-propagation failures
and successful final logs are retained with before-images in
`/tmp/host-publish-reuse.kdo6JY/`. The combined log also includes both concurrent
regular-opener regressions and all 84 filesystem cases; those implementation
changes belong to their separate extraction work.
Own versions stay 0.17.8, with compatible pending 0.17.9 notes. Native macOS remains
separate; no Git, release, publication, executable download or sibling edit ran.

## Custody-aware direct-child transfer — delivered 0.17.8

The Canic maintainer explicitly authorized the scoped Host bridge for IC Backup's
custody-aware spawn. `OwnedChild::from_spawned_direct` takes the existing child
into Host's bounded capture and cleanup owner without a second spawn or process-
group authority. Original IO and pid are retained; Host owns reaping after transfer.
All three direct-child cases and strict process library/test Clippy pass on Linux.
Evidence: `/tmp/canic-112-completion/host-child-real-tests.log` and
`/tmp/canic-112-completion/host-child-real-clippy-final.log`. Those scoped checks
are separate from native macOS acceptance. The API is published in 0.17.8;
Canic adoption remains owned by [#513](https://github.com/dragginzgame/canic/issues/513).
The scoped implementation did not change package versions, lock selection,
index or refs.

## Shared Tooling adoption — delivered 0.17.8

Shared 0.12.0 at `f7fe31bf80a2b445bf37a470527f001210241aea` is exported
from a clean isolated checkout of pushed upstream main. The snapshot has 90
entrypoints and 106 payloads, including the new literal catalog reader and two
linked guides. Host delegates numbered/top-entry changelog admission to the
strict canonical finalizer and removes equivalent local guards. Source-bound
receipts, exact metadata projection and release recovery remain local. The
optional publication runner and catalog checker were not selected in 0.17.8;
the pending publication cleanup now selects the runner and its fixture
([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73),
[#75](https://github.com/dragginzgame/ic-host-tooling/issues/75)).

Current Bash and genuine Bash 3.2 release-adapter/shared-runner checks pass with
substituted setup/Git effects. The updated IC fixture also passes under Bash 3.2.
Full combined offline Linux `make ci`, snapshot integrity, repeat preview (106
KEEP rows), shared/local documentation links and diff checks pass. The unchanged
Rust/lock inputs retain the preceding Rust 1.88 workspace evidence in
`/tmp/host85.JYnUQC/`; no new compiler requirement is introduced.
Evidence, task before-images and preservation hashes are in
`/tmp/host-shared0120.maRft6/`. The index and selected Rust/tooling inputs are
unchanged through qualification.
The incoming #84/#85 changes, lock selection, package versions and tool pins are
preserved. Compatible notes stay 0.17.8; no library migration is required.
Native macOS acceptance remains separate. No Git or release effects ran.

## Trusted fixture admission cleanup — delivered 0.17.8

Process and extraction fixtures delegate trusted local executable observation to
`admit_version`; their preliminary hash and constructed digest pin are removed.
Version cases share focused coverage, while the ordinary pinned-admission test
also checks successful identity/version and mismatched-version evidence. No
production API, executable policy, fixture serialization or capture bound changes
([#85](https://github.com/dragginzgame/ic-host-tooling/issues/85)).

Focused Linux process and extraction checks and full combined `make ci
msrv-workspace` pass, including strict workspace Clippy/docs and Rust 1.88.
Evidence, validation input hashes and task before-images are retained under
`/tmp/host85.JYnUQC/`. The incoming #84 repair is preserved. A concurrent lock
selection updates `equivalent` from 1.0.2 to 1.0.3; it is outside this task's edits
and included in the combined qualification. Rust inputs and the lock stayed
unchanged through the gate.
Native macOS qualification remains separate. Package versions remain 0.17.7;
pending compatible notes stay 0.17.8. No Git or release effects ran.

## Checked native permissions repair — delivered 0.17.8

The maintainer authorized the [#84](https://github.com/dragginzgame/ic-host-tooling/issues/84)
repair from IC Build. Against released 0.17.7 source
`2bdc5f1cae2c12040bf08d03b5c68502291599cd`, retained staging now reuses the
existing checked native-permissions helper, including Apple narrowing. Public
signatures, refusal order, exclusive/no-follow creation, synchronization and
retained evidence remain unchanged. High-bit permission refusals extend the
existing test; no second conversion owner was added.

Full Linux `make ci msrv-workspace` passes, including 82 filesystem tests,
strict workspace Clippy/docs, feature and release/tooling fixtures, and actual
Rust 1.88 all-target/all-feature workspace compilation. The lockfile is unchanged.
Evidence and before-images are retained under `/tmp/ic-build-host84/`;
`host-full-validation.log` records the full gate. Native macOS 15 Intel/Apple
Silicon qualification and publication remain outstanding. Pending compatible
notes are 0.17.8; own metadata remains 0.17.7. IC Build must adopt the published
repair before its exact native gates can qualify. No Git or release effects ran.

## Retained-staging primitive for Blob — delivered 0.17.4

`create_retained_at` creates a caller-named exclusive stage beneath an admitted
parent descriptor. `publish_retained_at` consumes its held file, verifies regular
single-link identity, synchronizes and reuses the existing no-replace publisher.
Producer failure/drop leaves evidence intact; no retry or content policy is added.
The formatting fixture explicitly includes both new module files before commit.

Three focused tests and strict fs Clippy pass. Full Linux `make ci` passes in
`/tmp/blob-owner-cleanup/host-check`, a frozen released 0.17.3 source plus this
primitive and formatter inputs. It excludes concurrent release-adapter/CLI work;
that other work is not qualified by this run. Earlier tool-directory and omitted
module fixture failures are retained alongside `/tmp/blob-owner-cleanup/host-ci-complete.log`.
Blob's isolated consumer replacement passes 119 CLI cases and Clippy. The real
Blob checkout retains its current publication path until this API is published.
No native macOS, commit, release, publication or actual provider operation ran.

## Released source and baseline

The latest finalized release is **0.17.9**, commit
`6fd078d9b914fff530b1efe521dc08ce18b97cc0`. Package versions remain unchanged.
Shared Tooling **0.12.2**, revision
`cff186285829481185e932d3c16514022b897de9`, is adopted through the canonical
exporter from a clean isolated checkout of pushed upstream main. The snapshot
contains 109 payloads, including the shared release-tag checker, fixture
lifecycle helper, selected publication attempt owner and evidence collector.
Tool pins are unchanged by adoption.
See [the snapshot](../../.shared-tooling.snapshot) and
[AGENTS.md](../../AGENTS.md).

## Delivered 0.17.3 tooling

Cargo CI cache keys now ignore unrelated pins, comments and record order in
literal catalogs, while preserving Cargo selections and build identity as inputs.
Complex catalogs retain byte-based keys and are never executed by inspection
([Shared #154](https://github.com/dragginzgame/shared-tooling/issues/154)).
This is compatible contributor tooling; crate source, Cargo metadata, common
catalogs, Makefile and library contracts are unchanged from released 0.17.2.

The 0.10.2 sibling-dashboard fix is outside Host's selected snapshot. The sibling
checkout's uncommitted 0.10.3 work was excluded by exporting the exact published
0.10.2 commit. Snapshot integrity, repeat preview (102 KEEP rows), preservation
hashes, documentation links and full offline Linux `make ci` pass.
Adoption evidence is retained in `/tmp/host-shared0102/`.
Prior 0.10.1 full offline Linux and genuine Bash 3.2 qualification remains in
`/tmp/host-shared0101/`; no tool pin, product source or dependency change is needed.
Native macOS 15 Intel and Apple Silicon qualification remains separate from
local Linux evidence. No commit, push, release or publication ran in this task.

## Delivered 0.17.7

Observed digest verification has one artifact owner. Six local fixture handlers
reuse the adopted cleanup helper, and release/publication workspace skeletons
derive from Cargo metadata. Caller policy and actual trap/rejection probes remain
local. Identity/tool checks, strict Clippy, actual trap/release/publication probes
under current Bash and Bash 3.2, workspace Rust 1.88 and full combined Linux
`make ci` pass. Inspection-example digest/limit checks also pass. Evidence is
retained in `/tmp/host82/`; native macOS qualification remains separate
([#82](https://github.com/dragginzgame/ic-host-tooling/issues/82)).

Git capture shares ordered failure handling, preserving earlier and current
query evidence. Malformed-tree and failed-status cases extend the existing
fixtures. Make jobserver/execution probes share target lists and cover workspace
MSRV, feature profiles and the CLI. The publication guide describes all selected
packages and both source layouts. Focused provenance tests, strict process Clippy,
ShellCheck, Bash 3.2 Make probes, Rust 1.88 process checks, documentation links
and full current-checkout Linux `make ci` pass. Evidence is in
`/tmp/host83/`; native macOS qualification remains separate. No consumer migration
is required ([#83](https://github.com/dragginzgame/ic-host-tooling/issues/83)).

## Delivered 0.17.6

Shared 0.11.3 changes two selected documentation payloads only. The optional
publication-admission checker is outside Host's selected roots and has different
tag/registry requirements from Host's current publication contract. All executable
payloads match 0.11.2. Snapshot verification, repeat preview (103 KEEP), links
and diff checks pass. Adoption evidence is in `/tmp/host-shared0113/`; existing
native qualification is not relabelled by this documentation-only update
([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73)).

## Delivered 0.17.5

Shared 0.11.2 centralizes annotated-tag validation in release recovery and
cleanup/retention in its selected fixtures, preserving cleanup/diagnostic failure
status and physical temporary roots. Host-owned fixture migration remains
a separate candidate. Full current-checkout Linux `make ci`, snapshot verification
and repeat preview pass. The committed upstream retention suite passes under
current Bash and genuine Bash 3.2. Evidence is retained in `/tmp/host-shared0112/`;
native macOS qualification remains separate. This is compatible contributor
tooling with no library migration or changed tool pins
([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73)).

## Delivered 0.17.4

Workspace-wide CI/MSRV selection and source-bound lock inventory remove stale
four-package assumptions. Publication retains one explicit allowlist and records
all selected packages before dispatch. Equivalent tools feature checks share one
loop; artifact-example hashing operates directly on its bounded owned slice.
Focused adapter/publication/feature checks, real isolated cargo-set-version
projection, workspace MSRV and full current-checkout Linux `make ci` pass.
Evidence is in `/tmp/host-next-three/`; native macOS remains separate. These changes and the
Shared 0.11.0 adoption are compatible, requiring no library consumer migration
([#75](https://github.com/dragginzgame/ic-host-tooling/issues/75),
[#73](https://github.com/dragginzgame/ic-host-tooling/issues/73)).

Example hashing and fixed tools-fixture admission are simplified
([#78](https://github.com/dragginzgame/ic-host-tooling/issues/78)). Focused tools
checks, direct example boundary checks and full Linux `make ci` pass in an
isolated 0.17.3 copy containing only these three source edits. Evidence is in
`/tmp/host78/`; concurrent source changes and native macOS remain unqualified
by that run. This compatible cleanup needs no consumer migration.

Release payload comparison is consolidated in one helper; working-tree admission
and historical-commit materialization remain separate. Focused adapter checks
pass, including a disposable integration with the proposed strict Shared
finalizer. Upstream work is owned by [Shared #162](https://github.com/dragginzgame/shared-tooling/issues/162)
and [Shared #163](https://github.com/dragginzgame/shared-tooling/issues/163), with
Host coordination in [#75](https://github.com/dragginzgame/ic-host-tooling/issues/75).
The strict-finalizer proposal is now delivered in Shared 0.12.0 and adopted here.
Its earlier patch evidence remains in `/tmp/host-release-convergence/`. Host's
publisher now delegates command-attempt mechanics in the pending 0.17.9 cleanup,
retaining its independent admission and reconciliation policy.
The earlier combined gate stopped on concurrent new
filesystem source absent from its tracked formatting fixture; the maintainer
selected release-only validation while those changes continue. Evidence is in
`/tmp/host-release-convergence/`; native macOS qualification remains separate.

The generic bounded Wasm-inspection CLI is published as `ic-host-wasm-inspect`
0.17.4, source `28f3a6d5771ecb941b86cd3c6ea2507de82b101e`. Argument/report
and actual CLI admission fixtures accompany the source transferred from Metrics.
Filesystem/artifact owners compose without Candid parsing or process execution;
response-only defaults stay intact. Metrics has admitted that exact published
selection through the standard managed installer and removed its private package
in its working tree. Consumer delivery qualification and hosted native acceptance
remain separately evidenced by [#81](https://github.com/dragginzgame/ic-host-tooling/issues/81).
The frozen producer integration passes full Linux CI and all five Rust 1.88 package
floors; this does not relabel later release/native workflow results.

## Evidence and coordination

Released 0.17.2 cleanup passed focused process, extraction and IC-limits tests
and full offline Linux `make ci`. Fixture retention probes and documentation
checks passed; evidence remains in `/tmp/host-0172-cleanup/` and
`/tmp/host-0172-example/`. Existing qualification owners are
[fixture cleanup #72](https://github.com/dragginzgame/ic-host-tooling/issues/72),
[fixture pruning #74](https://github.com/dragginzgame/ic-host-tooling/issues/74),
[Shared adoption #73](https://github.com/dragginzgame/ic-host-tooling/issues/73),
[interface consumers #71](https://github.com/dragginzgame/ic-host-tooling/issues/71)
and [lock isolation #70](https://github.com/dragginzgame/ic-host-tooling/issues/70).

The [historical handoff](archive-2026-10-13.md) preserves older source identities,
validation paths and consumer coordination. Its status claims describe their
original inspection, not current release state. GitHub issues remain the sole
follow-up tracker. Update this handoff in place instead of appending history.
