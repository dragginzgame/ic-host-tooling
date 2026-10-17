# Current handoff

## Authoritative state

Read the workspace version in [Cargo.toml](../../Cargo.toml), released notes in
[CHANGELOG.md](../../CHANGELOG.md), and exact release identities from Git tags.
Do not copy those version facts into this handoff. The selected Shared baseline
and exact payload identities are in [AGENTS.md](../../AGENTS.md) and
[the snapshot](../../.shared-tooling.snapshot).

The [0.27 migration](../changelog/0.27.md) moves native inode identity to FS;
Process retains custody policy. The [0.26 migration](../changelog/0.26.md) documents
consuming cleanup's single attempt and consumer-owned recovery.
The [0.25 migration](../changelog/0.25.md)
preserves the preceding tool-probe, replica-policy, Candid and durable-admission cuts.
The numbered changelog draft owns the candidate version; manifest/version changes
and release execution remain user-owned. Existing sibling checkouts stay read-only.

## Ownership and evidence

The [extraction contract](../extraction.md) owns the five crate boundaries.
Host Tools retains shared Candid interpretation with a normal parser dependency;
IC Build owns extractor execution. Consumers retain tool trust, replica revisions,
byte budgets, schemas, targets,
credentials, retries and lifecycle. There are no compatibility reexports.
Use [host qualification](../hosts.md) to select focused checks; Linux evidence
never substitutes for native macOS 15 qualification on both architectures.

The [previous handoff](archive-2026-10-17.md) preserves historical source identities,
commands, failures and receipts verbatim. Its pending/released statements are
historical observations, not current status. Earlier archives remain linked from
that document. Keep new receipts bound to their exact source and inputs rather
than restating earlier results as current qualification.

GitHub issues are the sole follow-up tracker. Consumer adoption is recorded in
[Canic #458 — Adopt published IC Host Tooling](https://github.com/dragginzgame/canic/issues/458)
and [IcyDB #307 — Adopt split Host owners](https://github.com/dragginzgame/icydb/issues/307).
The current work's qualification and migration patch are reported at handoff.

Publication interruption recovery stays split between Host preparation custody
and Shared provenance/dispatch receipts. The selected source refreshes
[Shared PR #269 — Share Cargo source evidence](https://github.com/dragginzgame/shared-tooling/pull/269)
onto released Shared 0.23.0 (`0b351cd2ba2bcd8927b783a5fec89afc8013f708`).
The canonical helpers remain an open contribution rather than part of upstream
main; Host pins its clean committed revision. Strict certificate wire admission
remains local
until an upstream replacement preserves its tests; see
[DFINITY #503 — Strict CBOR decoding without byte/array coercions](https://github.com/dfinity/response-verification/issues/503).
Operational directory CR/LF admission remains in Host pending consumer-owned
migration; see [Canic #575 — Own Backup operational path admission](https://github.com/dragginzgame/canic/issues/575).

## Pending 0.26.2 Backup sync/checksum extraction

FS `durable::tree::sync_checksum_directory_at` now owns the generic existing-tree
sync/checksum visitor extracted from Backup. Files sync before hashing; directories
sync after children, including the retained root. Ordered pre-sync hooks can refuse
an operation. Explicit per-file bounds and raw SHA-256 output reuse the existing
walker and framing owner. No path is published, removed or replaced; expected
identity admission, whole-tree budgets, stable custody and recovery remain local
to consumers. Provenance binds the original Backup source and crash fixtures.

All 139 FS units pass on current Rust and actual Rust 1.88, including five sync
cases for exact framing/order, hook refusals, limits, unsafe/raw names and held-root
replacement. Strict all-target/all-feature Clippy, warning-denied docs, snapshot,
pins, formatting and local links pass. An isolated Backup production adapter
passes all seven original publication/crash cases, 29 IC artifact cases and five
public directory/artifact/recovery cases, plus strict Clippy. Evidence is retained
at `/tmp/backup-30-members-sync-n1_bvf7k/`.

The consumer rehearsal uses direct workspace paths in its own checkout and target,
without Cargo patches. Normal Backup retains its published graph and local sync
visitor until a new Host package is available. Native macOS, registry adoption and
release/publication remain separate. No normal sibling checkout was edited.

## Backup directory-checksum extraction

Artifacts now composes declared file identities through `artifact::tree`, preserving
Backup's exact path-component ordering and path/NUL/lowercase-digest/newline bytes.
FS's `read::tree::checksum` selects normal-component paths or an already-held root,
uses the existing no-follow descriptor walker, and bounds every file stream.
The helpers perform no sync, publication, journal transition or recovery. Callers
retain complete-tree budgets, stable namespace/content custody and product records.
Source and fixture hashes are recorded in `ci/extraction-sources.json`.

The isolated contribution reruns the original golden and malformed-path cases
against the extracted implementation, plus filesystem framing, unsafe-entry,
file-limit, exact raw-name and retained-root regressions. After incorporating the
released 0.26 base, all 59 Artifact and 134 FS units pass on current Rust and actual
Rust 1.88. Strict all-target/all-feature Clippy, warning-denied Rustdoc and actual
MSRV checks pass. FS reuses its available error derive while retaining exact
messages and wrapped source identities; no second hand-authored formatting owner
is introduced.

An earlier 0.25-base isolated Backup rehearsal selects these production owners through direct
workspace dependency paths in its own checkout and target directory, without
Cargo patches or changes to normal consumer manifests. All 538 Backup units, two
declaration/publication cases, public local recovery/artifact cases, 34 Core and
12 Agent managed PocketIC journeys pass. Strict all-target/all-feature Clippy,
actual Rust 1.91 checks and warning-denied Rustdoc pass. Evidence is retained at
`/tmp/backup-29-flow-tree-oh9bcfcd/`; the normal Backup graph still selects published
packages. Native macOS, integration and registry adoption remain separate.

Backup's future public helper cut also affects Canic's artifact compositor adapter;
that caller must use the new Artifacts compositor and map its result into its
owned record when adopting the Backup cut. No normal sibling checkout was edited. The updated 0.26-base consumer also passes
all 538 Core units, five public cases and strict all-target/all-feature Clippy;
registry/native adoption retains its own checks.

## Qualification receipt

Shared adoption evidence is retained at `/tmp/host-shared023.hvmoBh/`, relative to
clean Host `3175934`. Canonical source `4eabe7f` combines published Shared
`0b351cd` with the unchanged Cargo source-evidence helpers on PR #269. Export
retains 90 roots and verifies 114 payloads, adding the shared host observer;
executable pins are unchanged. Actual offline Host/Cargo tool admission,
snapshot/pins, formatting, links, Make routing/admission, release-adapter and
failed-fixture retention pass on Linux with their documented substituted effects.
The refreshed source passes 74 catalog cases, substituted CI installer checks,
saved member-context and archive-provenance fixtures, plus affected ShellCheck.
Its exact commit is independently retrievable from GitHub. No executable tools
were downloaded. Rust inputs, own versions, Cargo.lock, index and Host refs remain
preserved; native macOS and upstream PR integration remain separate.

Shared adoption preparation evidence is retained at
`/tmp/host-shared023-prep.zC9TdF/`, relative to its captured incoming dirty tree.
Host's actual Make fixture now selects `make/consumer.mk` through the declared
script closure, and canonical export retires the separate Make-fixture copier.
The reduced snapshot verifies 113 payloads; Make routing/admission, affected
ShellCheck, formatting, links and pin checks pass on Linux. Rust inputs, own
versions, lock selection, index, refs and normal sibling checkouts are preserved.
At observation, Shared main still names `a3d4f61`; the sibling's 0.23.0 changes
remain uncommitted and are excluded. Host retains its recorded reviewed baseline
until a committed published source can be adopted. Native macOS remains separate.

Bordered-table implementation evidence is retained at `/tmp/host-pr129.n0bIr3/`,
relative to its captured incoming dirty tree. Host now implements the API from
[PR #129 — Share dynamic bordered tables](https://github.com/dragginzgame/ic-host-tooling/pull/129)
with the pending scalar-width contract and array-backed plain rows. All 18 Text
cases pass on current Rust and actual Rust 1.88, including ragged/extra columns,
empty/header-only reports, preserved bordered whitespace, control escaping and
joined emoji. A String-based case matches IcyDB's existing CLI call shape;
the full consumer is not qualified. Affected all-target MSRV, strict Clippy,
Rustdoc, formatting and links pass. Existing PR native results apply to its old
head, not these working-tree inputs; native macOS remains separate. PR merge,
publication, own versions, selected dependencies, index and refs remain user-owned.

Text/Make simplification evidence is retained at
`/tmp/host027-text-release.xhC2G1/`, relative to its captured incoming dirty tree.
All 16 Text cases pass on current Rust and actual Rust 1.88; the new joined-emoji
regression first reproduced inconsistent table padding. Affected all-target MSRV,
strict Clippy/Rustdoc, formatting and document links pass. A disposable actual
Make invocation selects future migration files through a substituted checker;
Host's Make routing and execution-admission fixture also passes. Native macOS
qualification remains separate. Prior pending work, own versions, lock selection,
index and refs remain preserved.

The Cargo editor is still in open
[Build PR #56 — Own Cargo release candidate editing](https://github.com/dragginzgame/ic-build/pull/56).
[Shared #273 — Retire the Perl renderers](https://github.com/dragginzgame/shared-tooling/issues/273)
owns reviewed-publication adoption. The receipt's deferred Host adoption patch
passes syntax, ShellCheck and apply admission against the proposed source
interface. It is not active or runtime-qualified and installs no unpublished tool.

Identity/fixture simplification evidence is retained at
`/tmp/host027-simplify.jKVpD4/`, relative to clean Host `fc6a854`. All 134 FS
units and seven Process custody cases pass on current Rust and actual Rust 1.88;
the generic error regression also passes against the handwritten implementation.
Affected all-target/all-feature MSRV compilation, strict Clippy/Rustdoc,
formatting, pins, links and snapshot checks pass. The actual release-adapter
fixture passes with setup, qualification and Git effects substituted, and failed
fixture retention remains qualified. Shared's companion-copy, saved-context and
archive-provenance fixtures pass on the isolated refreshed source. The selected
commit is published on Shared PR #269's branch; independent retrieval into a
fresh Git store passes. Backup's import-only patch
passes apply admission; consumer execution and native macOS qualification remain
separate. Own versions, lock selection, index and Host refs are preserved.

Descriptor/fixture cleanup evidence is retained at `/tmp/host0261-descriptors.0zuih3/`,
relative to clean Host `27d1c91`. Hash/metadata observations share one descriptor;
Process retains size/digest-before-permission policy. Private-read error messages
and sources retain their contracts; fixtures borrow the cleanup guard's path.
All 132 FS and 130 Process cases pass on current Rust and actual Rust 1.88.
The four HTTP cases require loopback access and pass separately after sandbox
socket refusal. Affected all-target MSRV, strict Clippy/Rustdoc, formatting,
pins, snapshot and links pass. Native macOS qualification remains separate;
package versions, dependency selection, index and Git refs are preserved.

Shared refresh evidence is retained at `/tmp/host-shared022.vsM8HK/`. Canonical
source `08e536c` combines released Shared `a41dee9` with the unchanged Cargo
source-evidence helpers. All 90 roots and 112 payloads remain selected; executable
pins are unchanged. Offline common-tool checks, snapshot/pins, formatting, links,
Host hook adoption and fixture retention pass on Linux. Shared's 42 catalog
cases, substituted installer checks and real-Git context/provenance fixtures pass;
affected ShellCheck and independent exact-commit retrieval pass. Native macOS
qualification remains separate; pending Rust source, own versions and index
are preserved.

The subsequent simplification receipt is at `/tmp/host0261-cleanup.f3cBJY/`,
relative to its captured incoming Host tree at `7232b97`. Durable wrappers share
descriptor dispatch, preserving early permission refusal; progress frames append
padding directly. The 22 stream and four progress cases pass on current Rust
and actual Rust 1.88, with strict affected Clippy. Shared source `e7fcbc0` adds
saved member context independently of its retiring editor, exported canonically
as 112 payloads. Its real-Git and renderer fixtures and Host's substituted release
adapter pass on Linux. Native macOS qualification remains separate.

The stream-copy/framing pass is retained at `/tmp/host0261-issues.wTW9Hm/`,
based on clean Host `7232b97cf976a5ccccdb5897bbcbc542da1fc6e9`. Six framing cases
and 130 FS units pass on current Rust and actual Rust 1.88; framing covers every
chunk partition of the UTF-8/multiple-frame/tail fixture. A diagnostic regression
first reproduced leaked I/O text, then passed with redacted Display and the
original source retained. Affected all-target MSRV, strict Clippy/Rustdoc,
formatting, pins, links and snapshot checks pass. Package versions, selected
dependencies, index, refs and sibling checkouts are unchanged. These additive
APIs remain local; native qualification and consumer adoption remain separate.

The earlier Shared adoption receipts are at `/tmp/host-shared021.UNdFnp/`.
Canonical export retains all 89 selected roots and 111 payloads. Offline host/Cargo
tool checks, snapshot, pins, formatting and links pass on Linux. Host's actual
Make routing, retention, release and publication fixtures pass with documented
substitutes. Shared's provenance, publication-attempt and script-path fixtures,
affected ShellCheck and links pass on the refreshed source. Exact remote retrieval
of the selected commit succeeds. Pending Rust source, Host package versions,
lock selection and index are unchanged. No executable tools were downloaded;
the new ICP catalog requires explicit IC-tool setup for callers that use it.
Native qualification of the refreshed source remains separate.

The current issue-pass evidence is retained at `/tmp/host-issue-triage.Kn8e2Q/`,
relative to its captured incoming tree at Host `4086a17`. Prepared gzip streaming
and verified durable copying pass 71 Artifact units, six consumer contracts and
128 FS units on current Rust and actual Rust 1.88. Affected all-target MSRV,
strict Clippy/Rustdoc, formatting, pins, links and snapshot checks pass. Published
Host 0.25.1 compiles alongside PocketIC 16.1.0 in both the minimal Tools graph
and the five-library/Testkit 0.50 graph, selecting actual `thiserror 2.0.18`
without overrides. Its released native matrix is green; the new local copy APIs
still need delivery and native qualification. Source/fixture digests and commands
remain with the receipts; package versions, normal consumer graphs and Host's
index/refs are preserved.

Cleanup, error-derive and publication-provenance qualification is retained at
`/tmp/host026-simplify.Mkj1sY/`, compared with its captured incoming working tree
at Host commit `4086a1728e8bd1faf9a01b723115b379c17b607a`. All 126 FS library cases
pass on current Rust and actual Rust 1.88; diagnostic/source regressions also pass
against the original handwritten implementations. Affected all-target MSRV,
strict Clippy/Rustdoc, formatting, pins, snapshot and local-link checks pass.
Host release/publication adapters, fixture retention and Make routing pass with
their documented substitute effects. Shared's focused provenance fixture uses
real local Git and archives, including refusal and retained-evidence cases;
existing publication-attempt and script-path fixtures and affected ShellCheck
pass. That slice used Shared `8880883152070b2f4c870592eff8195b47ede8b2`, published in
[Shared PR #269 — Share prepared Cargo package provenance checks](https://github.com/dragginzgame/shared-tooling/pull/269).
Retrieval of that exact commit into a fresh Git store succeeds. These results
remain bound to that source. Native macOS, consumer
adoption and real registry publication are not qualified. Package versions,
selected dependency versions and Host's index/refs remain preserved.

Private invocation-file qualification is retained at
`/tmp/host-scratch-pass.dchmGQ/`. All 124 FS library cases pass on current Rust
and actual Rust 1.88, including five scratch cases for complete private files,
collisions, invalid/open failures, partial writes and failed cleanup. Affected
all-target MSRV checks, strict current Clippy/Rustdoc, formatting, pins, links
and snapshot checks pass. Native macOS and Canic adoption remain separate;
normal sibling checkouts, package versions and lock selection are unchanged.

Hashing-writer inspection and HTTP fixture receipts are retained at
`/tmp/host-issue-pass.EqLEcl/`. All 68 Artifact unit and six consumer-contract
cases pass on current Rust; seven hashing cases and all four HTTP fixtures pass
on actual Rust 1.88, with loopback access for HTTP. Affected all-target MSRV
checks, strict current Clippy/Rustdoc, formatting, links and snapshot checks pass.
These local results do not qualify native macOS or consumer registry adoption.
Package versions, dependency selection, index and Git refs remain unchanged.

The preceding simplification slice's Linux receipts and captured sources remain at
`/tmp/host025-simplify.VHKQBw/`. The compiled Host inputs are bound by
`host-source.sha256` (manifest digest
`52b879408333a2071b5e29fa2ce1adc8fd2030ceeb7201b6c4ec90a6ecce9e77`).
Tool cases pass on current Rust and actual Rust 1.88; affected all-target checks,
current strict Clippy/rustdoc, feature graphs and Make routing pass. The frozen
Build rehearsal passes Rust 1.91 all-target compilation and targeted artifact/Candid
cases. The captured Canic module passes its relocated policy and boundary probes
with unrelated DTO stand-ins; it does not qualify the complete Canic graph.
That slice's sibling migration patches passed apply checks; normal checkouts and
selected registry graphs are untouched. Testkit/IcyDB execution and native macOS
qualification are not claimed. Consumer comments remain unposted pending permission.

The Candid dependency change is qualified separately at
`/tmp/host025-candid-default.NlxHH1/`, with inputs bound in
`tools-source.sha256`. All 14 default cases pass on current Rust and actual
Rust 1.88; the 16 interface and 19 certificate cases pass on current Rust.
Affected all-target MSRV compilation, strict current Clippy/rustdoc, graph
boundaries, Make routing, sorting, formatting and local links pass. Refreshed
Canic/IcyDB patches remove the retired `candid-service` selection and apply cleanly;
consumer execution and native macOS qualification remain separate. Package versions,
lock selection, normal sibling checkouts, index and refs remain preserved.

Response/certificate error derives are qualified at
`/tmp/host025-error-derive.yyU5Uk/`, with inputs in `tools-source.sha256`.
Both new source-chain/redaction regressions pass against the original handwritten
implementations and the derives. All 23 Tools cases pass on current Rust and
actual Rust 1.88; strict Clippy/rustdoc, graph checks, formatting, sorting and local
links pass. Candid source and dependency selection are unchanged by this slice;
shared parsing extraction waits on the upstream direction. Native macOS remains
separate. The only new lock edge selects the already-locked thiserror release.

Fixture custody and durable-admission receipts are retained at
`/tmp/host025-custody-cut.kCAiqk/`, bound by `host-source.sha256`.
All 119 FS cases pass on current Rust and actual Rust 1.88. Current Process
validation passes 119 cases in the sandbox; the four loopback fixtures initially
refused sandbox socket creation and pass separately outside it. Actual Rust 1.88
passes the 71 tool and three barrier cases, with all-target affected compilation.
Strict current Clippy/rustdoc, formatting, sorting and links pass. Frozen Canic
Backup compiles on Rust 1.99 and passes selected stop-recovery and manifest-crash
cases; its 1.91 test compilation refuses existing `std::assert_matches!` use.
The reduced workspace retains other registry selections and overrides only three
Host sources; it is not complete Canic graph acceptance. The exact Query confined
module passes 18 cases on Rust 1.91; Blob's exact artifact module passes seven
with its unrelated CLI error enum represented by a stand-in. Query/Blob probes
are scoped module evidence, not full consumer acceptance. Canic CLI's ignored
live-environment control is not run. All three migration patches pass apply checks;
normal sibling checkouts, Candid source, own versions, lock, index and refs remain
preserved. No consumer comment was posted or native macOS acceptance claimed.
