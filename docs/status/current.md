# Current handoff

## Controlled group capture refresh — pending 0.20.2

PR #99 normally incorporates released Host 0.20.1 at
`6952903377eb36f9d35c92770e334b30721f1d20`. Its existing controlled capture
implementation is preserved, sharing one process engine for caller cancellation,
explicit cleanup and a shared optional absolute deadline. No finite default,
signal handler or consumer admission policy is added. Own package metadata and
external selections match 0.20.1; pending notes select compatible 0.20.2.

All 68 selected tool/resolution/internal-engine cases pass on actual Rust 1.88,
along with strict affected all-target Clippy, actual MSRV checks, Rustdoc,
formatting, links and snapshot integrity. A frozen Build 0.7 consumer deletes
its local composition and passes all 65 Wasm-cache cases, strict all-target
Clippy and actual Rust 1.91 checking against this coherent Host candidate family.
Only the three Host registry identities are replaced in that temporary graph;
all other locked selections remain unchanged. The older native receipts remain
bound to their previous head; this exact source needs new hosted/native checks.
Build keeps its published graph until the API is released.
Normal sibling checkouts remain untouched. Evidence is retained at
`/tmp/ic-build-cuts071-20261015/`.

## Source and pending release

Released base: **0.20.0**, commit
`73b70e562ebd221baed93e311e1d8aef902452ce`. Package versions remain 0.20.0;
local, uncommitted work is collected in the **0.20.1** draft. The authoritative
scope and consumer actions are in [the release notes](../changelog/0.20.md).

Pending [PR #144](https://github.com/dragginzgame/ic-host-tooling/pull/144) source
adds pure terminal-column clipping, adapted from `0b95f12` to current `Cow<str>`
escaping. Zero columns return empty text; clipping escapes controls first and
reserves an ellipsis using conservative scalar widths. Relay retains dimensions,
queue presentation and redraw policy; its temporary Git selection can change
after publication. No sibling source, dependency or release metadata changes.

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

GitHub issues are the sole follow-up tracker. Update this handoff in place;
retain historical source and qualification evidence in its archive.
