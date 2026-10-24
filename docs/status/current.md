# Current handoff

## Delivered source and ownership

The committed baseline is release source
`4bc810c84ba45baa191030eb625b8e793124581e`, workspace 0.66.2. A tag or version
does not establish registry publication, consumer adoption or complete native
acceptance. Read the selected Cargo.lock before reusing qualification; the normal
checkout has an independent lock edit that this documentation change does not carry.

Host, Build and Testkit share one release workspace while retaining the
[crate ownership boundaries](../extraction.md). Host owns host primitives, Build
owns generic Cargo/build artifacts, and Testkit owns simulator admission and
lifecycle. Consumers retain credentials, targets, retry/retention policy and
application qualification. Public package identities survived consolidation;
the [migration design](../design/tooling-consolidation.md) retains import history.

The reviewed Shared snapshot is 0.26.9 at `3657c04`, recorded in
[the manifest](../../.shared-tooling.snapshot). Do not patch its payloads or infer
adoption from a newer sibling checkout. Rust edition 2024 and MSRV 1.91 remain
required; [host qualification](../hosts.md) owns focused checks and native hosts.

Pending [0.66.3 notes](../changelog/0.66.md#0663---unreleased) raise the shared
Platform minimum to published 0.6.11, already selected by the incoming lock edit.
Shared Tooling's current upstream revision matches the verified local snapshot.
Package versions remain 0.66.2; the incoming lockfile is preserved unchanged.
Focused Linux Clippy, Rust 1.91 caller compilation, Wasm compilation and dependency
graphs pass, with eleven memory, Candid and principal cases. Formatting, doc links
and snapshot admission pass. Evidence is under `/tmp/ic-tooling-0663/`;
fresh native macOS qualification of this pending batch has not run.

[#239](https://github.com/dragginzgame/ic-tooling/issues/239) is implemented locally
in the same pending patch: nested explicit workspace targets are excluded from
Cargo snapshots and transactional guards, while declared inputs and physical
source aliases remain watched. Original regression and related-case evidence
remain under `/tmp/ic-tooling-239/`. The current incoming lock also advances
generic-array/smallvec and changes crypto-common; five boundary cases on both
compilers and strict Clippy were refreshed under `/tmp/ic-tooling-issues-20261024/`.
Delivery, native macOS and actual Canic adoption remain separate; no
consumer-specific generated-state exclusion was added.

## Current public contracts and open acceptance

- [0.66 ownership migration](../changelog/0.66.md): PocketIC client/server fields
  are private. Borrow through accessors, use `into_parts()` to transfer teardown,
  and use `from_borrowed_server()` for externally managed servers. Sync owner
  drop preserves client-before-server order; async client deletion remains
  explicit. [#237](https://github.com/dragginzgame/ic-tooling/issues/237) is closed
  with producer evidence; consumer adoption remains separate. Do not restore
  field access or compatibility wrappers.
- [0.65 Wire migration](../changelog/0.65.md): native calls use explicit
  `WireCallSpec` and Serde request/response types; Wire owns framing/traversal.
  The pre-copy frame bound and caller-owned budgets remain.
  [#236](https://github.com/dragginzgame/ic-tooling/issues/236) is closed with
  producer qualification and Auth adoption of published Testkit.
- [Cycle-accepting fixtures (#235)](https://github.com/dragginzgame/ic-tooling/issues/235)
  bound acceptance and retain exact replies/counters across held release.
  A setup trap rolls back acceptance/slot consumption. Fixture support does not
  qualify a downstream funding journal or live provider.
- [0.64 migration](../changelog/0.64.md) removed duplicate dependency facades
  and policy types. Consumers import Platform types directly; native PocketIC's
  Candid graph is distinct from Testkit's Candid-free Wasm graph.
  [#233](https://github.com/dragginzgame/ic-tooling/issues/233) is retired as a
  coordination tracker; remaining Canic qualification has its own
  [native](https://github.com/dragginzgame/canic/issues/450) and
  [codec](https://github.com/dragginzgame/canic/issues/615) owners.

## Next independent work

[Canic's Wire producer (#613)](https://github.com/dragginzgame/canic/issues/613)
already composes the published Build snapshot/transaction APIs. Canic retains
normal phase invocation, release schema identities and frontend delivery.
[#225](https://github.com/dragginzgame/ic-tooling/issues/225) is retired as duplicate
coordination; the concrete Build input-boundary defect remains with #239.

[Crypto qualification (#240)](https://github.com/dragginzgame/ic-tooling/issues/240)
tracks the native Agent graph's crypto-bigint advisory. Isolated non-Agent and
Wasm graphs exclude it; combined native workspace features can unify it through
Platform. Caller-selected ECDSA signing prevents a blanket unused-code exemption.
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50) owns the
supplying correction; no compatible lock-only update or exemption was applied.
Current audit/graph evidence is under `/tmp/ic-tooling-issues-20261024/`.

For other acceptance, inspect the [owning issues](https://github.com/dragginzgame/ic-tooling/issues)
and matching current workflows. This summary changes no issue disposition and
does not carry forward superseded claims about the current checkout or open PRs.

## Qualification and retained evidence

Ownership-transfer checks passed thirteen focused runtime cases and seven
doctests on development Rust 1.99.1 and actual Rust 1.91, with strict native
Clippy and caller compilation. Original attempts and both observed lock inputs
remain under `/tmp/ic-tooling-237/`. Fresh native macOS acceptance was not
established by that local record.

Delivered 0.66.0 subsequently passed [source-bound CI](https://github.com/dragginzgame/ic-tooling/actions/runs/39930549305)
on Linux/MSRV and, separately, native macOS 15 Intel and Apple Silicon.
All four installed cycle-reply cases pass on both Macs; #235 is closed.
Five explicit real-server ownership/baseline cases remain ignored in hosted CI;
their passing Linux evidence above is not Mac execution evidence.

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
