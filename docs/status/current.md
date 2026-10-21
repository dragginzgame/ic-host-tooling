# Current handoff

## Integrated workspace

IC Tooling is at release commit `66e70087` (workspace 0.62.1). Host, Build and
Testkit were consolidated through [PR #212](https://github.com/dragginzgame/ic-tooling/pull/212),
retaining their crate boundaries and histories. Nine packages are public and two
are private testing packages. Rust edition 2024 and MSRV 1.91 remain approved.
See the [consolidation design](../design/tooling-consolidation.md) and
[consumer migration notes](../changelog/0.62.md).

The preceding crash-barrier, setup-ordering, release-MSRV, failure-evidence and
documentation-gate repairs are included in 0.62.1. The release commit's four
workflow runs completed successfully; this is historical evidence for that
commit, not qualification of subsequent working-tree changes.

## Upstream adoption

The local snapshot now adopts qualified Shared Tooling 0.25.9 at
`85aded191d85ac4a8c0e5deab207f6be18895862` through its canonical exporter.
The local overlay records the same revision. This supplies committed-release
retry protection without changing tool pins or the workspace release version.
It overlaps [PR #211](https://github.com/dragginzgame/ic-tooling/pull/211);
reconcile that PR when delivering the local adoption, rather than applying both.

Shared 0.26.0 was inspected but deferred: its native qualification failed and
[Shared #294](https://github.com/dragginzgame/shared-tooling/issues/294) owns the
macOS multi-tool installer defect. Do not adopt dirty sibling fixes as a snapshot.

Registry inspection found Platform/macros 0.5.1, Wire/derive 0.5.4 and Metrics
0.15.0 current. The incoming Cargo.lock already selected those versions, including
the maintainer's Platform/Candid patch update. It is preserved byte-for-byte;
the starting copy is `/tmp/ic-tooling-adoption/incoming-Cargo.lock`.
No broader registry refresh, executable installation or release was performed.
The subsequent authorized Platform update selects Platform/macros 0.6.0 only;
other incoming lockfile selections are preserved. Registry and Testkit explicitly
enable the newly optional principal feature. Update evidence and the pre-update
lockfile are retained under `/tmp/ic-tooling-platform-06`.
Pending local notes are under 0.63.0 because of the authorized Testkit API removal;
the workspace remains 0.62.1.

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
