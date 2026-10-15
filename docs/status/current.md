# Current handoff

## Typed retained evidence completion — pending 0.18.7

Based on released main 0.18.6 `7f291760ac1184e5235d08f75e3c1aee0fa60cc3`.
The existing retained-file owner now completes caller-bounded production and
file/directory synchronization without staging, replacement, retry or cleanup.
Typed failures distinguish creation, the original producer error and each final
sync. Failed or panicking producers keep their named partial evidence without
claiming failed bytes durable. Names, permissions, schemas, budgets and recovery
policy remain with the caller; platform support and package metadata stay fixed.

Five focused retained-file cases pass, including held-parent relocation,
occupied/symlink refusal and partial-error/panic retention. Strict filesystem
package lint/Rustdoc and actual Rust 1.88 all-target/all-feature compilation pass
on the committed graph. Logs are retained in the authorizing Blob checkout at
`.tmp/review-validation/adoption0270/host-*.log`. Full delivery/native macOS
qualification remains separate; integration and publication remain owner actions.

## PR refreshes against released 0.18.5

Twelve existing PR branches incorporate released main `26bfae9` through normal
topic merges. Focused package/module tests, strict affected-package Clippy/docs,
actual Rust 1.88 checks, formatting, links and snapshot admission pass on Linux.
Their production implementations are preserved; the only authored Rust delta
repairs PR #114's macOS FIFO fixtures with the existing POSIX utility convention.
Its refreshed head passes Apple Silicon CI; Intel qualification remains separate.
Compatible pending notes select 0.18.6; PR #90's breaking private-parent refusal
selects 0.19.0 and preserves the published 0.18 history.

[PR #100](https://github.com/dragginzgame/ic-host-tooling/pull/100) remains a draft
awaiting canonical integration of its Shared helpers in
[Shared PR #205](https://github.com/dragginzgame/shared-tooling/pull/205).
No distributed payload or manifest is hand-merged to bypass that dependency.
Exact branch receipts and focused logs are at `/tmp/host-pr-refresh-20261015/`.
Required hosted checks qualify each updated head separately; integration,
publication and consumer activation remain owner actions. This checkout's
pending Shared adoption, index, package metadata and refs are preserved.

## Shared Tooling 0.15.0 — pending 0.18.6

Released Host base is 0.18.5 `26bfae9ebc23773f53576db367d06478afdea7a1`.
The snapshot now selects pushed Shared main
`2737450d60fe176ea5a620a4e72acaa8f4a99650`, whose committed version is 0.15.0.
All 95 entrypoints and 116 payloads are retained byte-for-byte, including their
executable modes and tool pins. This refresh changes the recorded source identity
and local baseline reference only; it does not require a tool reinstall.

Shared's larger release/fixture integration remains in open
[PR #205](https://github.com/dragginzgame/shared-tooling/pull/205) and is not part
of this source. Its optional license checker is not selected. The requested
changelog records this provenance-only update under compatible patch 0.18.6;
package versions remain unchanged. Earlier pending sections below retain
historical evidence; the 0.18.5 adoption is now released.

Snapshot verification, repeat preview, local documentation links and diff checks
pass. Cargo metadata, pins, index and HEAD are preserved. Evidence is retained at
`/tmp/host-shared015.Tgm4vS/`; unchanged executable and Rust inputs do not need
fresh local execution. Host's complete
[0.18.5 native/MSRV CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38704924922)
now passes; [#124 — Shared release and fixture adoption](https://github.com/dragginzgame/ic-host-tooling/issues/124)
is closed with delivered-source acceptance evidence.

## Integrated Shared release helpers — pending 0.18.5

Released base is 0.18.4 `fb211b5273aee06772b8a0d1b46ca40b4d1b48c2`,
with complete [Linux/MSRV/macOS qualification](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38698314201).
The new snapshot selects pushed Shared main `176992860cbeba806d8d731ee92ea577e3a79b2a`
(committed version 0.14.0), replacing the now-integrated formatting contribution.
All 95 entrypoints remain; seven payloads update and one required release-input
validator brings the closure to 116 payloads. Newer sibling-local commits are not
adopted. Pins, Rust source and Cargo metadata remain unchanged.

Pure release-input and retained-assertion tests, the adopted release runner,
Host adapter and actual fixture retention pass under current Bash and genuine
Bash 3.2. The first Host adapter checks exposed a missing disposable input; its
copy list now includes the validator required by the increment helper. Failed
attempts and corrected evidence are retained at `/tmp/host-shared-refresh.93NcbD/`.
Affected lint/syntax, snapshot, links and repeat-export checks pass. Upstream's
exact native matrix is green; new Host native acceptance remains separate.
[#124 — Shared release and fixture adoption](https://github.com/dragginzgame/ic-host-tooling/issues/124)
owns this local, uncommitted adoption. The real index/HEAD and existing artifacts
are preserved; no tool download, release or publication ran.

## Shared Tooling 0.14.0 — pending 0.18.4

The canonical snapshot now records `6e46528e7d6f758c6de0c885fc2589949f67c2ae`
from the already authorized [Shared PR #196](https://github.com/dragginzgame/shared-tooling/pull/196),
refreshed by normal topic merge of pushed 0.14.0 main `5f70392`. Only unpublished
note placement conflicted; the formatting helper and its tests are unchanged.
Shared main and its sibling working checkout are untouched; integration remains
with the owner. Four selected documentation/reporting payloads change, retaining
95 entrypoints and 115 payloads. No executable helper or tool pin changes.

Focused selector, affected ShellCheck and producer/consumer documentation checks
pass. Snapshot verification and repeat export (115 KEEP) pass. The preceding full
Linux `make ci msrv-workspace` evidence remains applicable to unchanged Rust,
executable and tool inputs; no broad gate is repeated for this documentation-only
refresh. The pending filename fix, Cargo metadata, pins, real Host index and HEAD
are preserved. Evidence is retained at `/tmp/host-shared0140.rihV1h/`.
[#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)
owns Host acceptance; new source-bound native checks remain separate.

## Native filename fixture — pending 0.18.4

Released base is 0.18.3 `0e54003395f628c7234919c1882e7779ccab3369`.
The tree fixture now observes native filename creation: accepted non-UTF-8 names
must be rejected by the walker without visiting their contents; EILSEQ refusal
must leave only the retained regular file, which is still walked successfully.
Other creation errors fail. Symlink rejection and source preservation stay required
on every host. Production source, dependencies and package versions are unchanged.

All 96 filesystem units, strict selected Clippy/Rustdoc and actual Rust 1.88 checks
pass. Full Linux `make ci msrv-workspace` passes; raw logs are retained at
`/tmp/host-tree-fixture-0184/`. These changes remain local and uncommitted.
[#116 — Release and fixture cleanup](https://github.com/dragginzgame/ic-host-tooling/issues/116)
still requires the delivered repair's native macOS 15 Intel/Apple Silicon checks;
Linux exercises the accepting filesystem, not the native EILSEQ branch.

## Literal releases and fixture/walker simplification — pending 0.18.3

[#116 — Release and fixture cleanup](https://github.com/dragginzgame/ic-host-tooling/issues/116)
owns the three approved follow-on changes. Release preparation uses the canonical
literal renderer from the existing `e5a5bb1` Shared source; the snapshot adds one
root/payload (95/115). Host's exact dependency spelling, saved-source comparisons
and receipts remain local. Preparation restores original bytes/modes after failed
rendering, validation following partial install or catchable interruption, retaining
the originals. The release-only cargo-edit pin/installer/commands/fixtures and
native cache salt are retired; existing installations are preserved.

Remaining executable fixture writes/copies use the opt-in Process writer, also
compiled for Process's unit tests without changing its normal default feature
graph. Process/Tools global fixture mutexes and obsolete guard lint expectations
are removed. The unpublished tree visitor loses its directory-open flag callback;
observed directories always use directory-only opens before descriptor rechecks.
Native open errors may precede a visitor type callback on substitution. Backup
retains local walkers until publication and separate consumer qualification.

Source before-images and raw evidence are at `/tmp/host-cleanup-release-tests.XX9gSI/`.
Initial focused Process (96), extraction (20), tree (2) and release checks pass.
Three 32-thread runs each of Process and extraction pass after guard removal.
Strict affected-package lint/docs and actual Rust 1.88 checking pass; stale lint
expectations found by the first Clippy pass are removed, with its failure retained.
Release rendering, post-install failure and TERM rollback checks pass under modern
Bash and genuine Bash 3.2, preserving byte/mode identity. No real release,
publication, own version change, executable download or sibling edit ran.
Full combined Linux `make ci msrv-workspace` passes, including strict workspace
lint/docs, feature profiles and all tests. Final package/lock, index/HEAD and
common tool-pin comparisons are unchanged. Native macOS qualification remains
separate. [Backup #48 — Descriptor walk and crash-barrier adoption](https://github.com/dragginzgame/ic-backup/issues/48)
owns consumer convergence after publication; the sibling source is untouched.

## Formatter source discovery — pending 0.18.3

The maintainer explicitly authorized Git delivery of the prepared Shared change.
[Shared PR #196](https://github.com/dragginzgame/shared-tooling/pull/196) contains
commit `e5a5bb14d8521fc5dc7b5fc4ee6b1f1ca08b5ea6`, based on pushed 0.13.3 main
`d963db4b85b4fa65560fd87e8398f228955a2b71`. Host exports that exact clean committed
source, retaining 94 roots and adding the selector companion for 114 payloads.
The checker, selector, lifecycle companion and guide change; tool pins and all
other selected payloads stay unchanged. The contribution is not merged/released
upstream; its exact snapshot is approved here and must be refreshed if integration
changes its source identity.

Host selects `--rust-sources`, retiring its manual crate/module list while keeping
the selected perturbation and Make/tooling inputs. The newly arrived no-follow
tree/crash-barrier sources are preserved and discovered automatically. The two
prior cleanup slices and the real index/HEAD/package versions remain untouched.
Source before-images and task evidence are at `/tmp/host-formatting-adoption.bNjGQh/`.
[#113 — Fixture and graph simplification](https://github.com/dragginzgame/ic-host-tooling/issues/113)
owns Host qualification; [Shared #195 — Formatting source discovery](https://github.com/dragginzgame/shared-tooling/issues/195)
owns producer acceptance, separately from consumer adoption.

Full combined Linux `make ci msrv-workspace` passes against the adopted snapshot,
including the incoming tree/crash-barrier source. Selected tooling, actual hooks,
strict lint/docs, production graphs and Rust 1.88 checking pass. Selector and Host
hook checks also pass under genuine Bash 3.2; repeat export reports 114 KEEP.
Shared PR lint checks pass; its Linux/macOS regression jobs remain in progress
at final observation. Host native macOS acceptance remains separate.

## No-follow tree and crash-test primitives — pending 0.18.3

Host FS now has a proposed no-follow descriptor tree walker, and Host Process
has opt-in acknowledged direct-child crash barriers. The combined checkout
passes full Linux `make ci msrv-workspace` against the selected Shared Tooling
snapshot, including these APIs. Backup retains its local helpers until Host
publishes them and Backup qualifies that registry graph. Native macOS acceptance
remains separate.

## Fixture and graph cleanup — pending 0.18.3

Released base is 0.18.2 `55a31aa13a42f55fa3985db33e1586722638ad40`.
Six local fixtures use the adopted Shared allocator; retention and aliased-parent
policy remain unchanged. One graph checker retains every profile's exact required
and forbidden dependency selection and separately propagates observation errors.
[#113 — Fixture and graph simplification](https://github.com/dragginzgame/ic-host-tooling/issues/113)
owns the local batch. Package versions, dependencies, tool pins and snapshot stay
unchanged; ordinary implementation leaves the real index and refs untouched.

The third approved cleanup is implemented at Shared's owner in an isolated copy,
with the Host adoption patch prepared against the current snapshot. It adds
opt-in NUL-separated current Rust/manifest discovery, applies tracked deletions
only in disposable checkouts and rejects nonregular/escaping/failed selections.
[#195 — Formatting source discovery](https://github.com/dragginzgame/shared-tooling/issues/195)
owns the producer. Initial automatic approval refusal was resolved by the
maintainer's explicit Git authorization; committed adoption is described above.
Review patches and focused evidence are at `/tmp/host-maint-cleanup.4Wz7Za/`.
Upstream selection and actual Host hooks with an unstaged new crate/nested module
pass under current Bash and Bash 3.2. Native macOS qualification remains separate.

Full Linux `make ci msrv-workspace` passes for the local batch, including actual
Rust 1.88, strict lint/docs and workspace tests. Modern/Bash 3.2 graph controls
reject eight forbidden edges, twelve missing owners and failed Cargo observations;
selector failures retain their status. Retention and actual Host hooks pass under
Bash 3.2 too. Initial probe-formatting and empty-array failures remain alongside
corrected evidence. Final snapshot/link/graph checks and preservation comparisons pass.

## Shared Tooling refresh — pending 0.18.2

Shared 0.13.1 at `a348518c86f1b4cc5184ed94b8213c2d75d5e280` is exported
from an isolated clean copy of pushed main. The snapshot retains 94 entrypoint
roots and 113 payloads: the fixture-lifecycle helper and two guides change.
The publication guide was already selected; its new companion declaration adds
no command or upload effect. Tool pins, dependency selections and pending
certificate/text sources are preserved. The sibling's dirty 0.13.2 work and
rustdoc/probe proposals mentioned in pending upstream notes are not adopted.
The existing [#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)
owns this refresh; ordinary local source changes do not perform Git/release effects.

Full Linux `make ci msrv-workspace` passes for the combined pending source,
including strict workspace lint/docs and actual Rust 1.88 checking. The new
allocator's focused checks pass under current Bash and genuine Bash 3.2;
ShellCheck and a repeat preview (113 KEEP) pass. Before-images and exact logs
are retained at `/tmp/host-shared0131.AuVZvH/`. The real index/HEAD and all unrelated
starting source files are unchanged. Native macOS qualification remains separate.

## Certificate formats and host text — pending 0.18.2

Released base is 0.18.1 `be35654a35f6e2e79728712dd24db76c5daeef02`.
[#109 — Owned certificate decoding](https://github.com/dragginzgame/ic-host-tooling/issues/109)
adds an optional tools-only `certificate` profile. Direct Serde visitors return
owned upstream IC certificates/hash trees without a generic CBOR payload tree.
Byte/depth budgets, exact node tags/arities, byte payloads, pruned digest lengths,
strict fields and trailing-data refusal remain mechanical admission. Authentication,
combined evidence, commitments and leaf interpretation stay Query-owned.

The maintainer's screenshot separately authorizes the fifth generic owner:
[#110 — Host text ownership](https://github.com/dragginzgame/ic-host-tooling/issues/110).
`ic-host-text` renders escaped scalar values and Unicode-width tables with
explicit alignment/gap/separator choices. Report construction, size/truncation,
stdout, terminal detection, colour and progress remain consumer-owned. It depends
only on the already locked Unicode-width owner, with no other Host or IC edge.
The publication roster, workspace metadata, feature gate and ownership docs
include the new library. No existing API moves or compatibility reexports occur.

Five certificate cases and five text cases pass on Linux, with strict affected-
package Clippy/Rustdoc and actual Rust 1.88 all-target/all-feature checking.
Production graph exclusions, formatting and links pass. Initial fixture compiler
and lint failures are retained alongside corrected passes at
`/tmp/host-formats.rllgiU/`. Source provenance binds the clean Query inputs to
`3dd2fe811b5fbdbfebb57694c9937d2dff6c07b7`. Query's current decoder and renderer
remain until published adoption; no sibling source was edited. Package versions
and the real index/refs are unchanged; no release or publication ran.
Native macOS acceptance and consumer adoption remain independent obligations.
The corrected full Linux `make ci` gate and `make msrv-workspace` pass. The first
full attempt found missing new-module inputs in Host's disposable formatting
fixture; its roster now includes the text crate and certificate sources. All
previous lock selections/checksums are preserved; only required nodes/edges are
added. The complete failure and corrected gate logs retain their source identities.

## Fixture callers and resource reports — pending 0.18.1

The approved remaining tools fixture caller now uses the opt-in filesystem
allocator from [PR #103](https://github.com/dragginzgame/ic-host-tooling/pull/103).
Its source changes are applied locally without merging Git refs; filesystem,
process and tools allocation loops converge while their mutex/Drop policies
remain intact. [#98 — Fixture directory allocation](https://github.com/dragginzgame/ic-host-tooling/issues/98)
owns required native acceptance and subsequent published Build adoption.

The resource-report example delegates regular-file admission to the existing
filesystem reader, and its valid-fixture integer encoding to locked `leb128`.
[#106 — Resource-report example cleanup](https://github.com/dragginzgame/ic-host-tooling/issues/106)
owns this cleanup. The new edge is development-only; the response-only normal
graph excludes it. Independent artifact encoder golden/malformed fixtures,
selected limits, text/JSON output and reference policy remain unchanged.

Linux focused qualification passes: 94 filesystem units plus the fresh-process
collision case, 94 process units, 20 extraction-profile units and all three
resource-report cases. The compiled example rejects directories and a FIFO
without a writer, with failure status and no report. Changed-crate strict
Clippy/Rustdoc, actual Rust 1.88, production feature graphs, sorting, formatting
and links pass. Before-images and exact logs are under
`/tmp/host-example-cleanup.zyWqMD/`; the prior pending batch below is preserved.
The full combined Linux `make ci` and `make msrv-workspace` gates also pass;
the earlier batch's full-gate evidence remains bound to its original source.
Package versions and the real index/refs remain unchanged; no release or
publication ran. Required native macOS acceptance is separate from Linux evidence.

## Raw hashing and fixture convergence — pending 0.18.1

Released source is 0.18.0 `013a14ce22ac537e5e7d15942cb2079cc69003ca`.
The older sections below retain pre-release evidence for that delivered batch.
[#105 — Raw hashing and fixture simplification](https://github.com/dragginzgame/ic-host-tooling/issues/105)
owns the approved follow-on cleanup. Raw payload hashing uses the existing slice
digest after both allowances, preserving typed errors and refusal order.
Publication fixtures derive names/paths from independently prepared Cargo
metadata while Host retains explicit publication policy and provenance checks.
Host's exit-injection loop delegates to Shared's existing probe helper; its
fixture roster, real round trips and contradictory assertions stay local.

The selected helper extends the same Shared 0.12.7 revision, with 94 roots and
113 payloads. No sibling changes, helper options, tool-pin changes, package
version changes or release effects are required. All 13 gzip cases, strict
artifact Clippy/Rustdoc and actual Rust 1.88 checking pass on Linux. Retention
and its publication/release round trips pass under current Bash and genuine
Bash 3.2, including all seven canonical exit cases per Host fixture. Snapshot
integrity, links and scoped diff checks pass. Source before-images and raw
qualification logs remain at `/tmp/host-cleanup-next3.PZDmeV/`.
The full Linux `make ci` gate and `make msrv-workspace` pass for these sources,
including workspace strict lint/docs, feature profiles and all-target tests.
Native macOS 15 Intel/Apple Silicon acceptance remains separate at #105.
The real index, package versions and Git refs are unchanged; no release or
publication ran.

## Archive/read/release simplification — pending 0.18.0

[#96 — Archive, read and lock projection](https://github.com/dragginzgame/ic-host-tooling/issues/96)
owns the approved cleanup. Immutable archive/member slices hash directly through
existing digest/identity APIs after admission, preserving digest-before-decode,
complete archive validation and typed mismatch evidence. Following reads/hashes
open once and admit descriptor type/size together, retaining nonblocking FIFO
refusal and stream growth bounds. Native open errors now precede metadata
rejection; that public refusal-order hard cut selects 0.18.0 before 1.0.
Callers must handle open failures before interpreting type or byte-limit errors.

Saved-source Cargo.lock projection uses the selected Shared 0.12.7 rewriter.
Host retains exact manifest/dependency spelling, receipts, notes and older-commit
recovery. Same-version preflight checks original local versions through the
existing TOML reader. The supplemental canonical export adds one root/payload,
bringing the current snapshot to 93 roots and 112 payloads.

Nine archive and 18 read cases pass in the frozen qualification copy, together
with strict artifact/fs all-target Clippy/Rustdoc and actual Rust 1.88 checking.
Direct consumer checks pass for pinned tool admission, Candid extraction and the
Wasm inspector's unit/CLI cases in that same frozen copy.
Release fixtures pass under current Bash and genuine Bash 3.2 with Git/setup/full
gates substituted. The actual prepared Cargo version tool in an isolated fixture
matches both saved-source projections byte-for-byte. Initial lint evidence,
before-images, frozen inputs and focused logs remain under
`/tmp/host-cleanup3.8ZNFM7/`. Snapshot integrity, repeat preview (112 KEEP),
documentation links, manifest sorting, formatting and scoped diff checks pass;
all 19 canonical lock-rewriter fixture assertions pass too. Manifest/lock, pins,
workflow, index and HEAD remain unchanged through the task.
The complete pending notes, including #77/#94 and Shared adoption, move into the
0.18 minor-line file; published 0.17 history stays intact. Own versions remain
0.17.13. Incoming source/graph changes are independently owned and preserved.
Full repository/native macOS acceptance remains separate; no commit, release,
publication, executable download or sibling mutation ran.

## Blob custody and buffered-output extraction — pending 0.18.0

[Host #94](https://github.com/dragginzgame/ic-host-tooling/issues/94) owns the
maintainer-authorized producer additions. Durable private-directory creation
retains before/after-creation errors, parent sync and the original descriptor;
pathname admission rejects removed, symlink and substituted claims. Callers keep
trusted ancestors, namespace custody, naming, recovery and cleanup policy.
Buffered child-output completion reuses the existing bounded nonblocking engine,
retaining prefetched bytes, cached nonzero/terminated exits and cleanup scope.
It neither respawns nor introduces a default elapsed-time deadline.

Linux filesystem/process suites pass (90/92 units plus doctests), strict lint/docs
and actual Rust 1.88 all-target checks pass. The isolated Blob candidate passes
115 CLI cases, its buffered-output regression and three installed observation/
recovery cases. Exact before-images, graph overrides and raw logs remain at
`/tmp/blob-extract3.lPJtvW/`. These are local/substitute results; native macOS
and published consumer adoption remain separate. Blob #69 owns activation.
Real package versions remain 0.17.13; prospective 0.17.14 exists only in the
original isolated qualification graph. The combined pending notes now select
0.18.0 for #96's refusal-order hard cut; that does not relabel the older graph.
No release, publication, staging or Git effects ran.


## Bounded Wasm custom sections — pending 0.18.0

`wasm::encode_custom_section` encodes the existing borrowed `CustomSection` view
into one complete section. Name/payload u32 admission and the caller's encoded-
byte allowance precede fallible allocation; the existing locked `leb128` writer
owns integer encoding. A separate typed error preserves length, byte-limit and
allocation categories. Callers retain names, schemas, duplicates/order, module
assembly, aggregate budgets and validation
([#77 — Bounded Wasm custom-section encoder](https://github.com/dragginzgame/ic-host-tooling/issues/77)).

All 13 Wasm cases, 69 artifact units and seven consumer contracts pass on Linux.
Strict artifact Clippy/docs, actual Rust 1.88 all-target/all-feature checking,
13 interface-profile cases and six CLI cases pass. Exact-byte, UTF-8, duplicate/
order, LEB boundary and scalar overflow cases preserve independent parser fixtures.
The initial Clippy panic-doc diagnostic is retained; the canonical writer's
infallible preallocated Vec sink is documented in its narrow lint expectation.
Minimal dependency selection excludes the new optional edge; the lock changes
only the artifact dependency list, with all selected package versions retained.
Focused evidence, source before-images and review diff are under
`/tmp/host-custom-section.JImf0P/`. Full workspace/native gates are not run here.
Query's private framing remains until published registry adoption under its
existing [probe metadata adoption issue](https://github.com/dragginzgame/ic-query/issues/53).
No sibling source was edited. Package versions remain 0.17.13 and the additive
encoder work is included in the pending 0.18.0 batch; no staging, commit, release
or publication ran.

PR #90 owns the independent private-parent ancestor hard cut; it is not duplicated
or integrated here. PR #91's cadence wording is covered by delivered 0.17.13.
Current release CI `38549093241` has passing MSRV, Linux running and both macOS
lanes queued at inspection. The complete 0.17.11 matrix is historical evidence;
it is not relabelled as current encoder or Shared-adoption qualification.

## Descriptor locks for IC Build — delivered 0.17.11

The maintainer approved the scoped owner additions at
[#89](https://github.com/dragginzgame/ic-host-tooling/issues/89).
`durable::{lock_shared,lock_exclusive,unlock}` operate on caller-owned regular
file descriptors. Shared holders coexist; explicit unlock releases duplicated
kernel ownership without closing files or releasing independent shared holders.
Callers retain namespaces, clones, deadlines and final-owner policy. Nonregular
files refuse; no reopening, guard framework or elapsed-time default is added.

Six focused lock-module cases pass, including new shared-holder/duplicate-unlock
and nonregular regressions. Full Linux `make ci msrv-workspace` passes, preserving
incoming private-parent preparation. Its descriptor sources match the frozen
qualified copy byte-for-byte. Concurrent Shared 0.12.3 governance adoption is
preserved and is outside this task's source delta; no tool pins or Rust graph
were changed for this addition. Native macOS qualification remains separate.

IC Build's fs2-free candidate passes 161 units and Cargo/Wasm/process, handoff,
retention and repair cases, strict Clippy/docs and actual Rust 1.88. Its temporary
Host path overrides and prospective 0.17.11 metadata are confined to `/tmp`.
Actual registry adoption remains [Build #21](https://github.com/dragginzgame/ic-build/issues/21)
until publication; its normal graph still uses published dependencies. Evidence
and reviewed consumer patch are under `/tmp/ic-build-cleanup3/`. This task performs
no staging, commit, own version change, release or publication in either owner.

## Shared Tooling adoption — pending 0.18.0

Shared 0.12.7 at `9364ffe3f770a6d5d94809f0bf82c33202054bf8` is exported
from a clean isolated copy of pushed upstream main. The initial adoption keeps
92 roots and 111 payloads; #96's supplemental lock helper is described above.
Release-note layout/issue coverage guidance
changes, and linked CI-selection/Rust-toolchain guides enter the closure.
Every selected executable payload and tool pin matches the previous snapshot.
Host retains its native workflow; the optional classifier and two-toolchain setup
action are not selected. Local AGENTS.md records the exact new revision
([#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)).

Snapshot integrity, repeat preview (111 KEEP), 226 shared/160 local documentation
references and scoped diff checks pass. Before-images, source selection and logs
are retained under `/tmp/host-shared0127.hwmzxZ/`. Recorded Rust, manifest/lock,
pin and workflow bytes plus the index/HEAD stay unchanged; the concurrent worker's
extraction-guide additions are preserved and excluded from this task's delta.
Package versions remain 0.17.13; the complete pending batch selects 0.18.0 for the
read-admission hard cut.
Concurrent Rust additions and their notes remain independently owned; this
documentation adoption does not qualify them. Earlier 0.12.5 adoption evidence
remains in `/tmp/host-shared0125.bLuQyz/`, and executable-tooling evidence stays
bound to its 0.12.4 inputs in `/tmp/host-shared0124.JnTpCt/`.
No full suite or new native macOS qualification is claimed. No staging, commit,
release, publication, executable download or sibling mutation ran.

## Private parent preparation — delivered 0.17.11

The Host FS durable module now exposes owner-only missing-parent preparation,
reusing its existing directory-creation and synchronization sequence. Existing
path-based writers retain their ordinary parent modes. A focused private-mode,
idempotence and symlink regression plus all 29 durable module tests pass. The
complete Linux `make ci msrv-workspace` gate also passes on the Host checkout.
Backup adoption remains separate until a published Host version exposes the API;
no package version, published artifact or consumer graph changed.

## Portable FIFO fixtures — delivered 0.17.10

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

The latest finalized release is **0.17.13**, commit
`6f26efe58f2357e92cfdc7178cf3c9e5fbf65316`. Package versions remain unchanged.
Shared Tooling **0.12.7**, revision
`9364ffe3f770a6d5d94809f0bf82c33202054bf8`, is adopted through the canonical
exporter from a clean isolated checkout of pushed upstream main. The snapshot
contains 112 payloads, including the shared release-tag checker, fixture
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
[interface acceptance #72](https://github.com/dragginzgame/ic-host-tooling/issues/72)
and [lock isolation #70](https://github.com/dragginzgame/ic-host-tooling/issues/70).

The [historical handoff](archive-2026-10-13.md) preserves older source identities,
validation paths and consumer coordination. Its status claims describe their
original inspection, not current release state. GitHub issues remain the sole
follow-up tracker. Update this handoff in place instead of appending history.
