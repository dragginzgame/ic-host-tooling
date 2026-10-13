# Current handoff

## Released source and baseline

The latest finalized release is **0.17.2**, commit
`e3afa6ddf022309e7ba97f6cefb312c48738db4e`. Package versions remain unchanged.
Shared Tooling **0.10.2**, revision
`626fc2bd3125195520fb870b92abfec11b01454d`, is now adopted locally through the
canonical exporter from a clean isolated checkout, matching published upstream
main at inspection. The 88 entrypoints and 102 payloads remain unchanged in count;
all payloads are byte-identical to 0.10.1.
See [the snapshot](../../.shared-tooling.snapshot) and
[AGENTS.md](../../AGENTS.md).

## Pending 0.17.3

Cargo CI cache keys now ignore unrelated pins, comments and record order in
literal catalogs, while preserving Cargo selections and build identity as inputs.
Complex catalogs retain byte-based keys and are never executed by inspection
([Shared #154](https://github.com/dragginzgame/shared-tooling/issues/154)).
This is compatible contributor tooling; crate source, Cargo metadata, common
catalogs, Makefile and library contracts are unchanged from released 0.17.2.

The 0.10.2 sibling-dashboard fix is outside Host's selected snapshot. The sibling
checkout's uncommitted 0.10.3 work was excluded by exporting the exact published
0.10.2 commit. Snapshot integrity, repeat preview (102 KEEP rows), preservation
hashes, documentation links and full offline Linux `make ci` pass.
Adoption evidence is retained in `/tmp/host-shared0102/`.
Prior 0.10.1 full offline Linux and genuine Bash 3.2 qualification remains in
`/tmp/host-shared0101/`; no tool pin, product source or dependency change is needed.
Native macOS 15 Intel and Apple Silicon qualification remains separate from
local Linux evidence. No commit, push, release or publication ran in this task.

## Evidence and coordination

Released 0.17.2 cleanup passed focused process, extraction and IC-limits tests
and full offline Linux `make ci`. Fixture retention probes and documentation
checks passed; evidence remains in `/tmp/host-0172-cleanup/` and
`/tmp/host-0172-example/`. Existing qualification owners are
[fixture cleanup #72](https://github.com/dragginzgame/ic-host-tooling/issues/72),
[fixture pruning #74](https://github.com/dragginzgame/ic-host-tooling/issues/74),
[Shared adoption #73](https://github.com/dragginzgame/ic-host-tooling/issues/73),
[interface consumers #71](https://github.com/dragginzgame/ic-host-tooling/issues/71)
and [lock isolation #70](https://github.com/dragginzgame/ic-host-tooling/issues/70).

The [historical handoff](archive-2026-10-13.md) preserves older source identities,
validation paths and consumer coordination. Its status claims describe their
original inspection, not current release state. GitHub issues remain the sole
follow-up tracker. Update this handoff in place instead of appending history.
