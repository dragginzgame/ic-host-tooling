# IC Tooling consolidation inventory

Evidence captured 2026-10-20. This is migration design and source provenance,
not a replacement issue tracker. No API cleanup belongs in this migration.

## Identity and scope

The maintainer selected `dragginzgame/ic-tooling`, retaining the Host repository's
history, issues and protections. GitHub has been renamed. Existing package names
remain unchanged. Rust 1.91 is the approved common minimum; changing declarations
and qualifying that minimum remain part of workspace integration.

Host owns filesystem, process, artifact, text and IC interpretation mechanics.
Build owns build execution, input evidence and retained artifact caches. Testkit
owns simulator integration and test support. Platform, Wire, Metrics and Shared
remain external owners. Product recipes, budgets and recovery remain consumers'.

## Source baseline

| Source | Exact import revision | Published packages | Checkout state at capture |
| --- | --- | --- | --- |
| Host | `c3f6f605a47f248e18bc3ad337e431a990a7a619` | All five libraries 0.32.0 | Main; unrelated Cargo.lock +2/-2 preserved |
| Build | `c84c64e42a6bbbf06eba4490cf1cf6988caf5866` | ic-build 0.24.0 | Clean main, matches remote |
| Testkit | `29399a6ee336039af60a386ed400e53cb150945e` | ic-testkit and ic-testkit-native 0.61.0 | Clean main, matches remote |

Published versions were checked against the crates.io sparse index, including
non-yanked status. Testkit's formerly unpublished preparation commit is
`614d3bc0b72ec959d24f568c25faf9d995c517f3`, titled `0.61.0`; its manifest still
says 0.60.0. It prepares Host 0.32/Build 0.24 adoption and related documentation.
It is now included in the released 0.61.0 import. It is also retained as `evidence/testkit-prepared-061` in the integration
clone, accounted for by the separate import-refresh commit. Account for its dependency
and documentation changes during integration; do not release the old graph.

The existing Host and Build handoffs lag their released manifests. Exact commits,
manifests and registry evidence above take precedence over their pending wording.

## Preserved history and outstanding work

The isolated branch is `migration/tooling-workspace`, based on Host's exact
revision above. Build and Testkit are imported in separate merge commits beneath
`imports/ic-build/` and `imports/ic-testkit/`. Each import has the original source
commit as a parent; imported directory tree IDs match the source root tree IDs.
No original commits are rewritten. These directories are temporary staging, not
additional maintained production owners. A later configuration commit moves the
owned packages and removes redundant imported repository infrastructure.

All fetched Build and Testkit branch heads are retained under
`refs/remotes/import-build/` and `refs/remotes/import-testkit/`. Host branches are
retained by the initial clone. Saved branch/PR/index responses are in
`/tmp/ic-tooling-migration/`; the isolated checkout is its `integration/` directory.
Remote branch existence alone does not establish unique outstanding work. No old
branch is deleted or declared obsolete by this inventory.

Open work observed through GitHub:

- Host has no open PRs; [Host #117](https://github.com/dragginzgame/ic-tooling/issues/117)
  concerns HTTP ownership and is unrelated to consolidation.
- Build has no open PRs; [Build #9](https://github.com/dragginzgame/ic-build/issues/9)
  tracks consumer qualification and [Build #81](https://github.com/dragginzgame/ic-build/issues/81)
  tracks Wire frontend orchestration. Preserve both through eventual retirement.
- [Testkit #136](https://github.com/dragginzgame/ic-testkit/pull/136), head
  `cbb68d4979c38793c33cc46afb9646a5153ca9ef`, adapts benchmark discovery to Build.
  Compare its source changes with the imported main before choosing integration;
  its open status is not proof that the fix is absent or complete.
- [Testkit #97](https://github.com/dragginzgame/ic-testkit/pull/97), head
  `3cd17999a53689a035dc86b6f01155eca78c7140`, proposes optional memory IO
  instrumentation for [#95](https://github.com/dragginzgame/ic-testkit/issues/95).
  Its PR records a Platform-boundary blocker. Do not bundle this feature into
  the mechanical import or discard it when retiring the old repository.

## Destination map

| Imported content | Final owner/path |
| --- | --- |
| Host's existing five crates and inspection application | Existing paths |
| Build crates/ic-build | crates/ic-build |
| Testkit crates/ic-testkit | crates/ic-testkit |
| Testkit crates/ic-testkit-native | crates/ic-testkit-native |
| Testkit crates/ic-testkit-dev (unpublished) | crates/ic-testkit-dev |
| Testkit crates/ic-testkit-perf-probe (unpublished Wasm fixture) | crates/ic-testkit-perf-probe |
| Testkit browser session support | testing/browser, with explicit Testkit ownership |
| Package fixtures, license texts and package readmes | Beside the owning package; preserve publication contents |
| Product-specific Build/Testkit guides and historical changelogs | docs/build and docs/testkit, repairing relative links |
| Shared snapshots, root manifests, release helpers and generic CI | One root set; consolidate rather than retain three copies |

Use path-plus-version requirements for published internal dependencies. Keep
optional features at the consuming member, not enabled globally in the catalog.
Retain Testkit-native's dependency-free graph and exclude Host/Build/PocketIC
from Testkit's declared deployed Wasm profiles. Keep Build out of the published
Testkit library graph; only its development driver currently needs Build.

## Qualification and release boundary

Integration must preserve the existing gates, with commands adapted to the single
workspace: Host's independent feature tests and graph guards; Build's engine,
artifact-retention and optimizer checks; Testkit's native/Wasm profiles, real
PocketIC startup/recovery/concurrency, browser-session tests and benchmark driver.
Run formatting, strict Clippy, warning-denied Rustdoc, dependency-role and graph
checks, release/publication refusal fixtures, and actual Rust 1.91 checks.
Native Linux, macOS Intel and macOS Apple Silicon results remain separate.
Package all publishable crates and verify generated manifests, included fixtures,
licenses and internal dependency requirements. A successful workspace build alone
is insufficient evidence of publishable dependency resolution.

The recommended first coordinated version is 0.62.0; the integration workspace now selects this version, but publication has not been executed. It must exceed all
published package versions at release time and be unused for every package.
Public Host type identities change with a pre-1.0 minor requirement; document
that compatibility cut even when implementation behavior is unchanged.

Only after combined-workspace and package qualification: publish in dependency
order using resumable publication, migrate consumer requirements/locks, transfer
remaining issues and confirm no stranded work, then retire old workflows/repos.
No publication, consumer migration or retirement has occurred in this phase.

## Integration progress

Testkit 0.61.0 was published while this inventory was being prepared. Its exact
release source is now imported through a separate history-preserving refresh;
the imported tree matches the source tree. No Rust implementation changed.
Host's declared MSRV, CI compiler selection, Make routing fixture and current
support documentation now select Rust 1.91. Historical release notes and Shared's
own tool-compiler requirements retain their original values.

The original checkout still has an unrelated crc32fast lock update; it has not
been overwritten. Git bundles preserve all refs from both source checkouts,
including Build’s pre-sync stash, under /tmp/ic-tooling-migration. The prepared
PocketIC 16.1.0 bundle has been copied and admitted in the integration checkout.
