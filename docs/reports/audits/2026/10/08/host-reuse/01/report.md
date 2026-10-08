# Sibling host redundancy audit after 0.5.0

## Scope and identities

Requested 2026-10-08: audit sibling modules for further redundancy. Method:
[flow convergence and duplication](../../../../../../../../audits/flow-convergence-and-duplication.md),
Shared Tooling `2687f26317952c43c685f7f799ed09288dc10a67` (0.1.22).
Consumer overlay: root AGENTS.md and docs/extraction.md at Host
`db637fac8b7a9ef62301e1d9009ffeb5ffcd0be7`; exact relevant working-file hashes
are in [source-digests.txt](source-digests.txt). Host started clean. Shared
Tooling has pending 0.1.23 edits, excluded from adoption and method identity.
This is a fresh, selected host-code audit, not a whole-repository correctness
verdict or a comparable score against the earlier 0.3-series audit.

**Verdict: FAIL for the selected Toko reporting/enforcement contract.** Local
fixtures demonstrate contradictory exit behavior and incorrect/malformed facts.
The library extraction is sufficient for the selected consumer deletions; no
new Host abstraction is demonstrated. Consumer adoption is not yet qualified.
Siblings were inspected read-only. This report is frozen evidence; linked GitHub
issues own all follow-up work.

| Checkout | Reviewed HEAD | Relevant working-tree state |
| --- | --- | --- |
| Canic | `2140b0ba022bb963a3d390830ee2b5bccce498ac` | Two unrelated managed-root fixtures dirty; selected installer/artifact modules committed |
| Testkit | `a8e83a1940e5f44927c6df95b5d1269a3ac699bc` | Snapshot/host docs dirty; startup and digest modules committed |
| Toko | `44d4e2c6d41e3575b2503e79ffa369129ff3ecf2` | Clean; metrics script and both callers committed |
| Backup | `8a1152d0a510f34f8daed59f06632306f7cf134e` | Manifest/lock/changelog dirty, selecting Host 0.5 |
| Blob Storage | `8dd87013068c0560b6baae01d0a332050d20ab56` | Manifest/lock dirty, direct Host 0.5 alongside Testkit's Host 0.4.6 |
| Query | `08012b4bedd288b5882c0d8134ea94aa739228f6` | Host 0.5 selection and governance adoption dirty |
| IcyDB | `cb8cefca1d68c20025398eae6dfab18a2eb45883` | Active governance/tooling edits; inspected existing Host-backed CLI/build helpers |
| Memory | `692fbc81d698f4b8253565ff3036f3ad3fb42cd2` | Clean; repository helper already delegates file identities/copy |
| Toko Miner | `aeed004b03b9b5d848de3750d77a42c5160c5fdf` | Lock dirty; Host 0.4 host dependencies retained |

Metrics/Timers manifests were inventoried without a new host extraction found.
IC Auth has no committed runtime source yet. Canister runtime hashing, schemas,
cryptography, application retry/paid-effect recovery, frontend code and unrelated
GitHub backlog are outside this host audit. Shared snapshot copies are deliberate
reviewed distribution, not redundant production owners to delete locally.

## Owner and entry-to-result trace

| Behavior and entry surfaces | Existing canonical mechanism | Consumer obligations retained |
| --- | --- | --- |
| Testkit command runner and managed server spawn → poll/cleanup → original startup result | process `child::OwnedChild` | Readiness, logs, deadline/signal interpretation, borrowed URL, original failure precedence |
| Canic Binaryen/ic-wasm single executable → produce stage → closed-writer admission → publication | fs `durable::write_validated_with` | Trusted archive pin, version/environment, exact mode, original/cleanup errors, bundles |
| Canic final-artifact gate / metrics / transform comparison → Wasm facts → report | artifacts `wasm::inspect`, tools `install_limits` | Transform invariants, public Candid, warning policy, metric/schema stability, actual installability |
| Toko optimizer script and ic-wasm wrapper → JS metrics → text/JSON output and exit | Same facts/report, exposed through proposed Canic #481 command | Raw regression allowance, toolchain pins, optimization and compression choices, CLI schema |
| Query confined cache → opened descriptor → shared bounded I/O/publication → schema | fs/artifacts primitives already used | Capability confinement, alias checks, corruption admission, schema and replay |
| Backup admitted file/tree → shared byte identities → framed directory digest/barriers | Host byte mechanics and Backup's directory contract | Descriptor traversal, private modes, crash barriers, journal recovery |

State-space reduction sought: one owned-child implementation, one single-file
publication engine, and one Wasm decoder/reference comparison. Separate entry
surfaces and consumer error projections remain. No performance or code-size
improvement is claimed from line counts.

## Prioritized findings

1. **MEDIUM — Toko duplicates Wasm parsing and outcome decisions.**
   `bin/wasm-postprocess/wasm_metrics.js::readLeb` and `wasmMetrics` independently
   parse framing/imports; `main` returns JSON before applying enforcement. Both
   maintained `enforce_project_instance_limits` shell functions invoke the script
   in text mode. A valid module with 50,000 definitions and one import is refused
   as 50,001; JSON mode with enforcement returns success on that same observation.
   A global preceding a function import loses the import in the count. An 11-byte
   malformed module prints OK. [Captured output](toko-results.log) and the
   [shared-source probe](host-results.log) establish these bounded defects.
   Classification: duplicate flow/policy rediscovery. Disposition: consolidate
   through Canic's shared-report command, then delete the JS decoder. The command
   is not implemented in reviewed Canic. Preserve Toko's budgets/report contract;
   do not silently change historical `function_count` or full-payload metric
   meanings. Owners: [Toko #1791 evidence and concrete caller route](https://github.com/dragginzgame/toko/issues/1791#issuecomment-6055355350)
   and [Canic #481 prerequisite](https://github.com/dragginzgame/canic/issues/481#issuecomment-6055364940).
   Host inspection is structural, not instruction/type validation or proof of
   complete IC installability. This is a bounded build/report defect, not evidence
   of a successful invalid deployment.

2. **MEDIUM — Canic retains replaceable publication and limit mechanics.**
   The [prepared adoption patch](https://github.com/dragginzgame/canic/issues/458#issuecomment-6044765060)
   still applies to current committed selected source. It uses closed-writer
   admission, numeric gzip and a single limit report projection. Current Canic
   has the corrected 12 MiB constant, but compares full code-section payload,
   including its count prefix; the shared report compares code-body bytes.
   Classification: duplicate flow/late convergence. Disposition: apply within
   the consumer minor, preserve typed failure evidence and metric meanings.
   Keep `stage_executable`, also used by `stage_bundle_member`; a single-file
   primitive does not replace multi-file bin/lib publication. This turn ran
   applicability checks only, not the historical 35-test rehearsal again.
   Owner: [Canic #458 refreshed handoff](https://github.com/dragginzgame/canic/issues/458#issuecomment-6055356118).

3. **MEDIUM — Testkit retains an equivalent child cleanup owner.**
   `pic/startup.rs` is unchanged and the previously prepared OwnedChild patch
   still applies. It retires the private child guard/group-kill/reap implementation
   while retaining lifecycle policy. Classification: duplicate flow. Disposition:
   adopt in Testkit's consumer minor, then qualify its own native integration.
   The prior 16 startup + 7 runner test rehearsal is historical Linux evidence;
   no real PocketIC startup or new consumer-native run occurred here. This also
   unblocks dependency convergence: Blob's dirty lock has artifacts/fs 0.4.6
   through Testkit 0.21.3 and direct 0.5.0. Do not override incompatible minors
   merely to hide the duplicate graph. Owner: [Testkit #25 updated evidence](https://github.com/dragginzgame/ic-testkit/issues/25#issuecomment-6055355777).

## Intentional retention and limits

- Testkit `InputHasher::file_field` hashes domain/label/length framing directly
  into a retained state, detects changed length and reuses a size-sensitive
  buffer. `docs/fixture-reuse-benchmark.md` records Linux buffer-cost measurements.
  `hash_reader` computes an independent raw digest with different buffering;
  replacing it would alter the persisted identity or add a second hash. Keep
  the consumer format and measured specialization. No new shared hash framework.
- Query compact JSON hashing already uses sha2's canonical writer; wrapping it
  solely to route through Host would not remove a second algorithm. Confined
  publication and output-alias checks protect independent authority boundaries.
- Backup directory framing is product-owned and already exposes its reusable
  contract; Canic adoption belongs to Canic #490. Private crash barriers and
  descriptor custody cannot be deleted in favor of ordinary path replacement.
- Blob's `body.part` retention spans asynchronous transfer, integrity admission,
  failure receipts and no-replacement publication. A synchronous callback writer
  does not replace that lifetime. Single-file JSON/record writes already use Host.
- IcyDB CLI/artifact helpers and Memory repository tooling already use Host byte,
  path and executable owners. IcyDB domain fingerprints and Toko Miner network
  lifecycle policy are not neutral artifact mechanisms.
- Query piped capture/cancellation and Canic inherited operation-lock custody
  remain separate requirements under Host #5. Owned pipes do not establish
  descendant exit, inherited-lock lifetime or paid-effect recovery.

## Verification and artifacts

Remote main and annotated v0.5.0 resolve to Host
`db637fac8b7a9ef62301e1d9009ffeb5ffcd0be7`.
[Exact release CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37744999108)
passes Linux x86-64, Rust 1.88, macOS 15 Intel and Apple Silicon. The draft PR #21
is independently still open. Host #10/#14/#20 were already closed by concurrent
consumer verification when this audit reconciled them; release evidence was
added. This audit does not independently establish registry publication.

Local host: Linux x86-64, Node 24.21.0, Rust/Cargo 1.99.0. Fixture commands:
copy [fixtures.cjs](fixtures.cjs) into a private directory and run it with Node;
invoke Toko's script once with `--enforce` and once with `--json --enforce` for
all three generated files. Exit status pairs are respectively **1/0, 0/0, 0/0**.
Node's native `WebAssembly.validate` independently accepts the first two fixtures
and rejects the third. This controls for accidentally malformed positive inputs;
it is not qualification against an IC replica or every supported host.
The first generator draft collided with Node's `module` binding; a corrected
subprocess-capture attempt hit sandbox EPERM. Neither is used as product evidence.
Final direct shell invocations supply the recorded output/statuses.

For the independent library check, place [host-probe.rs](host-probe.rs) as
`probe/src/main.rs`, [probe-Cargo.toml](probe-Cargo.toml) as `probe/Cargo.toml`, and
[probe-Cargo.lock](probe-Cargo.lock) as `probe/Cargo.lock`, alongside the fixture
directory. The manifest records exact local source paths at the audited Host
commit; adjust location only if reproducing elsewhere. No registry dependency
substitute for these owners was used. Run from `probe/`:

```sh
cargo fetch --locked --offline
cargo run --locked --offline
```

All three fixture assertions pass. The actual initial lock was generated offline
before explicit locked/offline cache preparation. Selected dependencies include
wasmparser 0.261.0 and the exact lock's sha2/serde selections. Build files and failed
attempts remain at `/tmp/ic-host-050-audit.c6TeTV`; sibling targets were unused.
The current repository had no active Cargo/rustc owner before edits/compilation.
No full local CI/release gate, dependency download or native consumer execution
ran. Documentation links and whitespace checks pass.

This audit removes **no function, method or type**, and makes no production,
manifest or dependency changes. The proposed deletions above are consumer
handoffs, not edits applied to sibling repositories. No commit, push, release
or publication was performed.
