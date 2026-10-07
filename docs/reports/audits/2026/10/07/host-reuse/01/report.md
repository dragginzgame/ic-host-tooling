# Host reuse audit — 2026-10-07

## Scope, method and verdict

Trigger: maintainer requested current/sibling code review, improvement here and
removal of duplicated consumer mechanics. Named Canic and Toko Miner replacement
patches were subsequently approved explicitly. All other siblings stayed read-only.

Method: [flow convergence and duplication](../../../../../../../../audits/flow-convergence-and-duplication.md),
with the [common evidence contract](../../../../../../../../audits/README.md), at
Shared Tooling revision `46c02774a8335cb3949d6f04284c4f53375353c1`.
[Inspection identities](inspection-identities.json) record the method/overlay
bytes, source HEADs, dirty state, selected source hashes and lock package selections.
The reviewed local Shared Tooling HEAD was newer and dirty; it was discovery input,
not an implicit change to this repository's adopted method or snapshot.

Baseline: Host `38a2a5127be064014e6d39d72d0300ffb2cf20be`, released 0.3.1.
Thirteen siblings were discovered; generic host mechanics were traced in Canic,
Query, Testkit, Backup, Memory, IcyDB, Blob Storage and Toko Miner. No direct split
crate use was found in the selected Metrics, Timers, ICHelper, Shared Tooling or
Toko Miner Assets source families. This does not rule out unrelated duplication.
Inventory file hashes bind the inspected working bytes, not only their commits.
[Completion identities](completion-identities.json) retain the final observation;
concurrent sibling work is not imported or silently counted as audit qualification.
Backup advanced its HEAD and Backup/Blob Storage changed manifests/locks between
these observations. Their baseline findings refer to the inspection inventory;
this review does not qualify those independently updated graphs.

Baseline **FAIL**: native directory traversal and dangling alias contracts had
reproducible violations. After the shared repair, the selected Linux/source
convergence review is **PASS WITH FINDINGS**: Query adoption awaits publication,
and published transitive consumers retain old tooling selections. This is not a
whole-system correctness, release-readiness or complete host qualification verdict.
Comparison with yesterday's reports is limited to the owner map and entry surfaces;
changed dirty consumers/releases mean their older passing checks are not reused.

Excluded: canister/domain hashes, paid-operation recovery, capability confinement,
product schemas, arbitrary performance/security review, generated/vendor trees and
retained target artifacts. These belong to different semantic owners. Native macOS
and complete consumer graphs remain qualification gaps, not passing exclusions.

## Owner and flow traces

| Behavior | Single mechanical owner | Entry and result projection | Retained consumer contract |
| --- | --- | --- | --- |
| Gzip publication | artifacts `encode_gzip`, fs `write_with` | Canic captured Wasm → staged encoding → durable replacement; in-memory measurement uses the same encoder | source capture, best compression, output allowance, set publication/order, install limits |
| Streamed file digest | artifacts `hash_reader` | Canic admitted descriptor → bounded identity → frontend digest | NOFOLLOW/regular-file admission, exact length, tree traversal/budgets and domain framing |
| Lowercase SHA-256 bytes/text | artifacts `Sha256Digest` | Canic network adapters → strict parse/compute/display → product DTO/errors | trust anchor, authority ordering, serialization and recovery |
| Ordinary missing path resolution | fs `path` | explicit base → existing native resolution/iterative missing target expansion → canonical pathname | Query depth allowance, hard-link identity, before-create and before-truncate checks; Testkit cache identities |
| Browser output read | fs `read_file`, artifacts bounded stream | Toko `browser_bytes` → complete file read → bytes | 2 MiB bound, browser execution and invocation cleanup |
| Exact serialization match | artifacts `MatchingWriter` | Query compact serializer → complete match decision | encoding/schema, serializer success, capability confinement |
| Durable publication/lock wait | fs `durable` | Testkit/Memory and Canic direct calls → caller error/observation projection | retry/deadline/namespace, cache claims and product journals |

Backup already delegates streamed checksums/copy but correctly retains private
parent/file modes and crash barriers. Query's capability-opened reads correctly
use shared stream mechanics without replacing confinement with pathname APIs.
Blob Storage retains immutable publication/identity admission. IcyDB retains
product CLI mappings and query contracts while importing generic readers/tools.
These are protective/domain distinctions, not candidates for blanket deletion.

## Findings and dispositions

1. **MEDIUM — filesystem path contract gap; repaired here.** On the released
   baseline `file/..` returned the directory instead of preserving a native
   non-directory failure. Missing-suffix rewind also discarded trailing directory
   requirements, and a dangling alias returned its alias pathname instead of the
   missing target. [Baseline traversal failure](ic-host-path-traversal-before.log)
   and [alias/cycle failures](ic-host-path-alias-before.log) are actual executions.
   The shared iterative owner now expands relative/absolute targets, detects active
   cycles, preserves native errors and does not recurse on arbitrarily long valid
   missing-target chains. A new explicit-depth entry retains Query's caller-owned
   64 expansion allowance; the existing entry uses the same engine without that
   policy cap. Neither entry supplies confinement or race-proof opening.
   Disposition: fixed working source, pending 0.3.2 notes; publication not performed.
   Owner: [Host #1](https://github.com/dragginzgame/ic-host-tooling/issues/1).

2. **LOW — duplicated Canic mechanics; consolidated with authorization.** Local
   gzip construction, frontend streamed hashing and digest nibble decoding had
   equivalent shared owners in published 0.3.1. [Applied source patch](canic.patch)
   removes 64 lines and adds 27 across four files while keeping product adapters.
   This is footprint evidence, not a performance claim. The maintainer separately
   committed exactly these source surfaces as
   `d7698e1f0541cb52ad8d762bc1549a700b412e88` during validation; this agent created
   no commit. Candidate hashes were rechecked against actual source afterward.
   Owner: [Canic #458](https://github.com/dragginzgame/canic/issues/458).

3. **MEDIUM — stale Toko API use; repaired with authorization.** The direct
   dependency selected tools 0.3.1 while `browser_bytes` still called its removed
   artifact `read_file`. [Applied patch](toko-miner.patch) selects the filesystem
   owner directly. Offline lock synchronization replaces that edge and removes
   only unreachable process/tools 0.3.1; no retained package version changes.
   Native User Hub test compilation now passes. No function/type is removed.
   Owner: [Toko Miner #30](https://github.com/dragginzgame/toko-miner/issues/30).

4. **LOW — pending consumer deletion and registry convergence.** Query can remove
   private `resolve_output_path` after a published filesystem version supplies the
   corrected bounded API. [Prepared patch](ic-query.patch) keeps inode comparison,
   opening/rechecks and local depth policy; it is not applied. Its future 0.3.2
   minimum is a proposal, not an available registry version verified here.
   Canic still locks tools 0.2.0 via Query 0.47.6 and tools 0.1.14 via Testkit
   0.19.2. IcyDB, Blob Storage and Toko also lock Testkit 0.19.2/tools 0.1.14.
   Their dirty/local split adoption does not change those published graphs.
   No blanket dependency upgrade or compatibility reexport was introduced.
   Owners: [Query #11](https://github.com/dragginzgame/ic-query/issues/11),
   [Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13).

5. **LOW — stale extracted fixtures; deleted here.** Caller/include/script searches
   found no maintained references to the two fs shell fixtures, process Wasm
   fixture, and tools Wasm/git fixtures. The genuine owning fixtures remain.
   Exact deleted paths are in the removal inventory below. Selected process/tools
   library checks passed after deletion. No production API or named symbol is
   removed by these five deletions.

## Removed-symbol and artifact inventory

Actual named removal: Canic
`crates/canic-host/src/network/mod.rs::decode_nibble`, private function. Strict
`Sha256Digest` parsing replaces its validated-byte hex decoding. Other adapters
(`sha256_digest`, `parse_fingerprint`, `encode_digest`, `hash_file`,
`deterministic_gzip_bytes`, `write_gzip_artifact`, `browser_bytes`) remain with
current caller policy and shared mechanics. No methods or types were removed.

Actual artifact removals here:

- `crates/ic-host-fs/tests/fixtures/git.sh`
- `crates/ic-host-fs/tests/fixtures/tool.sh`
- `crates/ic-host-process/tests/fixtures/empty.wasm`
- `crates/ic-host-tools/tests/fixtures/empty.wasm`
- `crates/ic-host-tools/tests/fixtures/git.sh`

Those files contain no named functions/methods/types. Proposed, not actual,
removal: Query `cache_file::write::output::resolve_output_path`; replacement is
shared bounded path resolution with explicit base and the consumer's depth cap.
Earlier consumer deletions in other sessions are not counted as this audit's work.

## Verification and limits

Linux x86-64; Rust/Cargo 1.99.0. All Cargo checks were offline after explicit
selected-lock cache preparation. Canic already selected Host 0.3.1. Toko's first
locked preparation refused its changed dependency edge; [failed log](toko-host-reuse-fetch.log)
is retained. Its authorized offline sync succeeded without retained-version
changes. Host manifests, lock selections, tool pins and shared snapshot unchanged.

- `cargo test --locked --offline -p ic-host-fs --all-targets --all-features`:
  [39 passing tests](ic-host-reuse-fs.log), including seven pathname cases.
- `cargo clippy --locked --offline -p ic-host-fs --all-targets --all-features -- -D warnings`:
  pass. The earlier panic-documentation Clippy diagnostic was resolved by removing
  the avoidable `expect`; it was not a runtime failure.
- Filesystem Rust 1.88.0 all-target/all-feature check and strict all-feature
  Rustdoc: pass during this repair. No toolchain download occurred.
- Process all-feature library tests (24), tools all-feature library tests (19):
  pass after unused fixture deletion, recorded in the execution tool outputs.
- Canic `cargo test --locked --offline -p canic-host --lib network::`:
  [14 pass](canic-host-reuse-network.log). The resulting native executable
  `target/debug/deps/canic_host-d6640e8206839ccd` then ran
  `frontend::tests::payload_inventory` (2 pass) and `artifact_io::tests::`
  (16 pass), [recorded here](canic-host-reuse-direct-tests.log). Actual filesystem
  and compression boundaries execute; IC/tool fixture commands are substitutes.
  An independently started workspace Cargo build was preserved, not counted as
  this audit's verification. Changed HEAD alone does not requalify its graph.
- Toko `cargo check --locked --offline -p canister_toko_miner_user_hub --tests`:
  [pass](toko-host-reuse-check.log). Compile only; browser/PocketIC not executed.
- Query candidate: five actual output/export tests pass in
  `/tmp/ic-query-host-path-20261007`, [log](query-output-fixture.log). Actual candidate
  output, path helper, tests and temp-dir support are used with a reduced error
  enum and local shared fs source. The disabled `nns-host` full refresh case,
  feature manifest integration and real consumer build remain unqualified.
- Formatting, diff whitespace, adopted snapshot, dependency declarations and
  documentation links pass, including this report's local references and scoped
  consumer source formatting. The snapshot still verifies all 54 files.

The concrete [Query fixture](query-fixture.tar) and its [digest](query-fixture.sha256)
retain the executed reduced test graph. [Toko's task-only lock delta](toko-lock-delta.diff)
distinguishes this replacement from the pre-existing dirty dependency updates.
Owning feedback was submitted to
[Host #1](https://github.com/dragginzgame/ic-host-tooling/issues/1#issuecomment-6032567964),
[Query #11](https://github.com/dragginzgame/ic-query/issues/11#issuecomment-6032568180),
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6032568502),
[Toko #30](https://github.com/dragginzgame/toko-miner/issues/30#issuecomment-6032568748)
and [Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13#issuecomment-6032569004).

No full CI/release gate, native macOS execution, upload, tag, push or deployment
was performed. The shared fix and Toko adoption are working edits, with numbered
pending notes and unchanged package versions. GitHub issues own future adoption
and qualification; this immutable report is evidence, not a work queue.
