# Current handoff

## Build directory custody — pending 0.22.2

An isolated contribution from pushed main `798fd42` extracts Build’s generic
directory guard and final-entry removal into `ic-host-fs::owned_directory` under
[#159](https://github.com/dragginzgame/ic-host-tooling/issues/159). Four focused
Linux cases pass on actual Rust 1.88 with strict affected all-target Clippy.
Caller-controlled disarming, cleanup retry and final-symlink behavior retain their
existing semantics. Construction grants no ownership or namespace confinement.
Build retains cache lifecycle, locks and error composition; its normal source
keeps the published implementation until reviewed Host publication. Native macOS
qualification and release remain separate. Versions and selected graph stay fixed.

## Source and pending release

Latest tagged release: **0.22.1**, commit
`b99beeae7c3115ee5323ea3b48fccad2fe325596`. Package versions are 0.22.1.
The **0.22.2** draft adds existing lock-sidecar admission, one-shot locking
and caller-verified directory commit primitives. Backup adoption requires a
published Host package and separate consumer qualification. Release scope and
consumer actions are in [the release notes](../changelog/0.22.md).

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

Shared Tooling selects **0.15.7**, committed revision
`05548771d95c4f27f23f29e28a6db8f4c0cac44d`, with 88 entrypoints and 108 exact
payloads. Fixture exit probes now check physical scratch roots and require fresh
child receipts on repeated calls, preserving earlier failure evidence.
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
