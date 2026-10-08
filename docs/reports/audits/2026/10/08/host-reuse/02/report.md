# Sibling convergence audit after Host 0.5.1

## Scope, identity and verdict

Requested 2026-10-08: audit Host and sibling repositories to remove redundant
mechanisms and align adoption. Method:
[flow convergence and duplication](../../../../../../../../audits/flow-convergence-and-duplication.md),
Shared Tooling 0.1.23 `0ba0ad00ed94848e54ecc82629b6b7873b7284c0`.
Overlay: Host AGENTS.md and docs/extraction.md at released
`81f9809861159def2fd0987fcb7961cda4afd969` (0.5.1). Host started clean.
[Inventory](inventory.json) records exact sibling HEADs, dirty state, selected
manifest/lock/snapshot digests and textual lock reverse edges.
[Selected source digests](source-digests.txt) bind implementation observations.
Siblings were read-only and actively changing: this is a point-in-time working
source inventory, not a claim that pending dependencies are released.

**Verdict: FAIL for the retained Toko reporting/enforcement contract**, with
historical reproductions applicable to the unchanged selected source. The
[preceding report](../01/report.md) owns those executions; they were not rerun.
No new Host runtime defect was established. A newly demonstrated Query lifetime
requirement prevents treating all local process code as safely replaceable.
Missing native consumer evidence remains a gap, not proof of a runtime failure.
This report is frozen evidence; the linked GitHub issues own follow-up.

This selected audit covers host byte/file/process/Wasm mechanisms, dependency
selection and common release orchestration. Canister runtime, frontend business
logic, cryptography, schema correctness, paid-operation recovery and general
application correctness are excluded. The method revision changed since run 01;
aggregate comparison is N/A (method change). Named unchanged source/fixture
anchors below remain comparable.

## Owner and entry-to-result trace

| Behavior | Canonical mechanism and carried result | Consumer projection retained |
| --- | --- | --- |
| File read/hash/copy → bounded chunks → identity/result | artifacts visit_reader; fs opens/adopts descriptor | Path admission, budgets, schema and corruption policy |
| Command/server start → poll/cleanup → status/error | process OwnedChild | Testkit readiness, observer semantics and original errors |
| Successful background network start → running launcher → later stop | Missing explicit Host handoff contract | Query owns network identity/readiness/stop; existing lifecycle retained |
| Executable producer → closed-writer admission → publish outcome | fs durable::write_validated_with | Canic pins, version checks, bundles and typed cleanup projection |
| Wasm bytes → structural facts → selected budget comparison | artifacts wasm::inspect and tools install_limits | Canic/Toko report schema, optimizer invariants and product limits |
| Release hook → final identity admission → delivery/resume | Shared Tooling canonical run-release.sh | Consumer metadata, receipts, gate and delivery selection |

## Prioritized findings

1. **MEDIUM — Query needs a deliberate successful lifetime handoff.**
   Classification: contract gap, not equivalent duplicate flow. The attempted
   Rust bridge was withdrawn after a successful-start fixture showed that the
   background launcher's heartbeat stopped. Current Host `try_wait`/`wait`
   signals the owned group before reaping, as documented; Query needs the
   launcher to survive until its network stop operation. Owner evidence and
   failed fixture are in [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6055899595).
   This audit verified the current Host and Python source, not that fixture anew.
   Disposition: retain Query cleanup until an opt-in shared disposition proves
   successful survival, failed-start/cancel/timeout cleanup, exclusive leader
   reservation and no signalling after ID reuse. Keep Testkit's default intact.
   Assess the owner API and real consumer deletion together; do not add an
   unconditional escape hatch or another detached process engine.

2. **MEDIUM — Testkit's old public dependency boundary keeps two Host minors.**
   Classification: late convergence. Blob Storage has artifacts/fs 0.5.0 plus
   0.4.6 via Testkit 0.21.3; IcyDB has all four at both versions. Toko Miner's
   dirty direct fs/process selection is 0.5.1, while Testkit retains 0.4.6.
   Timers selects 0.4.6 through Testkit alone. Testkit's pending source already
   uses OwnedChild in startup and observed Cargo and selects 0.5.1, but its
   required 0.22 compatibility boundary is not delivered in the reviewed HEAD.
   Disposition: qualify/release that boundary, then update downstream callers
   and retire old lock edges through their approved resolution workflow.
   [Testkit #25 now carries these edges](https://github.com/dragginzgame/ic-testkit/issues/25#issuecomment-6056049867).
   Canic also selects old published Backup/Query: Testkit alone cannot converge
   its complete graph. Toko's much older Testkit 0.7.6 remains under
   [Toko #1796](https://github.com/dragginzgame/toko/issues/1796).
   Lockfile presence is not a claim of runtime linkage for every target.

3. **MEDIUM — Canic and Toko still retain the largest ready deletion route.**
   Classification: duplicate flow/policy rediscovery. Canic's single-executable
   publication/limit replacement still passes `git apply --check` at reviewed
   HEAD `e286b3fd98460c98670336853f80658a920966e0`; selected Wasm/installer
   source hashes match run 01. No replacement was applied to the real checkout.
   [Canic #458](https://github.com/dragginzgame/canic/issues/458) owns adoption.
   Toko's JS parser also has the identical run-01 hash: definition/import drift,
   malformed input acceptance and JSON/enforcement inconsistency remain evidenced
   by that earlier isolated execution. Converge through the command owned by
   [Canic #481](https://github.com/dragginzgame/canic/issues/481), then remove
   the parser and update both shell callers under
   [Toko #1791](https://github.com/dragginzgame/toko/issues/1791).
   Preserve report meanings, local budgets and optimizer policy. These are
   consumer adoption tasks; Host does not need another Wasm abstraction.

4. **MEDIUM — Four consumers still lack canonical final-release checks.**
   Classification: snapshot adoption drift. Canic records 0.1.18, IcyDB and Blob
   Storage 0.1.20, Query 0.1.22. Their direct runners lack the final-hook and
   completed-direct identity repair released in Shared Tooling 0.1.23 #58.
   This is source comparison plus prior upstream/Host substitute evidence,
   not a newly executed consumer release failure. Refresh the reviewed snapshot
   through its exporter, keep consumer receipts/adapters and qualify the owning
   mutation/resume fixtures. Existing owners updated:
   [Canic #453](https://github.com/dragginzgame/canic/issues/453#issuecomment-6056048639),
   [IcyDB #299](https://github.com/dragginzgame/icydb/issues/299#issuecomment-6056048945).
   New nonduplicate adoption issues:
   [Query #20](https://github.com/dragginzgame/ic-query/issues/20),
   [Blob Storage #28](https://github.com/dragginzgame/ic-blob-storage/issues/28).
   Toko Miner moved to 0.1.23 during this audit; the final inventory does not
   classify its earlier baseline as current. Toko's broader baseline gap already
   belongs to [#1789](https://github.com/dragginzgame/toko/issues/1789).

## Deliberate retention and verification

Host byte reads/hash/copy converge on the existing bounded visitor. Filesystem
admission and opened-descriptor reads retain distinct security inputs. Pull
readers and output writers preserve different failure/partial-write contracts;
merging their loops by appearance would not reduce semantic ownership. Tool
admission remains separate from caller-admitted execution. Canic's inherited
operation-lock descriptor requirement remains open under #5; no retry/recovery
policy belongs in Host. Keep Backup directory framing/private crash barriers,
Query confinement, Testkit domain framing/reused scratch buffer and Canic bundle
staging with their owners. No new generic helper is justified by these cases.

State-space reduction sought: one lifecycle owner where lifetimes match, one
Wasm decoder/comparison owner, one single-file publication engine, one reviewed
release runner and fewer incompatible dependency edges. No measured performance
or binary-size claim is made. Forcing every consumer to the same 0.5 patch is
unnecessary: `git diff` confirms no Rust source change from Host 0.5.0 to 0.5.1.

Performed: source/manifest/lock inspection, issue searches and owner updates,
selected hashes, patch applicability, exact-release CI inspection, documentation
link and whitespace checks. Initial Node child-process inventory failed with
EPERM; direct shell Git reads plus pure Node parsing produced the saved inventory.
An initial guessed Testkit startup path was absent; inspection used its actual
`src/pic/startup.rs`. No Cargo resolution/build, full local gate, download,
sibling write, package bump, Git commit/push or publication ran. No function,
method or type was removed.

Host's [exact 0.5.1 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37750135927)
passes Linux x86-64, Rust 1.88, macOS 15 Intel and Apple Silicon at the report's
Host SHA. [Host #22](https://github.com/dragginzgame/ic-host-tooling/issues/22)
is closed with that delivery/native evidence. These results do not qualify
pending consumer changes or independently verify registry publication. Testkit,
Query and the other dirty integrations need their own committed/native evidence.
