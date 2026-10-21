# Current handoff

## Integrated workspace

IC Tooling main is `c342bb281dc82ff8878c26b5f5449972e9f5341d` (0.63.0).
Host, Build and Testkit were consolidated through
[PR #212](https://github.com/dragginzgame/ic-tooling/pull/212), retaining crate
boundaries and imported histories. Nine packages are public and two are private.
Rust edition 2024 and MSRV 1.91 remain approved. See the
[consolidation design](../design/tooling-consolidation.md) and
[0.63 migration notes](../changelog/0.63.md).

The delivered source includes Platform/macros 0.6.0, explicit principal features,
removal of Testkit-owned Candid APIs and its Metrics dependency, and optional
Build/Testkit CLI features. Consumers of the binaries select `--features cli`.
Native PocketIC retains transitive Candid; Testkit's Wasm graph remains Candid-free.
The shared publication list and narrower runtime CI are also delivered.
This records source delivery, not registry publication or downstream adoption.

## Shared tooling

The snapshot is Shared 0.25.9 at `85aded191d85ac4a8c0e5deab207f6be18895862`.
[PR #211](https://github.com/dragginzgame/ic-tooling/pull/211) overlaps this
already-delivered adoption. Do not apply it a second time.
[Draft #226](https://github.com/dragginzgame/ic-tooling/pull/226) contains the
optimizer-only selection proposal. Its Shared successor requires qualification
under [Shared #294](https://github.com/dragginzgame/shared-tooling/issues/294)
before adoption; dirty sibling fixes are not a snapshot source.

## Current validation and local repair

The exact 0.63.0 [runtime matrix](https://github.com/dragginzgame/ic-tooling/actions/runs/39565694899)
passed on Linux, macOS 15 Intel and Apple Silicon. Optimizer and browser workflows
also passed. The [main CI run](https://github.com/dragginzgame/ic-tooling/actions/runs/39565694787)
failed native Linux at the Wasm compile stage because the development toolchain
lacked `wasm32-unknown-unknown`; native macOS was still running at inspection.
MSRV and lint jobs passed. This is not a fully qualified release matrix.

The local [#222 repair](https://github.com/dragginzgame/ic-tooling/issues/222)
explicitly adds the Wasm target after preparing the file-selected compiler.
Workflow lint, the explicit target command and focused Wasm builds pass locally.
The local target was already installed, so this is not fresh-runner proof.
Fresh hosted qualification is required after delivery. Notes target 0.63.1;
package versions, refs and lockfile remain unchanged by this repair.
Evidence is under `/tmp/ic-tooling-222`; the failed Linux log is
`/tmp/ic-tooling-063-linux.log`.

## Preservation and earlier evidence

Focused implementation evidence remains under `/tmp/ic-tooling-adoption`,
`/tmp/ic-tooling-cleanup`, `/tmp/ic-tooling-candid`, `/tmp/ic-tooling-platform-06`,
`/tmp/ic-tooling-no-metrics` and `/tmp/ic-tooling-cli`. Product evidence and
publication rejection fixtures passed again during issue review. Those local
checks do not substitute for the current required hosted matrix.

Migration backups, full checkout archives and retirement evidence remain under
`/tmp/ic-tooling-migration`, including `retirement/`. Source imports preserve
Host `c3f6f605`, Build `c84c64e4` and Testkit `29399a6e` ancestry. Git history does
not establish migration or closure of old GitHub issues/PRs. Remote deletion,
consumer release/publication and sibling edits remain separate decisions.
