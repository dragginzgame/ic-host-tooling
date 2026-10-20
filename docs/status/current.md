# Current handoff

## Authoritative state

Host 0.31.1 is released at `950669df9d2c91497e99ae1e3f1c5a77f8f80ef8`;
package versions remain 0.31.1. [Cargo.toml](../../Cargo.toml),
[CHANGELOG.md](../../CHANGELOG.md), Git tags and [AGENTS.md](../../AGENTS.md)
own release state and local rules. Pending [0.31.2 notes](../changelog/0.31.md#0312)
cover FS profile qualification and publication-guidance corrections. Real index,
HEAD, versions, release/push and publication remain user-owned.

## Boundary and released qualification

The [five-crate contract](../extraction.md) remains: Artifacts owns byte/Wasm
mechanics; FS owns filesystem mechanics; Process owns bounded execution; Text owns
generic decoding/rendering; Tools owns IC interpretation. Platform owns external
IC dependencies and Build owns native extraction/release rendering. Consumers
retain schemas, budgets, targets and effect recovery. HTTP extraction remains
an ownership decision in [#117](https://github.com/dragginzgame/ic-host-tooling/issues/117);
no sixth crate or competing body reader is introduced.

Platform 0.4/defaults-off remains selected. Its runtime/signature hard cuts do not
enter Host's selected capabilities. The 0.31.1 [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/39395970099)
passes Linux, both macOS hosts and MSRV. This qualifies that released source,
not the pending Make changes. Earlier adoption evidence remains committed at
`950669d:docs/status/current.md`, including `/tmp/host-platform-040.1OEwh5/`.

## Pending local 0.31.2 repairs

Review of the old producer PRs found their implementation bodies delivered, but
PR #210's independent FS feature dispatch had not carried forward. Make CI now
runs core, capability-only and temporary-file-only cases using the final feature
names. Actual Rust 1.88 passes those three profiles; the Host Make-routing fixture
passes. Native execution of the updated CI dispatch remains separate.

The 0.28.1 guide now has an explicit publication correction: the advertised batch
was preparation, while 0.29 supplied capability/scoped-file/full-validation APIs
and the final serde_cbor decoder. Existing archives and historical proposal text
remain intact. A fresh registry-only 0.31.1 consumer on Rust 1.88 passes private
permissions, binary child access, normal/unwind cleanup, explicit-close failure,
core encoding/full validation and response decoding without a sibling override.

Keep [#209](https://github.com/dragginzgame/ic-host-tooling/issues/209) and
[PR #210](https://github.com/dragginzgame/ic-host-tooling/pull/210) open until these
repairs reach main. Superseded producer PRs #203–#207 were closed without claiming
merges. Producer issues #76/#80/#200/#201 are complete; remaining consumer
acceptance belongs to [Auth #78](https://github.com/dragginzgame/ic-auth/issues/78)
and [Query #87](https://github.com/dragginzgame/ic-query/issues/87).
Testkit's encoder adoption closes #208: default source `96c01242` resolves Host
Artifacts 0.31.1 and [native CI](https://github.com/dragginzgame/ic-testkit/actions/runs/39401962713)
passes fixture/PocketIC query/update execution and MSRV on all three hosts.

## Isolated cross-repository work

[Shared PR #290](https://github.com/dragginzgame/shared-tooling/pull/290) gives the
scratch reuse consumer the declared compiler and supplies that input to its
portable fixture. The previously failing native reuse step passes all three
hosts; the companion fixture fix passes locally. Complete updated hosted checks
pass on Linux and Apple Silicon; Intel macOS and owner review remain before integration.
Host's snapshot remains 0.25.3 at
`c55c1065c0061455dab7f0e2be8ddd04fdeb9910`; this PR is not silently adopted here.

[Auth PR #80](https://github.com/dragginzgame/ic-auth/pull/80) removes local file
commands/batching in favor of pinned, offline-admitted Host CLI 0.31.1 with `cli`.
Contract generation/lock admission and Auth recovery policy stay there. Eleven
Rust 1.91 tooling cases, strict Clippy and the real local/mocked release fixture
pass on Auth's merged 0.11.0 base. A later hosted Linux failure exposed the Testkit
caller fixture's old tool roster. PR head `0384ddb` fixes that fixture and seals
the file-tool pin/delegation inputs across CI, with focused ordering/drift checks
passing locally. Updated hosted native checks and owner review remain with its PR.
Normal sibling checkouts were read-only; all contribution edits used isolated clones.

## Evidence and remaining limits

Receipt: `/tmp/host-followups.oBBZP6/` retains source comparisons, original failed
Shared artifacts, local tests, registry probes, PRs and closure records.
`/tmp/host-audit-20261020/` holds the follow-up failure/consumer logs and closures. Focused
Host formatting, snapshot, pins, links and Make checks pass. No full local
workspace/release gate or release/publication was run. The Host index, HEAD,
versions and other artifacts are preserved. An unrelated concurrent lock update
to rustls-webpki 0.103.16 was left intact; this tooling work does not qualify it.
