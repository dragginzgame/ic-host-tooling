# Current handoff

## Source and ownership

The delivered baseline is 0.67.0, commit
`248df96b27b44366095d9d8e195cca8a1023bde1`. The clean release passed
[CI](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807665),
[browser](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807675),
[runtime](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807707) and
[optimizer](https://github.com/dragginzgame/ic-tooling/actions/runs/40083807692)
qualification, including native macOS 15 Intel and Apple Silicon. Those results
qualify the released source, not subsequent local dependency changes.

Host, Build and Testkit retain the [extraction boundaries](../extraction.md).
Consumers own credentials, budgets, targets, retries, installation and recovery.
Shared Tooling 0.26.10 at `a8141e7` is recorded in
[the snapshot manifest](../../.shared-tooling.snapshot). Rust edition 2024 and
MSRV 1.91 remain required. Package versions are still 0.67.0.

## Current local work

Adopt published Wire/derive 0.7.0, following the supplying release
`aa7cc32acbd54a58d77720bedfb1ae530926f57e`. The incoming lockfile already
selected Platform/macros 0.6.14 and Wire/derive 0.6.13; preserve its independent
Platform update. This task changes only Wire selections in that starting lock.
The registry packages were available in the local dependency cache.

Testkit responses and private probe request decoding no longer require Serialize.
Wire now validates primitives in one pass, permits consumer-owned collection
normalization and returns Malformed for nonminimal varints. Typed helpers keep
owned responses and one-call behavior; raw calls retain original-byte evidence.
Read the [0.68 migration notes](../changelog/0.68.md) before adopting exposed Wire
types or relying on decoding/re-encoding for byte identity. The next draft is
0.68.0 because public Wire types and decoding semantics change. No own version,
Git ref, publication or installation policy is changed by this local adoption.

Focused adoption logs and the starting source snapshot are retained at
`/tmp/ic-tooling-wire-07/`. Eight adapter cases pass on development Rust and
actual Rust 1.91, including Deserialize-only and normalization admission. Private
protocol limits and the real PocketIC query/update, caller, rejection and trap
probe pass on Linux. The prepared server required no download; its first sandbox
run could not bind localhost, and the admitted native rerun passed. Changed-caller
Clippy, MSRV compilation, probe/Testkit Wasm checks, graph policy, formatting and
local doc links pass. Native macOS qualification of this later Wire graph has not
run; the earlier release matrix does not supply it.

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
