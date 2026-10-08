# Convergence audit after Host 0.7.0

## Scope, identity and verdict

Requested 2026-10-08: check latest Shared Tooling, then continue audits and issues.
Method: [flow convergence and duplication](../../../../../../../../audits/flow-convergence-and-duplication.md),
Shared Tooling `75a8a60f49cec11d3f6aecab5c977029c42cc549` (0.1.26).
The method is unchanged from run 02; this run refreshes selected source/adoption
evidence, without a whole-system correctness or performance claim.
Overlay: Host AGENTS.md and docs/extraction.md. Host began clean at released
`491fc0e231b9650526f5f57b9ab7b1f62f02218c` (0.7.0). The local 0.7.1 tooling
adoption and report changes are distinct from that committed runtime source.

[Inventory](inventory.json) records exact sibling HEADs, working changes,
manifest/lock/snapshot digests and textual Host reverse edges. All sibling
repositories remained read-only. Many are changing concurrently: the inventory
captured Testkit's 0.6 lock before its manifest advanced to 0.7; the later
[selected digests](source-digests.txt) bind that observation. These are dated
working-source facts, not delivered consumer dependency selections.

**Verdict: FAIL for Toko's reproduced Wasm reporting/enforcement contract.**
Other selected owners show adoption progress or bounded compatibility/acceptance
findings. No additional generic Host runtime owner is justified by this review.
This frozen report is evidence; linked GitHub issues own further work.

Scope: shared release/snapshot mechanics, host process/byte/file/Wasm reuse and
their selected consumer edges in Canic, IcyDB, Query, Testkit, Backup, Blob
Storage, Timers, Toko Miner and Toko. Application business logic, general canister
correctness, cryptography, paid recovery correctness, performance and complete
consumer builds are excluded. Native qualification gaps below are not exclusions.

## Owner and entry-to-result trace

| Behavior | Canonical owner and carried contract | Current consumer boundary |
| --- | --- | --- |
| Delivered release → confirmed remote observation → local Git status | Shared release runner, matching locked tracking-ref update | Host adopts exact 0.1.26; metadata/validation policy stays local |
| Committed source → declared export → verified local bytes | Shared snapshot exporter and independent verifier | 68 Host files retained; dirty upstream bytes excluded |
| Caller command → bounded IO → original failure and cleanup evidence | Host capture/communicate engine and OwnedChild | Toko encoder uses group capture; Query still owns Python lifecycle |
| Executable candidate → closed-writer admission → publication | Host fs write_validated_with | Canic's dirty publish_executable now delegates here; bundles remain local |
| Wasm bytes → structural facts → reference-limit result | Host artifacts inspector and tools install report | Canic build admission delegates; Toko CLI parser remains separate |
| Paid dispatch → custody/journal → recovery | Backup/Canic operation owners | Do not substitute group cleanup for paid-operation custody |

## Findings and disposition

1. **MEDIUM — Toko retains a demonstrated duplicate decoder defect.**
   Toko is clean at `44d4e2c6d41e3575b2503e79ffa369129ff3ecf2`; its parser hash
   is unchanged from run 01. Fresh Linux Node 24.21.0 execution of the retained
   [fixture generator](fixtures.cjs) gives these [exit results](results.tsv):
   50,000 definitions plus one import rejects in text mode but succeeds with
   JSON+enforcement; global-before-function loses a function import; an 11-byte
   truncated code section succeeds in both modes. Individual text/JSON logs are
   adjacent. Earlier Host comparison evidence remains bound to run 01; the Host
   Wasm/limit source diff from that release to 0.7 is empty, but no new Rust
   probe execution is claimed. The two current shell callers use text mode, so
   the JSON result is a combined-option defect, not a claimed bypass there.
   Disposition: expose Canic's now-shared report via one CLI outcome projection,
   then delete Toko readLeb/wasmMetrics while retaining optimizer and raw-budget
   policy. Updated owners:
   [Toko #1791](https://github.com/dragginzgame/toko/issues/1791#issuecomment-6059573941)
   and [Canic #481](https://github.com/dragginzgame/canic/issues/481#issuecomment-6059573203).
   No new Host parser or general-purpose CLI is needed.

2. **MEDIUM — Testkit's new public dependency boundary needs a minor release.**
   Testkit `59b1c1de924116752282eac48c6531dce159ccc9` is dirty and publicly
   re-exports all four Host crates. Its manifest advanced to 0.7 during this
   audit while notes still named a 0.23.1 tooling patch. Host's changed enum
   variants and Rust crate identities make completion a Testkit minor boundary.
   IcyDB's inspected lock contains direct Host 0.7 alongside Host 0.6 through
   Testkit 0.23.0. This is real public coupling and textual dependency evidence,
   not a measured runtime/size defect. Disposition: coordinate the next minor,
   update matches, qualify exact consumer graphs, then retire obsolete edges.
   [Testkit #32](https://github.com/dragginzgame/ic-testkit/issues/32) owns this
   new boundary; the previously completed #25 is not reopened. Do not manufacture
   aliases or upgrade unaffected consumers solely to make version counts equal.

3. **MEDIUM — Query process integration remains a consumer contract decision.**
   Released Host 0.7 supplies bounded duplex communication, cancellation and
   explicit success retention, with exact native CI now passing. Current Query
   `a05ec4d2a` remains on its Python process helper with concurrent receipt work;
   its manifest/lock moving artifacts/fs to Host 0.7 does not adopt process IO.
   Query's TERM grace and bounded reap differ from Host's synchronous KILL/reap.
   Disposition: retain that boundary until current receipt, signal and background
   lifecycle acceptance is demonstrated. The earlier isolated withdrawn bridge
   is evidence, not an apply-ready current patch. Updated
   [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6059575927)
   retains that obligation; no further Host controller is proposed.

4. **LOW — Host's confirmed release observation was not carried to local status.**
   Against the old adopted runner, the new owning real-Git fixture delivers its
   release successfully, then reports `ahead 1`: see
   [delivery](release-before-delivery.log) and [status](release-before-status.log).
   Disposition: fixed locally by adopting the canonical 0.1.26 runner. All 19
   real-Git tracking cases pass after refresh, including update/symbolic races,
   mismatched mappings, completed resume and lost push. No consumer-local second
   updater was added. [Shared #62](https://github.com/dragginzgame/shared-tooling/issues/62#issuecomment-6059575189)
   records pending Host delivery and separate native qualification.

## Adoption progress and deliberate retention

Canic's dirty selected source already delegates single-executable publication to
write_validated_with and Wasm inspection/comparison to the shared owners. Earlier
apply-ready patch recommendations are superseded for those working files.
Publication/runtime-library bundle policy and original paid custody remain local;
Backup's current original-plan/direct-transport work is separately tracked in
[Backup #29](https://github.com/dragginzgame/ic-backup/issues/29). A standard Child
returned by custody spawn and OwnedChild communication are not automatically
interchangeable. No unsupported integration or deletion is claimed.

Toko Miner's dirty encoder uses capture_group_command and no-follow output reads.
Its owner records actual Linux checks in
[Toko Miner #33](https://github.com/dragginzgame/toko-miner/issues/33#issuecomment-6059251907);
those checks were read, not rerun here. Managed/native application acceptance is
still separate. Blob Storage, Timers and Toko Miner select a consistent Host 0.6
through Testkit 0.23 in the inspected working locks; Canic retains Host 0.5.2
through published Backup/Query alongside its direct 0.6. Concurrent source moves
must be qualified at delivery instead of inferred from a dirty manifest.

Shared Tooling source advanced from adopted 0.1.23 to committed 0.1.26. Host used
a clean private checkout, never the upstream dirty changelog/test edits. Existing
consumer snapshot issues remain their adoption owners:
[Canic #453](https://github.com/dragginzgame/canic/issues/453),
[IcyDB #299](https://github.com/dragginzgame/icydb/issues/299) and
[Query #20](https://github.com/dragginzgame/ic-query/issues/20).
Their recorded snapshot identities in inventory are not native qualification.
Optional evidence archiving is not silently added to Host's selected file set.

Private framing, filesystem trust boundaries, application retries and retained
recovery evidence keep their existing owners. Textual similarity alone does not
justify replacing those paths. No measured performance claim is made.

## Verification and limits

Host [0.7.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37773664766)
passes Linux x86-64, Rust 1.88 and native macOS 15 Intel/Apple Silicon. This
qualifies the released communication engine, not the uncommitted tooling refresh
or registry publication. [Exact Shared Tooling CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37770856593)
passes Linux, Apple Silicon and lint/security; Intel remained running at the
observation recorded here. Host's new adoption still needs matching native CI.

Focused local checks passed: release-runner fixtures, Host adapter/receipt and
Make wiring, snapshot distribution, tooling LOC, selected ShellCheck, Perl syntax
and exact 68-file snapshot verification. The old-runner control fails as expected
at the first real tracking case. Initial tooling LOC execution lacked cloc in
PATH; using the already provisioned Host tool directory passes. No download or
online retry occurred. Logs, clean source clone and negative fixture controls
remain under `/tmp/ic-host-audit-20261008-03/`. Initial searches using guessed
Testkit/Toko subpaths were corrected to their actual crate/root paths.

Selected documentation links and whitespace are checked at handoff. No Rust
source/dependency change, broad local gate, sibling edit, commit/push, release
or publication ran. No function, method or type was removed. The 0.7 detailed
notes now carry version-only headings so the root ledger is the sole current
draft/date owner; the runtime release summary is preserved.
