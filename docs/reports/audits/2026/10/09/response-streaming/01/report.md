# Response and stream ownership audit

## Scope and evidence

Maintainer-requested audit and next-version repair on released Host 0.9.5,
`0f61811c88a6b6b4b026b04ca16e428409be877f`. The working tree was clean at
entry. Methods: `audits/code-hygiene.md` and
`audits/flow-convergence-and-duplication.md`, adopted from Shared Tooling
`47d6ae6488b8007323fa7c2e22a6efa11d77ae63` (0.2.7). Both method files match
the latest sibling bytes. Local authorities are AGENTS.md and docs/extraction.md;
their exact digests are in [overlay.sha256](overlay.sha256).

Review traces response decoding, Candid normalization, bounded read/write/hash,
chunking, gzip framing and selected archive/Wasm boundaries. Spot checks of
executable resolution, Git observation and publication adapters establish why
those contracts remain separate. This is not a new process-lifecycle, filesystem
race, native-host or performance qualification. Unchanged paths received source
review; only the changed response module received new Rust execution evidence.

Shared remote main remains 0.2.7. Its working tree contains uncommitted Make
admission corrections; none were adopted. Host retains its released local-root
binding and exact 88-file snapshot. No new shared helper or dependency is needed.

## Owner and consumer trace

| Behavior | Owner and flow | Disposition |
| --- | --- | --- |
| Response bytes | `response::decode` selects JSON/hex/label admission, then `decode_hex` validates and allocates | Consolidate text-empty admission into the digit pass |
| Candid normalization | `candid::normalize` measures normalized text before fallible allocation | Retain both passes; they establish the allocation bound |
| Stream identities | `visit_reader` drives read/hash/chunk/copy; writers account only accepted bytes | Retain one traversal owner; no parallel engine found |
| Gzip | `with_decoder` owns input admission and complete single-member framing | Retain decoder integrity and trailing-data checks |
| Archive and Wasm | Archive admits compressed/member identities; Wasm exposes structural facts | Retain independent trust boundaries and consumer validation |
| Consumer publication | Query owns confinement; Backup owns private parents and crash barriers | Retain stronger local guarantees around shared mechanics |

Read-only consumer observations used these HEADs, with selected source digests
in [consumer-inputs.sha256](consumer-inputs.sha256). Paths there resolve from
Host's root. Siblings were active worktrees: all five had dirty lockfiles,
and Canic also had a dirty root manifest. These are observed source identities,
not claims of committed dependency selection or executed consumer acceptance.

| Consumer | Observed HEAD | Relevant usage |
| --- | --- | --- |
| Canic | `ac55e50334dd6479ec36f404e89e60bcfe9184d6` | JSON envelope delegates to Host; Candid interpretation remains local |
| IcyDB | `63ac8cbf9337306187e8bf74309ed9c576aca2af` | Hex delegates to Host after local Unicode whitespace/label normalization |
| Query | `d1dc30ba8b9074d276d8e409a06850ce7aa85245` | Bounded/matching writers around caller-owned serialization and confinement |
| Backup | `d9517a42d685b240933affd58b211364823d5122` | Bounded size checks and shared publication with stronger parent/barrier policy |
| Testkit | `1a8f2ff570ea1c3bd58b215e28af52e5d99870e8` | Shared gzip/hash/verification with consumer provisioning policy |

The final digest recheck detected concurrent Canic edits: its error DTO import
moved from `canic_core` to `canic_contracts` while HEAD stayed unchanged. Re-read
the module; the Host response call and limits are unchanged. Preserve both the
initial digest and [canic-recheck.sha256](canic-recheck.sha256), rather than
claiming that the consumer stayed fixed. The other four selected digests match.

IcyDB's prechecks preserve its existing error ordering and presentation; deleting
them merely for uniformity is not justified. Query's direct dependency hasher
already implements its unbounded hashing contract. Replacing it with another
wrapper would not remove an independently maintained engine. Backup's explicit
prepublication sync supports its acknowledged crash barrier and is not redundant
with the later shared sync. No sibling edit or new consumer API is proposed.

## Finding and implementation

**LOW — late convergence of text-hex admission.** The private
`response::decode_text_hex` scanned for whitespace-only input before `decode_hex`
scanned the same input to validate and count digits. The count already determines
whether an admitted text body is empty. Removed that wrapper and scan; the
existing digit pass now rejects zero digits for text formats. JSON still permits
an empty string. Invalid bytes still fail before odd-length and decoded-limit
checks. Existing format selection, labels, byte allowances and fallible decoded
allocation are unchanged. No speedup or allocation reduction is claimed.

Only `ic_host_tools::response::decode_text_hex` was removed; `decode_hex` owns
its check. No public function, method or type was removed. This is compatible
internal cleanup for pending 0.9.6, requiring no hard cut or consumer migration.
The finding is resolved locally; there is no new follow-up queue.

## Verification and limits

Linux x86-64, Rust/Cargo 1.99.0, existing locked dependencies, offline:

- `cargo test -p ic-host-tools --no-default-features --lib response::tests --locked --offline`:
  all ten existing tests pass before and after the change. They cover empty JSON,
  empty text, whitespace, invalid digits, odd lengths, bounds, labels, duplicate
  envelope fields and redacted errors.
- Selected library Clippy with `-D warnings` and response-file rustfmt pass.
- Selected documentation links, snapshot integrity and whitespace checks pass.
- Manifests, lockfile, tool pins, snapshot and real Git index/config are preserved.

Logs are retained in `/tmp/ic-host-096-audit/`. No new tests mirror the deleted
wrapper; the existing contract tests cover the changed behavior. No full gate,
consumer build, native macOS check, tool download, release, commit or push ran.

Verdict: **PASS for the selected source/response contract review after the local
cleanup**. This does not qualify untouched runtime boundaries or native hosts.
Released 0.9.5 [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37954233417)
has passing Linux/MSRV jobs and queued Intel/Apple Silicon macOS jobs at inspection;
that CI is not evidence for this dirty change. Existing qualification remains
under [#42](https://github.com/dragginzgame/ic-host-tooling/issues/42) and the older
adoption issues. No additional runtime defect was demonstrated in this scope.
