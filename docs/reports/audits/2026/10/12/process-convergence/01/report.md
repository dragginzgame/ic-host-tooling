# Process completion and consumer convergence

Read-only source review on Host 0.14.2
`fed8724e79862d8273d01823619fe648c981c019`, using
[`audits/flow-convergence-and-duplication.md`](../../../../../../../../audits/flow-convergence-and-duplication.md)
at Shared `6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d`, under
[Host AGENTS](../../../../../../../../AGENTS.md) and the
[extraction contract](../../../../../../../extraction.md).
The continuation requested the next process/consumer review after release.
Incoming Host Cargo.lock updates select serde_json 1.0.154, syn 3.0.8 and
unicode-ident 1.0.27; this pass preserves them and does not qualify that graph.
[Input identities](inputs.txt) distinguish dirty manifests/locks from commits.

## Scope and trace

| Behavior | Canonical owner and carried contract | Reviewed projections |
| --- | --- | --- |
| Capture and communication | `tool/process.rs::exchange` owns bounded pipe IO, cancellation and completion | Raw command, admitted tool and existing-child entrypoints converge here |
| Failed execution and observers | `read_chunk` records retained bytes/truncation before observation; `finish_capture` preserves cause and separate cleanup evidence | Canic forwards terminal errors directly from the fallible observer |
| Child disposition | `child::OwnedChild` owns direct/group termination, reserved leader status and explicit handoff | Nonzero retained leaders use ordinary cleanup; successful retained leaders require caller disposition |
| Result admission | `CompletedExecution` carries required status; explicit success/completeness checks preserve evidence | Host version/Git/Candid paths require both; Testkit complete probes carry the result directly |

No duplicate IO engine, observer cancellation workaround or completed-status
reconstruction remains in these reviewed paths. Direct-child and process-group
ownership deliberately remain separate policies. Success admission, complete
output admission and consumer publication are distinct boundaries and must not
be collapsed. The bounded-cleanup timeout and no-build-deadline contracts remain
unchanged. No production function, method or type was removed.

Reviewed existing observer tests cover both streams, zero/hard retention, original
sink causes, descendant pipe closure and cleanup failures; completion tests cover
signals and retained ownership. This was source inspection, not a new execution
or exhaustive process correctness/security/performance audit. Filesystem and
artifact implementation internals, live consumer operations and unrelated runtime
graphs were outside scope.

## Consumer finding

**LOW — remaining dependency duplication, owned by Toko Miner.** Among the seven
inspected current lockfiles, Canic, Testkit, Query, Backup and IcyDB contain only
Host 0.14.2; Timers contains only 0.14.1. Toko Miner retains Host 0.13.5 through
Testkit 0.33.8 alongside its direct Host 0.14.2 packages. Its root manifest still
declares Testkit 0.30.0, so these lock records are not proof of an admitted current
resolution. See [the exact lock inventory](lock-selections.tsv).

Reconcile the root Testkit selection and lock at
[Toko Miner #6](https://github.com/dragginzgame/toko-miner/issues/6), with its existing
application graph work. Use the published Testkit 0.34 contract, qualify the
selected packaged CLI/PocketIC and application callers, and confirm one Host
family without source overrides. The separate dual-Timers Wasm graph remains
that issue's independent acceptance requirement. No sibling edit, dependency
resolution or consumer build was performed here.

## Evidence and disposition

Verdict: **PASS WITH FINDINGS** for this bounded convergence review. No new Host
source defect or abstraction requirement was established. Keep the single IO
engine and explicit consumer policies; the remaining reduction belongs to the
application dependency owner.

Host 0.14.2 [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38159733484)
passes Linux/MSRV and both macOS 15 architectures. Testkit 0.34.0
`ccfd0dbc47a527e330cd55d91b1d02f83cb7a9f2`
[exact-source CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38150408955)
passes all twelve checks, portable-host, MSRV and concurrency jobs across those
hosts. Its reviewed probe/composition files match that commit. Canic's workaround
removal has its own recorded Linux proof under
[Canic #458](https://github.com/dragginzgame/canic/issues/458); current dirty-source
inspection does not establish delivered/native consumer acceptance.

[Host #66](https://github.com/dragginzgame/ic-host-tooling/issues/66) has completed
producer implementation and native qualification plus the requested concrete
consumer simplifications. Remaining consumer delivery/graph work retains its
own issues. Shared's separate staged-hook receipt gap remains with
[Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130); remote main
is still adopted 0.6.4. No shared repair is available to adopt in this pass.

Snapshot, documentation and diff checks qualify the evidence-only updates.
No full CI, compilation, tool setup, release or publication ran. Released-source
CI does not qualify the incoming Host lock changes or any dirty sibling graph.
