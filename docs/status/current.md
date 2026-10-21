# Current handoff

## Integrated workspace

This draft is refreshed against release commit `c342bb28` (workspace 0.63.0). Host, Build and
Testkit were consolidated through [PR #212](https://github.com/dragginzgame/ic-tooling/pull/212),
retaining their crate boundaries and histories. Nine packages are public and two
are private testing packages. Rust edition 2024 and MSRV 1.91 remain approved.
See the [consolidation design](../design/tooling-consolidation.md) and
[consumer migration notes](../changelog/0.63.md).

The preceding crash-barrier, setup-ordering, release-MSRV, failure-evidence and
documentation-gate repairs are included in 0.62.1. The release commit's four
workflow runs completed successfully; this is historical evidence for that
commit, not qualification of subsequent working-tree changes.

## Upstream adoption

The local snapshot selects committed Shared Tooling 0.26.2 at
`1959ef75d0b86ed7725d41ae5056303d35eb633d` through its canonical exporter.
Producer and exact-source consumer native acceptance remain required before
integration. Pending Shared fixes are excluded. Build selects only `wasm-opt`;
common native/release setup, pins and product APIs are unchanged.
Tracking: [PR #226](https://github.com/dragginzgame/ic-tooling/pull/226).
The earlier qualified 0.25.9 adoption from
[PR #211](https://github.com/dragginzgame/ic-tooling/pull/211) is already present
in main, so this draft targets current main directly.

Registry inspection found Platform/macros 0.5.1, Wire/derive 0.5.4 and Metrics
0.15.0 current. The incoming Cargo.lock already selected those versions, including
the maintainer's Platform/Candid patch update. It is preserved byte-for-byte;
the starting copy is `/tmp/ic-tooling-adoption/incoming-Cargo.lock`.
No broader registry refresh, executable installation or release was performed.
The subsequent authorized Platform update selects Platform/macros 0.6.0 only;
other incoming lockfile selections are preserved. Registry and Testkit explicitly
enable the newly optional principal feature. Update evidence and the pre-update
lockfile are retained under `/tmp/ic-tooling-platform-06`.
The Testkit API removal shipped in 0.63.0. This compatible tooling-only adoption
collects pending notes under 0.63.1; the workspace version remains 0.63.0.

## Bounded consolidation review

The review traced publication selection, tool setup/version inputs, native and
Wasm qualification, and representative Build/Testkit adapters into Host owners.
Findings are tracked in [#220](https://github.com/dragginzgame/ic-tooling/issues/220),
[#221](https://github.com/dragginzgame/ic-tooling/issues/221) and
[#222](https://github.com/dragginzgame/ic-tooling/issues/222). Their local cleanup
now shares `ci/public-packages.txt`, narrows runtime setup to PocketIC and reads
development Rust/Node pins from their existing files. Archive tests, registry dry-runs, feature
profiles and real PocketIC recovery checks retain distinct proof obligations.

## Validation and preservation

Focused actual Rust 1.91 native checks passed for Host Tools and Testkit with all
features. Testkit native/Wasm profile and probe MSRV checks also passed.
Release routing and the release-runner recovery fixtures passed with substituted
effects. Snapshot integrity, dependency-policy, shell lint and doc links pass.
Adoption logs and
snapshot preview are retained under `/tmp/ic-tooling-adoption`.
Native macOS qualification for this local adoption remains pending. No full local
suite, commit, push, release or publication is part of the current request.

Cleanup evidence is under `/tmp/ic-tooling-cleanup`: the private package driver
compiles, and publication fixtures reject missing, extra, private and duplicate
package selections before effects. Workflow/shell lint passes. Hosted setup
qualification for these workflow changes remains pending.

Testkit-owned Candid helpers, error/context types and explicit Platform codec
feature are removed locally under [#223](https://github.com/dragginzgame/ic-tooling/issues/223).
Calls use optional Wire helpers or caller-encoded raw bytes. Native PocketIC still
requires Candid transitively; the Wasm profile remains Candid-free. This is a
breaking change with [0.63 migration notes](../changelog/0.63.md), not a release.
Focused removal evidence is retained under `/tmp/ic-tooling-candid`.
Linux checks pass for default/all-feature transport handling, Wire calls, live
PocketIC query/update/rejection, synthetic transport/teardown, strict Clippy and
actual Rust 1.91 native/Wasm compilation. Native macOS remains unqualified for
the removal. Incoming Cargo.lock selections remain unchanged.

Testkit's benchmark arithmetic now has its own `BenchmarkAverageError` and no
Metrics dependency, following the maintainer's Canic capability-layer boundary.
The 0.63 migration notes cover the error-type change. Focused benchmark tests,
Clippy and native/Wasm graph checks pass; evidence is under
`/tmp/ic-tooling-no-metrics`. Native macOS qualification remains pending.

Older repair evidence remains under `/tmp/ic-tooling-audit`. Migration backups,
full checkout archives and retirement evidence remain under
`/tmp/ic-tooling-migration`, including `retirement/`. Source imports preserve
Host `c3f6f605`, Build `c84c64e4` and Testkit `29399a6e` ancestry. Git history does
not establish migration or closure of old GitHub issues/PRs. Remote deletion and
consumer release/publication remain separate maintainer decisions.
