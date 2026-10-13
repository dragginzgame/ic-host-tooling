# Current handoff

## Released source and baseline

The latest finalized release is **0.17.1**. This cleanup starts from clean commit
`b7ddc770ff0452e53a22f1f69a1edb7550e881d5`; package versions remain unchanged.
Shared Tooling **0.10.0**, revision
`ed9c26f015691859f67b50b512a94709dcdc0c6c`, is recorded in
[the snapshot](../../.shared-tooling.snapshot) and [AGENTS.md](../../AGENTS.md).

## Pending 0.17.2

The compatible maintenance batch narrows each subprocess test fixture to its
own callers and removes the unused tools SHA-256 dev-dependency
([#74](https://github.com/dragginzgame/ic-host-tooling/issues/74)). Production APIs,
crate boundaries and dependency versions are unchanged. The current handoff now
separates active evidence from preserved history
([#46](https://github.com/dragginzgame/ic-host-tooling/issues/46#issuecomment-6144677508)).
The IC-limits example's test helper now skips occupied names and preserves its
input on panic, including initialization-write failure ([#72](https://github.com/dragginzgame/ic-host-tooling/issues/72)).

Focused process, extraction and IC-limits tests pass. Full offline Linux
`make ci` passes again after the example follow-up. A disposable probe using the
actual example helper verifies occupied file/directory/symlink preservation,
successful cleanup, panic retention and immediate non-collision error reporting.
Example follow-up evidence is retained in `/tmp/host-0172-example/`.
ShellCheck, unexpected-mode rejection, documentation links and byte-for-byte
archive preservation also pass. Logs are retained in `/tmp/host-0172-cleanup/`.
Required native macOS 15 Intel and Apple Silicon qualification remains separate;
no commit, push, release or publication ran in this task.

## Evidence and coordination

The [historical handoff](archive-2026-10-13.md) preserves the previous document
byte-for-byte, including earlier validation paths, source identities and consumer
coordination. Its pending/released claims describe their original inspection
rather than current status. It links the still-earlier archive in the same directory.

GitHub issues remain the sole follow-up tracker. Existing qualification owners:
[Shared adoption #73](https://github.com/dragginzgame/ic-host-tooling/issues/73),
[0.17 cleanup #72](https://github.com/dragginzgame/ic-host-tooling/issues/72),
[interface consumers #71](https://github.com/dragginzgame/ic-host-tooling/issues/71)
and [lock isolation #70](https://github.com/dragginzgame/ic-host-tooling/issues/70).
Update this handoff in place; preserve older evidence without appending another
historical status section here.
