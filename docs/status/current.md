# Current handoff

## Delivered source and ownership

The committed baseline is release source
`d71f4690c7e8e3ffe99f72467bc35cbadce33042`, workspace 0.66.0. A tag or version
does not establish registry publication, consumer adoption or complete native
acceptance. Read the selected Cargo.lock before reusing qualification; the normal
checkout has an independent lock edit that this documentation change does not carry.

Host, Build and Testkit share one release workspace while retaining the
[crate ownership boundaries](../extraction.md). Host owns host primitives, Build
owns generic Cargo/build artifacts, and Testkit owns simulator admission and
lifecycle. Consumers retain credentials, targets, retry/retention policy and
application qualification. Public package identities survived consolidation;
the [migration design](../design/tooling-consolidation.md) retains import history.

The reviewed Shared snapshot remains 0.26.6 at `4cb9bc9`, recorded in
[the manifest](../../.shared-tooling.snapshot). Do not patch its payloads or infer
adoption from a newer sibling checkout. Rust edition 2024 and MSRV 1.91 remain
required; [host qualification](../hosts.md) owns focused checks and native hosts.

## Current public contracts and open acceptance

- [0.66 ownership migration](../changelog/0.66.md): PocketIC client/server fields
  are private. Borrow through accessors, use `into_parts()` to transfer teardown,
  and use `from_borrowed_server()` for externally managed servers. Sync owner
  drop preserves client-before-server order; async client deletion remains
  explicit. [#237](https://github.com/dragginzgame/ic-tooling/issues/237) owns
  remaining acceptance; do not restore field access or compatibility wrappers.
- [0.65 Wire migration](../changelog/0.65.md): native calls use explicit
  `WireCallSpec` and Serde request/response types; Wire owns framing/traversal.
  The pre-copy frame bound and caller-owned budgets remain.
  [#236](https://github.com/dragginzgame/ic-tooling/issues/236) owns acceptance.
- [Cycle-accepting fixtures (#235)](https://github.com/dragginzgame/ic-tooling/issues/235)
  bound acceptance and retain exact replies/counters across held release.
  A setup trap rolls back acceptance/slot consumption. Fixture support does not
  qualify a downstream funding journal or live provider.
- [0.64 migration](../changelog/0.64.md) removed duplicate dependency facades
  and policy types. Consumers import Platform types directly; native PocketIC's
  Candid graph is distinct from Testkit's Candid-free Wasm graph.
  [#233](https://github.com/dragginzgame/ic-tooling/issues/233) retains consumer
  qualification; do not infer it from Tooling's producer checks.

## Next independent work

[Wire artifact composition (#225)](https://github.com/dragginzgame/ic-tooling/issues/225)
depends on [Canic's producer contract (#613)](https://github.com/dragginzgame/canic/issues/613).
Canic owns compiled endpoint metadata and release identities. Compose Build's
existing exact-input/cache transactions only against those concrete artifacts;
do not invent a parallel schema catalog or duplicate the active Canic work.

For other acceptance, inspect the [owning issues](https://github.com/dragginzgame/ic-tooling/issues)
and matching current workflows. This summary changes no issue disposition and
does not carry forward superseded claims about the current checkout or open PRs.

## Qualification and retained evidence

Ownership-transfer checks passed thirteen focused runtime cases and seven
doctests on development Rust 1.99.1 and actual Rust 1.91, with strict native
Clippy and caller compilation. Original attempts and both observed lock inputs
remain under `/tmp/ic-tooling-237/`. Fresh native macOS acceptance was not
established by that local record.

Wire adoption evidence is `/tmp/ic-tooling-wire064/`; cycle-acceptance and
rollback evidence are `/tmp/ic-tooling-235/` and `/tmp/ic-tooling-235-rollback/`.
The original identities and limitations remain authoritative. The earlier
[0.63.3 native CI](https://github.com/dragginzgame/ic-tooling/actions/runs/39637698526)
does not qualify later 0.66 source or the independent dirty lock.

Migration archives, imported proposal objects and retirement evidence remain
under `/tmp/ic-tooling-migration/`, including `retirement/`. The former Host
checkout retains original refs. The retained stale release lock is
`/tmp/ic-tooling-stale-release-lock.qHQFTn/lock`; saved plans were preserved.
Never delete those recovery inputs to make a later attempt pass.

The complete former handoff, including HTTP/native-observer evidence, historical
workflow links and earlier failed attempts, is retained at
[the baseline revision](https://github.com/dragginzgame/ic-tooling/blob/d71f4690c7e8e3ffe99f72467bc35cbadce33042/docs/status/current.md).
Use it for a specific historical obligation rather than requiring it at startup.
No artifacts, locks, package versions or release state change in this cleanup.
