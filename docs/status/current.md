# Current handoff

## Integrated workspace

This contribution is refreshed against main `62ab46bf` (0.63.1). Host, Build and
Testkit were consolidated through [PR #212](https://github.com/dragginzgame/ic-tooling/pull/212),
retaining crate boundaries and imported histories. Nine packages are public and
two are private. Rust edition 2024 and MSRV 1.91 remain approved. See the
[consolidation design](../design/tooling-consolidation.md) and
[0.63 migration notes](../changelog/0.63.md).

Delivered 0.63 includes Platform/macros 0.6.0, explicit principal features,
removal of Testkit-owned Candid APIs and its Metrics dependency, and optional
Build/Testkit CLI features. Binaries select `--features cli`. Native PocketIC
retains transitive Candid; the Testkit Wasm graph remains Candid-free. The shared
publication list and narrower runtime CI are also delivered. Source delivery
does not establish registry publication or downstream adoption.

## Shared tooling adoption

This draft selects committed Shared 0.26.2 at
`1959ef75d0b86ed7725d41ae5056303d35eb633d` through its canonical exporter.
Its [complete native producer matrix](https://github.com/dragginzgame/shared-tooling/actions/runs/39568135391)
passed on Linux, Intel macOS and Apple Silicon; Shared #294 is closed.
Build selects only `wasm-opt`; native/release host/Cargo/editor/server scope,
product APIs and tool pins remain unchanged. Main's earlier 0.25.9 adoption from
closed [PR #211](https://github.com/dragginzgame/ic-tooling/pull/211) is preserved.

[PR #226](https://github.com/dragginzgame/ic-tooling/pull/226) owns this adoption.
Only unpublished contribution notes move to compatible 0.63.2; published 0.63.1
notes and upstream package/lock versions are preserved. Normal merges retain the
contribution history. Pending work in the maintainer checkout is not copied.

## Validation and remaining acceptance

Main 0.63.1 already explicitly adds the development Wasm target after compiler
setup. This contribution retains the same actual-workflow regression prepared
under [#222](https://github.com/dragginzgame/ic-tooling/issues/222); do not add a
second copy when reconciling the owner's pending work.
The preceding PR head passed Linux native CI and all three optimizer, browser
and PocketIC concurrency jobs. Those results do not qualify this refreshed head.
Current-head consumer CI and maintainer integration remain required under
[#224](https://github.com/dragginzgame/ic-tooling/issues/224).

Focused Make/workflow routing, setup failure propagation, snapshot admission,
shell/workflow lint and changed documentation checks qualify this contribution.
They do not replace configured native CI, installed-product checks or release
qualification. No full local suite, release or deployment is part of this work.

## Preserved evidence

Earlier focused implementation evidence remains under `/tmp/ic-tooling-adoption`,
`/tmp/ic-tooling-cleanup`, `/tmp/ic-tooling-candid`, `/tmp/ic-tooling-platform-06`,
`/tmp/ic-tooling-no-metrics`, `/tmp/ic-tooling-cli` and `/tmp/ic-tooling-222`.
The original missing-target run retains its own source and failure identity;
no earlier run is relabelled as passed.

Migration backups, complete checkout archives and retirement evidence remain
under `/tmp/ic-tooling-migration`, including `retirement/`. Source imports preserve
Host `c3f6f605`, Build `c84c64e4` and Testkit `29399a6e` ancestry. Git history does
not establish migration or closure of old GitHub issues/PRs. Remote deletion,
consumer releases/publication and sibling edits remain separate decisions.
