# Sibling host-code extraction evidence

Review date: 2026-10-06. Trigger: maintainer request to extract reusable sibling
code into IC Host Tooling and give affected repositories adoption issues.
Method: `audits/flow-convergence-and-duplication.md` from the exact Shared Tooling
snapshot `21f3ec3dd97f2968c9f0b08924451bb2f71770d1`. Local authority is AGENTS.md
and docs/extraction.md: four host crate boundaries; siblings remain read-only.
IC Host Tooling source: `efd402e0063ccbbf8a143cc970be52ab41b1766d`, with the
previous uncommitted license metadata correction and this pending 0.3.1 extraction.

Verdict: **PASS WITH FINDINGS** for the scoped extraction. Shared primitives
are implemented and locally qualified; consumer convergence remains tracked in
the owning GitHub issues. No consumer execution, deployment, full CI or native
macOS result is inferred. This is an evidence record, not a parallel issue queue.

## Scope and identity

Discovery inspected host primitive/API references across Canic, IC Backup,
IC Blob Storage, IC Memory, IC Metrics, IC Query, IC Testkit, IC Timers, ICHelper,
IcyDB, Shared Tooling, Toko Miner and Toko Miner Assets. Detailed traces followed
the eight existing consumers of the old host monolith. Searches were discovery
aids; similar hashes and filesystem calls were not treated as equivalent contracts.
Canister runtime/storage models, gameplay hashes, application wire formats,
frontend tooling and canonical Shared Tooling snapshot copies were excluded from
extraction into host libraries. This is not a complete audit of those families.

| Consumer | Reviewed HEAD | Working-tree distinction |
| --- | --- | --- |
| Canic | d815abfc661d791ecf72afc5b1e4b6a990f37a91 | Broad unrelated lifecycle/tooling edits; selected extraction files match HEAD |
| IC Testkit | 827157434eb8b2d6c13c4b8e47493bd6a38678b9 | Cache/tooling/tests dirty; selected digest source matches HEAD |
| IC Query | 70f02f0709f157ecdd9f6fc35e0cb84411e8b391 | Tooling/dependency edits; selected JSON writer matches HEAD |
| IC Backup | 04da09a5a919bbf9aa56e71eaacaf245107021d4 | Documentation dirty; selected source committed |
| IC Memory | d56af42b0bd9ac5a794de2248e31b335945a1e1c | Clean |
| IcyDB | 049e561a843a8d3f4526460fd15876a4a238df10 | Runtime schema/tooling edits; selected host callers committed |
| IC Blob Storage | 7eed7397d49b06f1cb44699fdf2d4feb91f5a2d8 | Cargo.lock dirty; selected CLI source committed |
| Toko Miner | 9071b4cc9c1cf0d6d905f73592154b5108ab2147 | Clean |

Exact digests for the three extracted source selections are in
`ci/extraction-sources.json`. Each working file's SHA-256 was compared with the
corresponding `git show HEAD:path` bytes. No dirty sibling library source was
silently imported, and no sibling files or artifacts were changed.

## Owner and flow traces

| Behavior and current entrypoints | Shared convergence | Inputs/result | Retained consumer authority |
| --- | --- | --- | --- |
| Query canonical_json_matches -> private MatchingWriter -> serializer writes | ic-host-artifacts::artifact::MatchingWriter | Borrowed expected bytes; sticky comparison; complete-match observation after producer success | JSON encoding/order, schemas, hash domains, errors, budgets and confined publication |
| Canic deterministic_gzip_bytes -> GzBuilder zero mtime -> encoded bytes -> artifact publication | ic-host-artifacts::artifact::encode_gzip | Caller-held input, compression, sink and compressed-byte budget; header/trailer included | Source admission, backend selection, best compression policy, Wasm transforms and artifact-set publication |
| Testkit write_atomic/copy_file_atomic -> write_file_atomic -> staging/write/sync/rename | ic-host-fs::durable::write_with, sharing the existing byte-write commit engine | Caller producer; result only after file/parent sync and complete replacement | Cache framing/keys, input graph, destination ownership, locks, pruning and retained entries |
| Memory repo_tool::write_atomic -> staging/write/sync/rename | Existing ic-host-fs::durable::write_bytes | Complete caller-selected bytes; durable replacement | Release identity, receipt/schema selection, Git effects and interruption reconciliation |
| Backup sha256_hex -> generic hex_bytes -> ArtifactChecksumRecord | Existing ic-host-artifacts::artifact::Sha256Digest | Raw bytes or exact SHA-256; lowercase display | Tree checksum framing, arbitrary-byte wire hex, journals and topology |
| IcyDB diagnostic artifact read -> metadata size/take/read_to_end -> JSON/provenance validation | Existing ic-host-fs::read after explicit caller admission | Regular descriptor and caller byte allowance | Diagnostic schema, ownership, final-symlink policy, error projection and validation |
| Blob CLI and Toko qualification old artifact file imports -> existing bounded reads | Existing ic-host-fs::read and ic-host-artifacts::artifact | Admitted file/descriptor, exact caller budgets and raw identity | Upload/lifecycle authority, browser fixtures, credentials, ingress limits and recovery |

The extraction introduces no JSON production dependency, feature expansion,
tool pin, cache model, lifecycle state or compatibility reexport. The gzip
encoder uses the existing optional compression dependency. MatchingWriter
retains the entire serializer run after a mismatch rather than hiding failures.
The filesystem change generalizes the producer within the existing descriptor-
relative commit sequence rather than adding another production publication engine.

## Findings and deliberate separation

The three extracted local mechanic owners are LOW maintenance findings: the
library now owns reusable comparison, gzip output and streamed staging semantics.
The old dependency/import ownership is a LOW coordinated adoption finding across
eight consumers; existing delegation is retained rather than falsely described
as duplicate implementation. There is no demonstrated runtime defect inferred
solely from a matching function name.

Existing issues were searched before feedback. The concrete dispositions and
qualification instructions are owned by:

- [Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6021451439): update existing host adoption evidence; replace durable/gzip mechanics and select split owners.
- [IC Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13): retire the local atomic artifact writer while preserving cache custody and error projections.
- [IC Query #11](https://github.com/dragginzgame/ic-query/issues/11): replace the private comparison sink, preserving the serialization and confinement owner.
- [IC Backup #11](https://github.com/dragginzgame/ic-backup/issues/11), [IC Memory #16](https://github.com/dragginzgame/ic-memory/issues/16), [IcyDB #307](https://github.com/dragginzgame/icydb/issues/307), [IC Blob Storage #14](https://github.com/dragginzgame/ic-blob-storage/issues/14), [Toko Miner #30](https://github.com/dragginzgame/toko-miner/issues/30): adopt existing split owners and remove only semantically redundant local mechanics.

Query's capability-confined roots, file/parent permissions and refresh leases
remain protective separation. Backup's 0700 parent hierarchy, 0600 staging,
descriptor-relative tree walks, crash barriers and multi-file journals cannot
be replaced by ordinary shared durable writes. Testkit's framed tree digests,
exclusions, cached input roots, retention/pruning and PocketIC process groups
remain local. Canic ICP inherited effect-lock descriptors, trusted executable
authority gaps, public error payloads, release-set ordering and paid recovery
are not solved by these primitives. IcyDB persisted database fingerprints and
game/runtime digest domains are not raw host artifact identity APIs.

No existing functions, methods or types were removed from this workspace in
this batch. The source functions in sibling repositories remain until their
owners adopt the shared APIs; implementation here does not prove global code
removal. New APIs require the pending compatible 0.3.1 release; existing split
0.3.0 primitives can be adopted independently. No manifest version was changed.

## Qualification

Native Linux x86-64, Cargo/Rust 1.99.0, locked/offline dependency selection:

- `cargo fetch --locked --offline`: prepared the selected cache successfully.
- `cargo test -p ic-host-artifacts --all-features --lib --locked --offline`: 46 passed, including byte comparison, repeatable gzip format, output budgets and sink failures.
- `cargo test -p ic-host-fs --all-features --lib --locked --offline`: 25 passed, including streamed producer results, partial failure cleanup, bounded copy and the existing publication/sync barriers.
- `make test-artifacts-minimal`: 21 passed without optional features.
- Strict all-target/all-feature Clippy and Rustdoc with warnings denied passed for the two changed packages.
- Rust 1.88.0 all-target/all-feature locked/offline checks passed for those packages.
- Formatting, dependency declarations/inheritance, documentation links and diff whitespace checks passed. Cargo.lock selections and tool pins were unchanged.

The first Clippy attempt failed because test fixture items followed statements;
their declarations were moved before statements, and strict qualification passed.
This correction does not relabel the earlier failed attempt. Test evidence covers
local source, not sibling builds, actual publication or installation. Native
macOS 15 Intel/Apple Silicon and consumer-specific qualification remain separate.
No broad CI/release gate, commit, tag, push or upload occurred.
