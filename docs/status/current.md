# Current handoff

## Integrated workspace

Host, Build and Testkit are integrated in `dragginzgame/ic-tooling` through
[PR #212](https://github.com/dragginzgame/ic-tooling/pull/212), merged as
`80cc30a4904d3b27aecdb9ed67394b3bd57235b7`.
The current checkout is on the 0.62.0 release commit `f36f9dc9`.
The maintainer reports the release is live; no publication is performed by the
current repair task. Nine published packages and two private testing crates
retain their names and boundaries. Rust edition 2024 and MSRV 1.91 are approved.
See the [consolidation design](../design/tooling-consolidation.md) and
[consumer migration notes](../changelog/0.62.md).

Shared Tooling remains the recorded 0.25.8 snapshot. Shared release recovery
changes are separate adoption work; no shared snapshot files are edited here.

## Local repair scope

Pending 0.62.1 notes cover the authorized audit repairs. The workspace version
remains 0.62.0; changes are unstaged and uncommitted.

- [Crash-barrier publication](https://github.com/dragginzgame/ic-tooling/issues/213)
  now uses the existing atomic filesystem publication owner. Marker paths become
  visible with complete bytes; publication failures still reap the child.
- [Cold-checkout prerequisites](https://github.com/dragginzgame/ic-tooling/issues/214)
  now fetch locked dependencies before the explicit server setup build. Missing
  release parsers produce an actionable setup diagnostic without installing.
- [Release MSRV coverage](https://github.com/dragginzgame/ic-tooling/issues/215)
  now shares native, Testkit Wasm and unsupported-filesystem checks with CI through
  `make msrv-qualification`.
- [Product failure evidence](https://github.com/dragginzgame/ic-tooling/issues/217)
  is explicitly archived by native and optimizer CI, excluding Cargo build caches.
- [Documentation coverage](https://github.com/dragginzgame/ic-tooling/issues/218)
  includes maintained imported guides and package/browser READMEs through Git's
  source inventory, including untracked documents and excluding ignored output.

The pre-existing Cargo.toml/Cargo.lock Metrics 0.15 and Wire 0.5.4 edits belong to
the maintainer and are preserved. Their original diff is retained at
`/tmp/ic-tooling-audit/preserved-metrics.patch`.

## Qualification and evidence

Focused Linux crash-barrier tests pass, including publication-error child cleanup.
Make substitute fixtures pass setup ordering, fetch refusal and sequential MSRV
routing under parallel Make. They verify orchestration, not actual host execution.
Release-adapter fixtures pass, including missing-parser refusal and withholding
validation receipts after MSRV failure. Focused strict process Clippy, formatting,
shell/workflow lint, documentation links and the shared snapshot check pass.
Audit and repair logs are under `/tmp/ic-tooling-audit`.
Product evidence fixtures pass archive readback, partial-attempt retention and
cache exclusions. Hosted upload and native macOS qualification remain pending.

The released commit's Apple Silicon CI exposed the marker race: run
`39464225759`, job `118771657577`; its log is retained as `arm-release.log`.
Native macOS qualification for the local repairs remains outstanding. Previous
migration qualification is historical evidence, not proof for these edits.
No full local suite, release command or publication is part of this repair.

## Preserved migration evidence

Source imports preserve Host `c3f6f605`, Build `c84c64e4` and Testkit `29399a6e`
ancestry. Original refs also survive under `refs/migration-archive/` in the
original Host repository. Backups, full checkout archives and retirement audit
evidence remain under `/tmp/ic-tooling-migration`, including `retirement/`.
Remote issue/PR inventory is separate from Git history preservation. Do not infer
that source imports migrated GitHub issues or PRs, or authorize remote deletion.
Consumer dependency selection and release/publication remain maintainer actions.
