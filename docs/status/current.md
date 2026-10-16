# Current handoff

## Source and pending release

Latest tagged release: **0.23.1**, commit
`bcd4f9c32b4c3c8f1ea219632fc813fe846039e3`. Package versions remain 0.23.1.
The pending candidate is **0.23.2**, adding digest-verified durable artifact
copying for [#102 — Digest-verified artifact copying](https://github.com/dragginzgame/ic-host-tooling/issues/102)
through [PR #104 — Verified copying](https://github.com/dragginzgame/ic-host-tooling/pull/104).
Memory's [#92 — Adopt published Host verified copying](https://github.com/dragginzgame/ic-memory/issues/92)
retains the published-package adoption boundary. Consumer naming, byte limits,
reuse, retained-file rehash and release receipts remain local. Linux focused
durable tests, strict filesystem Clippy, Rustdoc and Rust 1.88 checks pass;
native hosted acceptance and publication remain separate.

Tagged 0.23.0 moved bounded line normalization from Tools' Candid module into
`ic_host_text::normalize::normalize_lines` without an alias. Callers update imports
and direct dependencies; the operation retains
its whitespace, UTF-8 and byte-admission behavior. Tools selects Text only for
its extraction tests/example. Package versions remain under the user's release flow.
Tagged 0.23.0 also shares private filesystem entry operations and retains admitted
HTTP framing length while reading bodies. Existing caller error/traversal policy,
deadlines and partial-byte evidence remain at their owners.
That release requires an explicit missing-target depth in the single
`canonicalize_allow_missing(path, base, depth)` API, admits durable permissions
once into a native mode and removes duplicate cleanup rendering from explicit
process diagnostics. Existing permission preflight, filesystem identity checks,
resolver traversal semantics and typed cleanup evidence remain intact.
Build and Canic retain unlimited resolution explicitly; Query retains its export
depth of 64 and selects unlimited expansion explicitly in Governance smoke.
Canic's import, direct dependency and error-downcast migration is recorded in
[#458 — Adopt published IC Host Tooling](https://github.com/dragginzgame/canic/issues/458#issuecomment-6214600612).
Its active checkout and selected registry graph remain unchanged by this session.
It adds bounded encoded Wasm code/data statistics for
[#145 — Encoded Wasm statistics](https://github.com/dragginzgame/ic-host-tooling/issues/145),
adapting [PR #146 — Code and data statistics](https://github.com/dragginzgame/ic-host-tooling/pull/146).
`wasm::code_statistics` includes tail indirect calls and excludes data framing;
operand/data bytes are not counted as instructions. It reuses the selected
parser and existing errors, without changing `inspect`, dependencies or features.
IcyDB retains report semantics, artifact comparisons, callback collection and
feature/tool admission; registry adoption remains in
[#307 — Adopt split Host owners](https://github.com/dragginzgame/icydb/issues/307).
Release scope and consumer actions are in [the release notes](../changelog/0.23.md).
It adds pure terminal-safe progress frames for
[#157 — Progress-line redraw frames](https://github.com/dragginzgame/ic-host-tooling/issues/157),
adapting [PR #158 — Pure terminal progress frames](https://github.com/dragginzgame/ic-host-tooling/pull/158).
It preserves the streaming clipper, measures escaped progress text with scalar
column widths and leaves terminal/output/lifecycle policy with Query. Published
consumer adoption remains in
[#70 — Adopt Host progress frames](https://github.com/dragginzgame/ic-query/issues/70).
It also adopts Shared 0.16.0. Its adoption and the new statistics/progress APIs
are compatible; the normalization/resolver API cuts and diagnostic format change
made that release breaking.
That release retires both combined pathname open-and-lock helpers. Durable
lock-file preparation is followed by descriptor `lock_exclusive` or the new
immediate `try_lock_exclusive`; preparation, contention and descriptor lifetime
remain explicit. Build, Testkit and IC Crates own the published adoption.
Group spawning also requires the cleanup policy explicitly through the single
`OwnedChild::spawn(command, cleanup)` entry point. The default wrapper and
`spawn_with_cleanup` name are retired; former default callers select
`CleanupPolicy::KillAndWait`, while bounded callers retain their selected policy.
There is no new execution deadline. Direct ownership, natural waiting, retained
admission, handoff and the existing group-capture cleanup behavior are preserved.
Build, Testkit and Query own the published source adoption.
Artifact stream admission also uses `hash_reader(reader, limit)?.verify(expected)`
directly; the combined `verify_reader` wrapper is retired without an alias.
Hash/limit errors still precede digest comparison, with the complete observed
identity retained on mismatch. Testkit owns its published archive-reader update.

Released 0.22.0 consolidates byte publication into `write_bytes(path, options,
bytes)`, makes Candid normalization an explicit caller operation and retires the
unused tar-member API/feature and example. Callers must preserve their previous
publication mode/permissions; Testkit selects `gzip` instead of `archive`.
The lockfile drops only `tar` and its unused `filetime` dependency.
The same release removes the unused strict labeled-hex codec and example format;
callers select JSON/plain hex and own CLI presentation handling. Host no longer
replays generic formatter/hook, Make-consumer or tool-command matrices; actual
formatting and Make adoption, prepared tools and failure/preservation checks remain
local. Snapshot selection retires only those unused fixture payloads and companions.
Terminal clipping shares the escape iterator
and stops at proven overflow while preserving the current output contract.
The release also adds existing-parent pathname publication through the held
descriptor engine and opt-in process failure-evidence rendering for issues
[#148 — Existing-parent publication](https://github.com/dragginzgame/ic-host-tooling/issues/148)
and [#149 — Explicit failure diagnostics](https://github.com/dragginzgame/ic-host-tooling/issues/149).
The existing-parent adapter uses directory-only opens to reject FIFO parents
without waiting, while retaining parent aliases and rename custody.
Descriptor streaming with and without result admission now shares one adapter;
public entrypoints and publication/cleanup contracts are unchanged.
Released 0.21.0 derives Git's dirty observation from retained status bytes,
admits optional-file descriptor metadata once, decouples Candid service inspection
from Wasm parsing and uses explicit durable writer custody. Consumers needing
both inspection APIs select `candid-service` and `ic-interface` explicitly.
Released 0.20.1 includes the column-clipping source from
[PR #144](https://github.com/dragginzgame/ic-host-tooling/pull/144), adapted to
`Cow<str>` escaping. Consumer adoption retains its own qualification.

Shared Tooling selects **0.16.0**, committed revision
`989752490b392f7f4c542c7652dcb84e75ecf363`, with 88 entrypoints and 108 exact
payloads. CI binary installs and assertion-retention probes require explicit
destinations and selections. Host has no caller needing those migrations and
already selects the retained Make target-admission helper. Fleet LOC commands now
remain in Shared's own Makefile; Host retains local LOC. Standard fixture exit
probes retain all eight cases, including cleanup refusal and scratch-root checks.
The Make/CI adoption remains applied: Shared owns common
fixture copying, per-target unsafe-mode probes and CI tool preparation with
logs and PATH publication.
Host retains its actual target roster, routing, ordered aggregate/preflight,
LOC, jobserver and formatter-failure checks, compiler selection and cache identity.
Existing optional maintenance runtime selection remains inactive. The optional
Rust panic scanner is not selected. Common tool pins and Host's MIT license
are unchanged.
Preserve pending source, index/refs,
Cargo versions and existing artifacts; sibling working checkouts remain
read-only.

Released 0.20.0 unifies pinned admission around `VersionSpec` plus a
mandatory digest, borrows Git IDs from private captures, retains typed gzip
causes in archive errors, requires explicit named-write options and returns
borrowed-or-owned terminal text. Build, Testkit, IcyDB, Canic and Query adoption
requirements are documented with those contracts in the release notes.

Compatible cleanup consolidates durable failure evidence, Git observations,
table buffering, Unix read opening, Cargo-derived publication paths, private
artifact doubles and filesystem fixture allocation. The Wasm inspector replaces
the filesystem example with optional digest admission and detail records. Host
keeps actual consumer qualification while Shared owns its generic regressions.
The latest cleanup shares crash-marker polling, centralizes inspector output
error conversion and moves historical handoff claims into the archive.
Descriptor locking now shares native dispatch, identical denied-reader fixtures
have one private owner, and release projection uses Shared's saved-source renderer.

## Qualification evidence

These are scoped Linux records for the inputs identified by each task, not a
combined release receipt. Before-images and command logs remain at their owners:

The merge of tagged 0.23.0 (`52794c85`) with incoming Backup primitives
(`2307c39f`) passes 77 durable-filesystem, seven custody and 18 child-ownership
cases on Rust 1.99 and actual Rust 1.88. Both incoming features and the 0.23 API
cuts remain, with immediate descriptor locking using its existing dispatcher.
Strict affected-package all-target/all-feature Clippy/rustdoc, actual Rust 1.88
compilation and formatting pass. Source and index before-images, original conflict
stages and logs remain at `/tmp/host-merge-repair.VcsWR8/`.
The unpublished incoming 0.22.2 notes move to the 0.23.1 draft; the tagged 0.23.0
entry and package/lock selections are preserved. No full workspace, consumer or
native macOS qualification is claimed for this combined tree. The merge commit
and subsequent delivery remain user-owned.

Incoming custody-source qualification (before this merge):
Backup retains journal/operation binding, persisted custody records, spending
and reference-release policy. Its original source passed all seven native custody
cases on Rust 1.99 and 1.88, all 118 all-feature Process tests and its doctest,
strict all-target/all-feature Process Clippy and warning-denied rustdoc. Formatting,
Cargo inheritance and documentation links pass. An isolated Backup adapter passes
ten custody/model/reference-release cases and strict all-target Core Clippy.
The first failed-spawn probe exposed a transient descriptor copy from concurrent
forks; bounded contention-only waiting preserves the original custody contract.
Native macOS acceptance and Backup adoption from a published package remain
separate. Source and consumer rehearsal remain at `/tmp/ic-host-backup-custody`
and `/tmp/backup-custody-consumer`; no sibling working checkout was changed.

The Shared 0.16.0 adoption verifies all 108 selected payloads. Repeat canonical
previews on current Bash and genuine Bash 3.2.57 show 108 retained payloads and
no changes. Host's actual Make routing, target admission, local LOC and pipe
jobserver checks pass with substituted effects on both interpreters. All eight
exit cases pass against each of its five fixture entrypoints on both interpreters,
using prefix-only probes that run no effectful fixture bodies. Affected
ShellCheck passes. Before-images, reviewed previews and command logs remain at
`/tmp/host-shared0160.kcVtqy/`. Host's GitHub description still matches its scope.
Pending Rust work, Cargo metadata/lock, common pins, license, index and refs are
preserved. No Rust build, tool installation, release or native macOS qualification
was performed for this tooling adoption.

The stream-verification cut passes nine stream/identity and six consumer-composition
cases on Rust 1.99 and actual Rust 1.88, including byte admission, original I/O
causes and complete mismatch identity. Strict Artifacts all-target/all-feature
Clippy/rustdoc, Rust 1.88 compilation and formatting pass. Before-images, logs
and the reviewed Testkit patch remain at `/tmp/host023-verify-cut.Knw8Ql/`.
[Testkit #107 — Adopt Host byte options and retire archive](https://github.com/dragginzgame/ic-testkit/issues/107#issuecomment-6215753208)
carries its authenticated-reader migration. Its source/graph and the existing
Host pending work, versions, selections, index and refs remain unchanged by this
cut. Consumer execution, full release and native macOS acceptance remain separate.

The explicit group-spawn cut passes 18 child-ownership, 68 tool/communication and
two executable-fixture cases on Rust 1.99 and actual Rust 1.88. This includes
cancellation, unwinding, bounded reaping, retained-success handoff and no-deadline
communication. The updated spawn doctest compiles on both toolchains. Strict
Process all-target/all-feature Clippy/rustdoc and actual Rust 1.88 compilation
pass. Before-images, command logs and reviewed consumer patches remain at
`/tmp/host023-spawn-cut.rCGKvg/`.
Existing adoption owners carry the additional source changes:
[Build #9 — Qualify the exposed Host boundary](https://github.com/dragginzgame/ic-build/issues/9#issuecomment-6215625069),
[Testkit #107 — Adopt Host byte options and retire archive](https://github.com/dragginzgame/ic-testkit/issues/107#issuecomment-6215625453)
and [Query #68 — Adopt Host byte options in Governance smoke](https://github.com/dragginzgame/ic-query/issues/68#issuecomment-6215625748).
Their working source and registry graphs remain untouched. No consumer run,
full workspace gate or native macOS acceptance is claimed for this source cut;
publication and adoption remain separate. Existing pending work, versions, lock
selections, shared payloads, index and refs are preserved.

The pathname-lock cut passes six descriptor-lock and 29 durable/lock cases on
Rust 1.99 and actual Rust 1.88. The cross-process contention case also passes
after replacing its remaining native lock calls with the new descriptor API.
Strict affected-package all-target/all-feature Clippy/rustdoc, actual Rust 1.88
compilation and formatting pass. An initial strict-lint run found the enlarged
contention test over its line limit; using the descriptor operations shortened
that fixture while preserving immediate contention, release and byte checks.
Before-images, command logs and reviewed consumer patches remain at
`/tmp/host023-lock-cut.Y30NeI/`.
The existing consumer owners carry the source migrations:
[Build #9 — Qualify the exposed Host boundary](https://github.com/dragginzgame/ic-build/issues/9#issuecomment-6215503542),
[Testkit #107 — Adopt Host byte options and retire archive](https://github.com/dragginzgame/ic-testkit/issues/107#issuecomment-6215504024)
and [IC Crates #6 — Adopt Host private publication](https://github.com/dragginzgame/ic-crates/issues/6#issuecomment-6215504360).
Their working files and selected graphs remain untouched. Producer source is
uncommitted; registry adoption, full release checks and native macOS acceptance
remain separate. Existing pending work, versions, lock selections, index/refs
and shared payloads are preserved.

The latest 0.23 simplification passes seven resolver, 22 streamed publication,
11 named publication, three retained publication, 29 durable/lock and three
diagnostic cases on Rust 1.99 and actual Rust 1.88. The executable-admission
fixture's nested driver passes on each toolchain as well. Invalid permission
preflight still precedes parent creation/opening and producer dispatch; all four
cleanup-error fields render once while capture bytes remain retained.
Strict affected-package all-target/all-feature Clippy/rustdoc and actual Rust
1.88 compilation pass. Before-images, command logs and reviewed consumer patches
remain at `/tmp/host-next-three.hr8MYx/`.
Build, Query and Canic resolver migrations are recorded in their existing
[#9 — Qualify the exposed Host boundary](https://github.com/dragginzgame/ic-build/issues/9#issuecomment-6215235440),
[#68 — Adopt Host byte options in Governance smoke](https://github.com/dragginzgame/ic-query/issues/68#issuecomment-6215235833)
and [#458 — Adopt published Host Tooling](https://github.com/dragginzgame/canic/issues/458#issuecomment-6215236499).
IcyDB's existing
[#307 — Adopt split Host owners](https://github.com/dragginzgame/icydb/issues/307#issuecomment-6215236176)
records the shorter diagnostic. Sibling sources and selected registry graphs
remain untouched; no consumer rehearsal, full workspace gate or native macOS
qualification was performed for this batch. Package versions, lock selections,
shared payloads and the incoming source remain preserved.

The 0.23 cleanup passes 25 filesystem read, nine HTTP, 15 Text and eight Candid
extraction cases on Rust 1.99 and actual Rust 1.88. The extraction example builds
on both toolchains. Strict affected-package all-target/all-feature Clippy,
rustdoc with denied warnings, Rust 1.88 all-target/all-feature compilation,
production feature-graph exclusions, Cargo sorting and ShellCheck pass.
The release-adapter fixture passes with setup, qualification and Git effects
substituted; this includes projection of the added workspace Text dependency.
No full workspace gate, release, publication or native macOS run was performed.
Before-images and logs remain at `/tmp/host-three-cleanup.QVuTIR/`.
The first metadata attempt exposed the missing root Text declaration; the first
locked check then refused its new development edge. Offline lock preparation adds
only that edge, preserving all selected package versions/checksums. The socket
tests initially failed at sandbox-denied loopback binds and pass with that access.
Strict lint initially identified value-passing and assertion diagnostics; these
were corrected before the passing run. Existing Wasm/progress source is retained.
Native acceptance and published consumer adoption remain separate obligations.

The Shared 0.15.8 adoption verifies all 108 payloads and repeats its canonical
preview without changes on current Bash and genuine Bash 3.2.57. All eight exit
cases, including cleanup refusal, pass against Host's five actual fixture
entrypoints on both interpreters, without running their effectful bodies.
Affected ShellCheck passes with sourced companions enabled; the first invocation
omitted that option and reported companion/EXIT-variable warnings.
The progress batch passes all 13 Text tests on Rust 1.99 and actual Rust 1.88,
strict package all-target/all-feature Clippy, rustdoc with denied warnings and
formatting. An initial Clippy doc-paragraph warning was fixed before passing.
Source before-images, reviewed Shared/PR source and focused logs remain at
`/tmp/host-shared-issues.MzPTir/`. Query's earlier isolated rehearsal belongs to
its existing issue; it was not rerun as current registry adoption here.
Native macOS qualification remains outstanding for the pending local batch;
original PR checks and released 0.22.1 checks do not qualify these adapted bytes.

The 0.22.2 statistics batch passes all 16 Wasm cases on current Rust and actual
Rust 1.88, including malformed operators/data, input bounds, both indirect-call
forms, active/passive data and opcode-looking operands/payloads. Strict affected
package all-target/all-feature Clippy/rustdoc, actual Rust 1.88 compilation and
minimal-feature compilation pass. A native Binaryen 133 control produces a Wasm
whose two indirect calls and five payload bytes match the compiled Host API.
This controlled comparison is not IcyDB artifact/report qualification.
Before-images, reviewed PR source, control inputs and logs remain at
`/tmp/host-0222-wasm.vpKL8h/`. An initial standalone control-harness compile used
the wrong dependency directory; correcting that harness input passes without
changing Host source. Versions, lock selection, pins, index and refs are retained.
Native macOS and full release qualification remain separate for this dirty tree.

The delivered 0.22.1 CI observation is
[run 38887331178](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38887331178)
at `b99beeae7c3115ee5323ea3b48fccad2fe325596` is now completed successfully:
Linux, MSRV, macOS 15 Intel and Apple Silicon all pass. Delivered producer issues
[#130 — Native RSS capture](https://github.com/dragginzgame/ic-host-tooling/issues/130),
[#131 — Retire the local dependency projector](https://github.com/dragginzgame/ic-host-tooling/issues/131),
[#134 — Shared Cargo ordering fixture](https://github.com/dragginzgame/ic-host-tooling/issues/134),
[#148 — Existing-parent publication](https://github.com/dragginzgame/ic-host-tooling/issues/148)
and [#149 — Explicit failure diagnostics](https://github.com/dragginzgame/ic-host-tooling/issues/149)
are closed against that source and gate. Consumer adoption remains separate.
That run does not qualify the new uncommitted statistics/progress source.

The Shared 0.15.7 adoption verifies all 108 selected payloads from the published
revision. All seven exit cases pass against each of Host's five actual fixture
entrypoints on current Bash and genuine Bash 3.2.57, using prefix-only probes
that run no fixture bodies, tools, release adapters or uploads. Affected
ShellCheck passes. Before-images, preview and logs remain at
`/tmp/host-shared-next.KO2HEr/`. Existing staged source, merge/index state,
Cargo metadata/lock, common tool pins, MIT license and Git refs are preserved.
No Rust build, installation, release or native macOS qualification is claimed.

The Shared 0.15.6 adoption verifies all 108 payloads from a clean checkout of the
published revision. The three new helper/action payloads match the saved rehearsal
bytes; actual Host Make routing, ordered setup/preflight, LOC, jobserver and
formatter-failure checks pass on current Bash and genuine Bash 3.2.57. The adopted
CI action passes selected-caller install/check failure, output-log and PATH
publication checks on both interpreters with substituted Make effects; no tools
were installed. Failure-retention and relative-temporary-path checks pass on both
interpreters. Actionlint and affected ShellCheck pass. Source before-images,
reviewed previews and logs remain at `/tmp/host-shared0156.Eutrxn/`.
Package source, Cargo metadata/lock, tool pins, MIT license, index and refs remain
unchanged. Full release, hosted workflow execution and native macOS qualification
remain separate from this tooling adoption.

The descriptor-adapter cleanup passes all 22 stream-publication cases on current
Rust and actual Rust 1.88, strict Filesystem Clippy/rustdoc and source formatting.
Before-images and logs remain at `/tmp/host-022-converge.HlZlTL/`.
That directory also retains the earlier Make/CI patches and rehearsal against
then-uncommitted Shared helper bytes. Current adoption evidence is recorded above;
the earlier rehearsal is not relabelled as committed or native qualification.

The existing-parent/diagnostics batch passes 74 durable and 68 Process tool cases,
strict affected-package Clippy/rustdoc and actual Rust 1.88 all-target/all-feature
checks. Five new pathname cases and three explicit diagnostic cases also pass on
Rust 1.88. A retained negative control reproduces the FIFO wait with an ordinary
parent-file open and is killed/reaped by the bounded test; directory-only opening
passes without a writer. Formatting, 179 local links and 105 snapshot payloads
pass. PRs #150/#151 supplied reviewed additive source; their native checks cover
their original branches, not this adapted dirty tree. Source before-images and
logs remain at `/tmp/host-022-issues.FQmAIZ/`. Siblings, versions, lock selection,
index and refs remain unchanged. Main delivery, combined release and native
macOS qualification remain separate; the owning issues stay open for that evidence.

The Shared 0.15.4 adoption verifies all 105 payloads, 179 local documentation
links and exporter ShellCheck. Canonical refresh passes with the pending dirty
source preserved; repeat previews on current Bash and genuine Bash 3.2.57 show
only retained inputs. Common catalogs, Rust source, Cargo metadata/lock, MIT
notices, index and refs remain unchanged by this adoption. The 0.15.5 upstream
working changes are excluded. Before-images, reviewed preview and focused logs
remain at `/tmp/host-shared0154.fjeXrY/`. No Rust build, tool installation,
release or native macOS qualification was run for this tooling/documentation
update.

The clipping/Make-suite batch passes all nine Text cases on current Rust and
actual Rust 1.88, strict package Clippy/rustdoc and 9,601,538 clipping comparisons
against the saved implementation. Escaping also matches for every Unicode scalar.
Actual Host retention, Make routing, release-command admission and formatter/hook
adoption checks pass with substituted effects; routing also passes on genuine
Bash 3.2.57 on Linux. Canonical snapshot refresh retires only the two unused
generic suites and their generic jobserver-probe companion, leaving 104 payloads
at the same Shared revision. Before-images and focused logs remain at
`/tmp/host-022-clipping.OqQLI3/`. Native macOS and full release acceptance remain
separate.

The byte/extraction/tar batch passes all 99 Filesystem cases, 20 extraction-profile
Tools cases, 60 full-feature and 34 minimal Artifacts cases, and six remaining
consumer compositions. Strict affected-package Clippy/rustdoc and Rust 1.88
all-target/all-feature checks pass, as do Process/inspector dependent checks.
Actual Rust 1.88 additionally passes 69 durable and eight extraction cases.
Formatting, feature dependency graphs, 179 local links and 107 snapshot payloads
pass. Task before-images (including incoming dirty work) and logs remain at
`/tmp/host-022-hardcut.JcLjzf/`. Consumer migrations are recorded in owning
GitHub issues linked from the release notes; no sibling source was changed or
consumer acceptance inferred. Full release and native macOS qualification remain
separate.

The 0.22 response/formatter batch passes all 12 minimal Tools cases and the saved
response example's JSON/hex smoke; all ten response cases pass on actual Rust
1.88. Strict package Clippy/rustdoc and Rust 1.88 all-target/all-feature checks
pass. Actual Make formatting/hook adoption and routing fixtures pass, as does the
unchanged publication-boundary fixture with substituted Git/upload effects.
Formatting, 179 local links and 107 snapshot payloads pass; the canonical refresh
retires only the three previewed fixture payloads at the same source revision.
Before-images, preview/refresh output and focused logs remain at
`/tmp/host-022-response-format.hQj2cb/`. Full release and native macOS acceptance
remain separate.

The earlier Git/read batch passes five Git provenance and 20 read cases on Linux, strict
affected-package Clippy/rustdoc and actual Rust 1.88. Regressions first reproduced
both dirty-state disagreements and missing private-descriptor type admission.
The unchanged certificate profile passes five cases. A standalone probe observes
three wire-type coercions accepted by selected `ic-certification 3.2.0` with
`ciborium 0.2.2` and refused by Host. Reviewed upstream main `88dd491` has identical
certificate/hash-tree decoder files; its workspace was not built. Evidence,
before-images and failed attempts remain at `/tmp/host-0202-admission.zAxStk/`.
Formatting, 176 local links and snapshot integrity pass for this batch.

The feature/admission simplification passes 69 durable cases and the Candid-only,
Wasm-only and combined Tools profiles (16, 13 and 18 cases). The Candid-only
production graph excludes Host Artifacts and Wasm parsing; its 16 cases also pass
on actual Rust 1.88. Strict affected-package Clippy/rustdoc and Rust 1.88
all-target/all-feature checks pass. The Candid-only lint/docs profile passes
separately. Formatting, 177 local links and all 110 snapshot payloads pass.
Before-images, the reproduced feature-boundary failure, repaired lint attempts
and command logs remain at `/tmp/host-021-simplify.917Cwh/`. These are focused Linux
checks, not full release or native macOS qualification.

The 0.20.1 clipping batch passes all nine Host Text cases on Rust 1.99 and actual
Rust 1.88, strict package Clippy/rustdoc, formatting, 175 local links and snapshot
verification. Evidence and before-images remain at `/tmp/host-144.Mux6V1/`.
The older PR's native CI does not qualify this adapted source. Full release and
native macOS qualification remain separate.

| Evidence directory | Qualified scope |
| --- | --- |
| `/tmp/host-0191-refactor.NcfioH/` | 67 durable, five Git provenance and six text cases; strict affected-package Clippy/rustdoc and actual Rust 1.88. |
| `/tmp/host-0191-admission.TkDo7y/` | 20 read cases, five inspector units and two CLI cases; publication substitutes under current Bash and genuine Bash 3.2; strict package checks and Rust 1.88. |
| `/tmp/host-shared0151.3AeuC4/` | Host formatting/hook, retention round trips, selected Make/jobserver and evidence-helper callers; current Bash and genuine Bash 3.2. |
| `/tmp/host-next-three.2ORfLA/` | 69 all-feature artifact, 100 process, 34 minimal artifact and 20 extraction cases; strict package checks, Rust 1.88 and actual Make/release substitutes. |
| `/tmp/host-020-contracts.R8wveR/` | Nine archive cases, bundle admission, 69 durable and 100 process cases; strict package checks and Rust 1.88. |
| `/tmp/host-020-text-fixtures.vC8VUE/` | Six text, two CLI and three limits-example cases; strict package checks, Rust 1.88 and production graph without development fixture support. |

Formatting, documentation links and snapshot checks accompany the records above.
Earlier evidence is preserved in the [archived handoff](archive-2026-10-15.md),
including links to the preceding archives. Historical status is not current
release or consumer-adoption evidence.

The polling/report cleanup passes three barrier cases, five inspector units and
two CLI cases, strict affected-package Clippy/rustdoc and actual Rust 1.88.
Formatting, 174 local links and snapshot verification pass; the archived body
matches the starting handoff byte-for-byte. Evidence and before-images remain at
`/tmp/host-020-poll-report.4kcwoA/`.

The lock/reader batch passes six descriptor-lock cases, 47 all-feature artifact
cases and 34 minimal artifact cases, strict affected-package Clippy/rustdoc and
actual Rust 1.88. Release-adapter checks pass under current Bash and genuine Bash
3.2 on Linux, covering minor/major projection, aliases, missing current member
paths, failed readers, exact saved payloads and rollback. Setup, Git effects and
full gates are substituted in those adapter fixtures. Formatting, local links,
snapshot integrity and repeat preview pass; common pins are unchanged. Evidence,
before-images and repaired failed attempts remain at
`/tmp/host-020-lock-release.JvnHiu/`.

The complete combined release gate (`make ci msrv-workspace`) has not run for
this pending tree. Native macOS 15 qualification on Intel and Apple Silicon
remains separate from Linux evidence. Follow [the host guide](../hosts.md) and
[the validation policy](../../DRAGGINZGAME.md#validation-and-evidence).

## Release projection ownership

Host's AWK dependency projector was retired in 0.20.0 for
[#131 — Retire the local dependency projector](https://github.com/dragginzgame/ic-host-tooling/issues/131).
Shared 0.15.2 contains the saved-source fix tracked by
[Shared #210](https://github.com/dragginzgame/shared-tooling/issues/210).
Host derives complete member identities from the selected Git source and explicit
workspace paths; no new receipt format or old workspace reconstruction is needed.
Host's Cargo ordering fixture already uses the canonical pair preparer from
Shared 0.15.1 ([#134](https://github.com/dragginzgame/ic-host-tooling/issues/134));
native and consumer acceptance remain distinct from source adoption.

## Publication attempt ownership

The publication review retains Host's preparation lock/admission evidence and
Shared's immutable dispatch attempt. Their windows and interruption obligations
differ; a generic callback layer would not remove those requirements. The
source-bound review is recorded on
[Shared #163 — Converge Cargo publication attempts](https://github.com/dragginzgame/shared-tooling/issues/163#issuecomment-6199544254).
The focused Host fixture preserves source-drift refusal, original Cargo/logger
failures and uncertain-dispatch locks. Publisher sources and behavior are
unchanged; no upload, retry or sibling edit was performed.

## Certificate decoding ownership

The opt-in upstream request is
[DFINITY #503 — Strict CBOR decoding without byte/array coercions](https://github.com/dfinity/response-verification/issues/503).
Host retains its strict wire types and visitors until an accepted upstream
replacement preserves the admission tests. No certificate dependency, decoder,
authentication policy or sibling source is changed by this proposal.

## Operational directory admission

The private-directory creator retains its CR/LF guard. Blob's current artifact
tests depend on it; Canic's dirty Backup adoption also delegates to that creator.
Moving operational-path policy requires consumer checks before Host removes the
guard. The reviewed prerequisites are recorded on
[Blob #69 — Host custody and archive adoption](https://github.com/dragginzgame/ic-blob-storage/issues/69#issuecomment-6198024538)
and [Canic #575 — Own Backup operational path admission](https://github.com/dragginzgame/canic/issues/575).
Testkit already checks its absolute root and rejects symlink components. Sibling
sources and artifacts remain unchanged; inspection is not consumer qualification.

GitHub issues are the sole follow-up tracker. Update this handoff in place;
retain historical source and qualification evidence in its archive.
