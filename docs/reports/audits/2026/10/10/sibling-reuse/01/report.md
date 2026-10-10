# Sibling Host reuse audit

Method: [flow convergence and duplication](../../../../../../../../audits/flow-convergence-and-duplication.md),
Shared baseline `b2646cde9abbc8861857a4379c683a0c19eba43e`, Host AGENTS.md overlay.
User requested read-only sibling inspection after Host 0.10.1 delivery. Host HEAD
is `c7bdc3d4e1c658957202eebd76bff2c51e22f645`. Its incoming dirty Cargo.lock
updates syn 3.0.6 to 3.0.7 and is preserved (SHA-256
`83c0a9b586a94676c54c171c5d945342962331e2e03d28f005ce473bc3245686`).

Verdict: **PASS WITH FINDINGS** for the bounded reuse review. Existing Host APIs
cover the identified mechanical gaps; no new library surface is justified.
This is source inspection and one isolated read-boundary probe, not consumer
compilation, a whole-system correctness audit or a performance measurement.

## Inspected identities and scope

| Repository | HEAD | Working state |
| --- | --- | --- |
| Canic | `ac55e50334dd6479ec36f404e89e60bcfe9184d6` | Dirty adoption and product edits |
| Testkit | `48cfff270d5e228c0e09d6ee452786fb14c9a66e` | Dirty tooling/lock edits |
| Query | `52e924412562273e19cc29e2f92aff556391e86b` | Dirty tooling/lock edits |
| Backup | `f1bba34b667a724274a65ac9653b4de88f30414f` | Dirty tooling/product/lock edits |
| Memory | `9137192d4b3425a229fa21df5f343883f56d80e8` | Dirty tooling/product/lock edits |
| Blob Storage | `9a808cdf3cee2dd7153e50dd6fa10592f72f6137` | Dirty lock |
| Auth | `edafe5b7414857dc5c8f6b19096234972945a30b` | Clean at inventory |
| Toko Miner | `cb9031b13ad7f685e51237828ed0fc915fe97910` | Dirty dependency/product edits |
| IcyDB | `76dc93ead6b66c8db9cb5c402355867dbdf188e3` | Dirty runtime/lock edits |

Selected manifest and Rust source scans covered host filesystem, hashing, gzip,
bounded reads, process capture and publication. Followed concrete candidates
through CLI persistence, config interpretation, diagnostic exports and error
projection. Runtime canister hashing, application schemas, paid-effect replay,
backup journals and capability policies were examined only to exclude them from
generic mechanical extraction. No exhaustive runtime audit is implied.

## Owner and flow trace

| Behavior | Mechanical owner | Consumer entry and retained policy |
| --- | --- | --- |
| Durable single-file publication | Host fs durable engine | Canic pending reservation/completion -> local JSON encoder -> currently duplicated staging/sync/rename; keep log identity and paid-operation order local |
| Bounded regular-file read | Host fs read -> artifact bounded reader | Canic icp.yaml -> local YAML parse -> gateway/network/report policy; keep 1 MiB allowance and missing/symlink policy local |
| Complete create-only export | Host fs descriptor publication | IcyDB diagnostic validate/encode/budget -> currently direct final-file write; keep schema, provenance, mode and absent-parent policy local |
| Typed publication error | Host NamedWriteError -> consumer domain boundary | Testkit/Memory/Auth/Query/Backup/Blob use current contract; Canic still has retired calls alongside dirty 0.10 requirements |
| Process lifetime and bounded capture | Host process | Toko capacity encoder uses capture_group_command; Testkit startup retains simulator protocol and readiness ownership |

## Findings

1. **MEDIUM — incomplete Canic 0.10 adoption.** Root requirements select ^0.10
   while fleet-package lock and canic-backup JSON still import write_typed_with;
   artifact_io retains a two-argument write_with. Both fs dependencies inherit
   the root selection. This is a source-level mismatch, not a new compilation
   result. Disposition: finish existing migration under
   [Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6095149839).

2. **MEDIUM — Canic config reader silently truncates.** read_optional_icp_yaml
   applies take(1 MiB + 1) but returns success without checking the bound.
   The unchanged function body in a std-only harness returned 1,048,577 bytes
   from a 1,048,635-byte file, omitting trailing configuration. The harness
   aliases only IcpConfigError to io::Error; no Canic parser/build was executed.
   Use an existing bounded Host reader, preserving explicit admission policy.
   [Canic #509](https://github.com/dragginzgame/canic/issues/509) owns the defect.

3. **MEDIUM — Canic pending journal duplicates persistence mechanics.**
   write_pending_operation_log, pending_temp_path, sync_directory and the lock
   acquisition body repeat Host operations. Delegate publication and regular
   locking while retaining the full read/modify/write guard and typed failures.
   Parent precreation must move with the durable lock operation; leaving an
   unsynced create_dir_all first would bypass the intended parent guarantees.
   Existing replay policy is a separate domain concern. Disposition recorded in
   [Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6095149839).

4. **LOW — IcyDB diagnostic export can expose partial output.** write_new uses
   exclusive final-file creation followed by write_all. Atomic visibility is
   not an existing documented promise; improving it can reuse write_at_with
   CreateNew while preserving absent-parent refusal and existing permissions.
   No new Host API is needed. Proposed under
   [IcyDB #307](https://github.com/dragginzgame/icydb/issues/307#issuecomment-6095149967).

## Retained separation

Query's confined cache publication already uses write_at_with; its explicit
export writer additionally owns path and inode alias refusal before truncation.
Replacing that with pathname replacement would change its identity semantics.
Backup's directory checksums frame relative paths/digests and its publication
owns multi-file crash barriers: these are not ordinary file hashes or writes.
Testkit's port reader has pending-until-newline and partial-readiness semantics,
and its bundle/cache directory renames commit multiple artifacts. Retain them.
Domain-framed hashes in Auth/IcyDB are not duplicate generic artifact identities.
Memory's release helper and Blob's terminal redacted failure boundary already
retain their required recovery behavior through current Host APIs.

## Evidence and next action

Raw scans and the reader probe are under `/tmp/ic-host-sibling-audit-20261010/`
and `/tmp/ic-host-siblings-101-scan.txt`. Selected Canic pending.rs SHA-256 is
`4580a06bdc9810773c6724b43f09e19b92d836d3f3e247ed743c47cc1e82e860` (matches HEAD);
icp_config/mod.rs is
`42eb4baf972d913c85f0a5ff2fd1b44fbb02d0494b31ac1a36bfe9fe19f479e0` (dirty import
only); IcyDB diagnostic artifact.rs is
`e639f8b467b2fc9cdcc3de30dc5b73f8055299960a818cf3a4be0a42ce0305d0` (matches HEAD).

All nine root manifests select Host 0.10; that alone does not prove adoption.
Testkit #44, Query #37 and Backup #35 are closed; Canic #458, Memory #46, Blob #46
and IcyDB #307 retain their own acceptance scope. No sibling checks or source
mutations were performed; active builds there were not disturbed.

Host's [exact 0.10.1 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37964131536)
now passes Linux, MSRV, Intel macOS 15 and Apple Silicon macOS 15. Closed Host #43
on that delivery/native evidence. This result belongs to the released lock/source,
not the incoming dirty syn update. Recommend the Canic bounded-read repair and
completion of existing adoption first, then the persistence consolidation.
No Host code, manifests or release notes changed; no new release is needed for
the audit. No functions, methods or types were removed.
