# Consumer crate usage review

Date: 2026-10-06. Trigger: maintainer asks how sibling repositories use IC Host
Tooling's crates. Method: `audits/flow-convergence-and-duplication.md`, exact local
Shared Tooling snapshot `21f3ec3dd97f2968c9f0b08924451bb2f71770d1`; overlay:
AGENTS.md and docs/extraction.md. Host HEAD:
`efd402e0063ccbbf8a143cc970be52ab41b1766d`, with existing uncommitted extraction,
license and other ongoing filesystem work. This review changes no library source.

Verdict: **FAIL** for complete consumer source/API alignment: Canic and Toko Miner
select tools 0.3.0 while referencing removed modules. This is established by
manifest/lock/source inspection against the checksum-verified package archive,
not by a compiler run in this review. Four other consumers have aligned split
imports in their working trees; two retain the older monolith. No broad correctness,
deployment or exhaustive duplication verdict is inferred.

## Scope and identities

Discovery covered the 13 sibling Git repositories. Active manifests, Rust host
call sites and 15 tracked workspace lockfiles were inspected. Hidden caches,
retained temporary snapshots, target/node_modules directories and historical docs
are excluded from current usage. Canister domain hashing, schemas and lifecycle
protocols were not audited. No sibling files or artifacts were mutated.

The [source inventory](source-inventory.json) records selected file digests and
dependency-table/reference lines. [Lock edges](lock-edges.jsonl) record locked
versions, sources, checksums and direct host-package edges.
[Repository identities](repository-identities.txt) distinguish HEAD from dirty work.
Relevant referenced files were rehashed after inspection and matched the saved
inventory. These are observed working-tree inputs, not an atomic Git snapshot.

| Consumer | Reviewed HEAD | Working-tree usage |
| --- | --- | --- |
| Canic | d815abfc661d791ecf72afc5b1e4b6a990f37a91 | Dirty manifest/lock selects tools 0.3.0; source still uses removed artifact/wasm/tool modules |
| Query | 0905078d1ef410c63a89fb2924cd18a0653ffa4e | Dirty adoption uses artifact streams, filesystem reads and IC response decoding from split 0.3.0 owners |
| Testkit | 827157434eb8b2d6c13c4b8e47493bd6a38678b9 | Dirty host-only split dependencies and public crate reexports; pending consumer notes select breaking 0.20.0 |
| Backup | 654790f374f9923df9020f4812cec65e47cbe3af | Root manifest/lock retains tools 0.2.0 |
| Memory | d56af42b0bd9ac5a794de2248e31b335945a1e1c | Native dev dependency for repo-tool example retains tools 0.2.0 |
| IcyDB | 049e561a843a8d3f4526460fd15876a4a238df10 | Dirty integration/CLI adoption selects split 0.3.0 owners |
| Blob Storage | 279b863b127168d20e610129619b78d474098659 | Dirty native CLI adoption selects artifacts/fs 0.3.0 directly |
| Toko Miner | 9071b4cc9c1cf0d6d905f73592154b5108ab2147 | Dirty manifest/lock selects tools 0.3.0; native qualification reader still uses removed artifact module |

Metrics, Timers (including its testing workspace), ICHelper, Shared Tooling and
Toko Miner Assets have no active direct host-crate manifest/call-site usage in this
scope. The three Canic blob-service workspace locks and Memory's runtime
qualification lock contain no host packages either.

## Owner and caller traces

| Consumer entry to result | Actual selected owner | Retained consumer authority |
| --- | --- | --- |
| Query pretty/canonical JSON -> bounded writer; confined opened stream -> read_reader | artifacts, optional under cache-bearing host features, default features disabled | JSON encoding/order, cap_std confinement, metadata admission, aggregate budgets and typed errors |
| Query CLI governance example -> regular Wasm read/inspection; stdin -> response decoder | fs dev dependency; artifacts dev dependency with wasm; tools dev dependency for response | Wasm/response limits and exact Candid text decoding |
| Testkit cache sidecars/read_wasm -> bounded regular reads; admitted producer fixtures/examples -> bounded execution | fs::read; artifacts identity/errors; process::tool; four host-only crate reexports | Cache framing, retention, destination custody, PocketIC process groups; local atomic writer remains |
| IcyDB optimizer -> file hash/executable resolution; size report -> hash/Wasm facts; CLI -> response decoding | fs::read, process::tool, artifacts with wasm, tools::response | Tool pins, raw process execution, output/report policy, database domains, schemas and Candid decoding |
| Blob native/probe inputs -> descriptor/no-follow reads and identities; installation record -> private create-new | fs::read/durable and artifacts identity/errors, native dependencies only | File admission, 0600 policy, byte budgets, no overwrite, schemas, upload/installation authority |
| Backup copy/JSON/upload record -> old bounded mechanics | tools 0.2.0 artifact module | Private parents/staging, crash barriers, descriptor traversal, journal and tree checksum framing |
| Memory repo-tool -> executable metadata hash and local atomic release receipt write | tools 0.2.0 artifact module; still-local writer | Version/release selection, receipts, Git effects and reconciliation |
| Canic host Wasm/gzip/read/resolution and Toko native browser read | Stale tools 0.3.0 paths | Must adopt direct artifact/fs/process owners while retaining consumer policy |

No inspected consumer calls the pending shared MatchingWriter, encode_gzip or
write_with. Query's private MatchingWriter and Testkit's write_file_atomic still
exist while those shared 0.3.1 additions await reviewed availability. Testkit's
whole-crate reexports name the actual owners; they do not restore removed tools
modules. Archive/wasm features are explicitly enabled by its host facade.

## Findings and tracking

1. **MEDIUM — incomplete source/API alignment.** Canic's host code and Toko's
   native test module still name tools::artifact/wasm/tool against directly
   selected 0.3.0. That package's public library exports only candid/response.
   Canic's existing [adoption comment](https://github.com/dragginzgame/canic/issues/458#issuecomment-6021707352)
   already records its owning compile attempt and repair; this review does not
   duplicate that comment or count its build as local evidence. The new
   [Toko #30 comment](https://github.com/dragginzgame/toko-miner/issues/30#issuecomment-6021838385)
   records the now-immediate mismatch. Ignored browser execution does not exempt
   the Rust test body from compilation. Direct imports and native target checks
   belong to those consumers; no upstream compatibility reexport is proposed.
2. **LOW — published dependency convergence is incomplete.** Canic's lock carries
   tools 0.2.0 through published Query 0.47.6 and tools 0.1.14 through published
   Testkit 0.19.2. IcyDB, Blob and Toko also retain 0.1.14 through Testkit. Query
   and Testkit local adoption does not change those registry packages. Their
   owning release/adoption issues are [Query #11](https://github.com/dragginzgame/ic-query/issues/11)
   and [Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13); downstream
   lock convergence follows actual reviewed publication. Version multiplicity
   here is a maintenance finding, not evidence of a runtime defect by itself.
3. **LOW — remaining older direct owners and duplicate mechanics.** Backup and
   Memory still use tools 0.2.0; their adoption remains in
   [Backup #11](https://github.com/dragginzgame/ic-backup/issues/11) and
   [Memory #16](https://github.com/dragginzgame/ic-memory/issues/16). Backup's raw
   checksum_reader also repeats a generic stream hash loop; the new
   [source-backed comment](https://github.com/dragginzgame/ic-backup/issues/11#issuecomment-6021839940)
   identifies existing hash_reader reuse and error qualification. Tree framing,
   private publication and recovery retain their current consumer owners.

## Evidence limits

During discovery, Query's example/reader and Blob's native imports changed in
their owning checkouts; the saved inventory captures the later aligned source.
An initial broad inventory also included hidden cached packages and frozen
fixtures; it is retained at /tmp/ic-host-usage-snapshot.json and excluded from
current usage claims. A guessed module-path search encountered absent mod.rs
paths; the actual qualification.rs/opening.rs/costs.rs chain was then traced.

All four cached 0.3.0 crate archives matched the checksums selected by the consumer
locks. The tools public module list was read directly from its verified archive.
GitHub issue bodies and recent comments were inspected before the two authorized
feedback additions. No live crates.io freshness assertion is made in this run.
Report links and JSON/JSONL parsing checks passed. No Cargo compilation/tests,
consumer cache preparation, broad CI, native macOS execution, version changes,
commits, tags, pushes or publication were performed. Linux inspection does not
qualify native Linux/macOS consumer behavior; previously reported consumer and
upstream checks remain separately attributed evidence.
