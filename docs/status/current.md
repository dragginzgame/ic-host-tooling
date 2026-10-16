# Current handoff

## Chunk-fed process framing — pending 0.22.0

Existing [PR #128](https://github.com/dragginzgame/ic-host-tooling/pull/128) now
includes pushed main `a0a969a67ed75d65589a5f14303b9a0335dc7d0a` through a normal
topic merge. Its existing reader API is retained; Host Process also owns incremental
newline framing for output callbacks. Scratch is reused between frames, byte limits
include LF, and a refused prefix retains at most the allowance plus one byte.
Sink/overflow failures reject subsequent feeds without implicit recovery. EOF tails
are returned to caller policy. No JSON schema or process lifetime is owned here.
The existing 0.22.0 draft includes upstream's independent breaking stream-error
addition; own metadata and the selected dependency graph remain 0.21.0 inputs.

All six framing cases pass on actual Rust 1.88, including every partition of a
UTF-8/multiple-frame/tail input, reader/chunk limit parity and original sink I/O
causes. Strict Process all-target/all-feature Clippy, actual Rust 1.88 all-target
checking, Rustdoc and formatting pass on Linux. A frozen
Build candidate removes its local line-buffering loop and passes five native
protocol/caller cases plus the real Cargo binary/example integration on actual
Rust 1.91. Its only lock changes replace the three coherent Host registry identities
with this candidate family. Other selected versions remain unchanged.

[Build #47](https://github.com/dragginzgame/ic-build/issues/47) retains normal
registry activation after reviewed publication; normal Build still uses published
APIs. Required hosted/native qualification, consumer publication and lifecycle
admission remain separate. No normal sibling checkout, main merge, release or
artifact cleanup ran. Before-images, caller deletion and logs remain under
`/tmp/ic-build-three-cuts-20261016.wnMuH7/`.

## Source and pending release

Released base: **0.20.1**, commit
`6952903377eb36f9d35c92770e334b30721f1d20`. Package versions remain 0.20.1;
local, uncommitted work is collected in the **0.21.0** draft. The authoritative
scope and consumer actions are in [the release notes](../changelog/0.21.md).

Pending work derives Git's dirty observation from retained status bytes and
admits optional-file descriptor metadata once at its reader. It also decouples
Candid service inspection from Wasm parsing: callers needing both select
`candid-service` and `ic-interface` explicitly. This feature-contract change
requires the pending minor release; package versions remain untouched. Durable
publication uses explicit writer custody through the same admission/cleanup engine.
Released 0.20.1 includes the column-clipping source from
[PR #144](https://github.com/dragginzgame/ic-host-tooling/pull/144), adapted to
`Cow<str>` escaping. Consumer adoption retains its own qualification.

Shared Tooling selects **0.15.2**, committed revision
`f5080b812489654eaa04d368a09f460aea07d4ce`, with 89 entrypoints and 110 exact
payloads. Common tool pins are unchanged. Preserve pending source, index/refs,
Cargo versions/lock and existing artifacts; sibling working checkouts remain
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
