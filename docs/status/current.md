# Current handoff

## Integrated workspace

Local HEAD is `a706ca89`, the committed 0.63.2 preparation batch; workspace
metadata remains 0.63.1 and root/detail notes target 0.63.2 Unreleased.
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
Source commits do not establish registry publication or consumer adoption.

## Current local work

[#117](https://github.com/dragginzgame/ic-tooling/issues/117): the HTTP reader now
also accepts caller-owned storage so an outer timeout preserves accepted bytes.
Returned errors move retained evidence into the existing error; cancellation
leaves the prefix for the caller to retain or discard. Status, requests, timeout
and redaction remain caller-owned. Query and Blob adoption have their own issues:
[Query #93](https://github.com/dragginzgame/ic-query/issues/93) and
[Blob #96](https://github.com/dragginzgame/ic-blob-storage/issues/96).
Blob's streaming provider download must remain a verifier/file stream.

[#227](https://github.com/dragginzgame/ic-tooling/issues/227): local opt-in
`ic-testkit-native/stable-memory` adapts preserved Testkit PR #97 source
`3cd17999a53689a035dc86b6f01155eca78c7140` through Platform. The already-selected
registry Platform 0.6.2 supplies Memory and VectorMemory; the manifest floor now
requires that supplying version. Default native and all Wasm graphs remain
independent of the substrate. Clones share backing/counters, resets preserve
bytes, and overflow is sticky and must refuse exact reports. Metrics stays out
of Tooling; Metrics/Persistence own consumer adoption.

Focused Linux HTTP and memory tests, strict Clippy, Rustdoc, actual Rust 1.91
native/Wasm checks, dependency graphs, pins and documentation links pass.
Native macOS qualification remains outstanding for these local changes.
The working tree changes are uncommitted; no version bump or publication ran.

## Existing contributions and qualification

Shared snapshot remains 0.25.9 at `85aded191d85ac4a8c0e5deab207f6be18895862`.
[PR #226](https://github.com/dragginzgame/ic-tooling/pull/226) owns optimizer-only
setup and qualified Shared 0.26.2 adoption. [PR #229](https://github.com/dragginzgame/ic-tooling/pull/229)
owns the ICP decoder. Both include their own pending notes; avoid duplicating
implementations or the native setup regression when integrating. Both were open
with native checks still running at inspection; no merge was performed.

For 0.63.1 `62ab46bf`, [main CI](https://github.com/dragginzgame/ic-tooling/actions/runs/39573097382)
passed Linux and native Apple Silicon; native Intel was still running at the
last review. Optimizer, browser and Testkit runtime matrices passed. These do
not qualify newer local changes. The earlier missing-Wasm-target failure was
repaired in 0.63.1; its regression is included in local HEAD.

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
