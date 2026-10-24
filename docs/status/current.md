# Current handoff

## Source and ownership

The delivered baseline is 0.68.0, commit
`8f281177faaeee0adc51edc1031d4a7b268b0ce0`. The preceding 0.67.0 release passed
[CI](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807665),
[browser](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807675),
[runtime](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807707) and
[optimizer](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807692)
qualification, including native macOS 15 Intel and Apple Silicon. Those results
qualify that historical source, not subsequent releases or local dependency changes.

Host, Build and Testkit retain the [extraction boundaries](../extraction.md).
Consumers own credentials, budgets, targets, retries, installation and recovery.
Shared Tooling 0.26.10 at `a8141e7` is recorded in
[the snapshot manifest](../../.shared-tooling.snapshot). Rust edition 2024 and
MSRV 1.91 remain required. Package versions are still 0.68.0.

## Current local work

Adopt published Wire/derive 0.7.1 from supplying release
`2978dee37ddaf62c09ce1df411a0b1b6c9748bcd`. The incoming lockfile already
selected both 0.7.1 packages and is preserved byte-for-byte. The workspace minimum
and consumer examples now require 0.7.1. See the
[0.68.1 notes](../changelog/0.68.md#0681---unreleased).

Wire 0.7.1 adds opt-in host schema advice and Packed<i8>; existing Tooling fields,
frame bytes and the 0.7 decoding contract remain unchanged. The next draft is
0.68.1 because the existing consumer API and semantics remain compatible. Own
package versions, Git refs and publication remain user-owned.

Current adoption evidence and the starting source snapshot are under
`/tmp/ic-tooling-wire-071/`. Eight adapter cases and the private protocol limits
test pass on Linux. Actual Rust 1.91 caller compilation, probe/Testkit Wasm checks,
strict changed-package Clippy, dependency/graph policy, formatting and doc links
pass. No Rust implementation or field changes were needed.
Original Wire 0.7.0 integration evidence remains under `/tmp/ic-tooling-wire-07/`,
including Deserialize-only and normalization cases and a real PocketIC probe.
Native macOS qualification of this later Wire graph has not run; historical host
results do not supply it.

## Dependency blocker

[Crypto qualification #240](https://github.com/dragginzgame/ic-tooling/issues/240)
remains owned separately, with the supplying correction at
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).
The selected graph still contains crypto-bigint 0.5.5. Actual ECDSA signing
prevents an unused-code exemption; no local crypto fork, override or exemption
was added. Wire adoption does not qualify or resolve that dependency risk.

## Retained evidence and recovery

The closed 0.67 producer issues and native logs own their delivery evidence.
Original audit evidence remains under `/tmp/ic-tooling-audit-fixes/`,
`/tmp/ic-tooling-audit-2/`, `/tmp/ic-tooling-audit-2-fixes/` and
`/tmp/ic-tooling-shared-02610/`. The original browser reproduction remains at
`/tmp/ic-tooling-audit-browser-RwZzM9/`.

The delivered nested-workspace target fix and hashing-phase correction are in
the baseline. Preserve `target/package-qualification-3180841-0/`,
`/tmp/ic-tooling-239/` and `/tmp/ic-tooling-package-phase/`. Earlier detailed
handoff evidence is retained in the
[released handoff](https://github.com/dragginzgame/ic-tooling/blob/248df96b27b44366095d9d8e195cca8a1023bde1/docs/status/current.md).

Migration and local-repository retirement archives remain under
`/tmp/ic-tooling-migration/`, including `retirement/`; original refs were
preserved in the former Host repository. Stale release-lock recovery evidence
remains at `/tmp/ic-tooling-stale-release-lock.qHQFTn/lock`. Retained plans,
failed attempts, original artifacts and backups are not disposable.
