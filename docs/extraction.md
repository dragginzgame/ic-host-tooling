# Crate ownership and extraction contract

The initial library source is committed IC Host Tools 0.2.0 at
be7d73908225e73ab5babf8fbb9fe959f40d439b. Pending sibling tooling edits
were excluded. Canic's clean durable module and its tests are bound to their
observed committed bytes in [source provenance](../ci/extraction-sources.json).

With the default `candid-extraction` feature, dependencies point down from
ic-host-tools to ic-host-process and ic-host-fs,
from ic-host-process to ic-host-fs, and from each to ic-host-artifacts.
No generic crate depends on ic-host-tools, Canic, IcyDB or an IC runtime.
Test/example artifact features do not become production filesystem dependencies.
Disabling the tools defaults retains response decoding while excluding all three
generic extraction dependencies from its normal graph. This is an additive
feature choice; default callers retain the existing Unix extraction surface.
The optional `ic-limits` profile adds only generic artifact inspection to the
response-only graph; IC reference values and comparison remain in the tools owner.

- ic-host-artifacts owns stream identities, bounded read/write/copy mechanics,
  optional gzip encoding/decoding, verified tar member bytes and optional Wasm structural facts.
- ic-host-fs owns pathname/descriptor reads and the durable module's replace,
  create-new, private-file, parent synchronization and descriptor-lock operations.
- ic-host-process owns executable resolution/admission, explicit execution context,
  output limits, deadlines, direct-child/explicit group cleanup evidence and separate Git queries.
- ic-host-tools owns Candid text/extractor interpretation, ICP response formats
  and explicit revision-bound IC resource comparisons.

Source tests move with their owner; public artifact projection tests belong to
ic-host-artifacts. Runnable examples retain explicit dependencies on each owner.
No generic compatibility reexports remain in ic-host-tools.

Within `ic-host-fs`, `read` is the sole owner of required/optional/private reads.
Following paths, final-component no-follow admission and already-open descriptor
reads remain distinct contracts. Optional reads reuse the same bounded stream
engine and return `ArtifactError`; private reads add typed permission, link-count
and fixed-length rejection. Durable publication and locks do not own a second
reader hierarchy. Lock errors retain typed admission and original I/O causes.
The [0.4 notes](changelog/0.4.md) describe the required consumer changes.

`durable::write_named_with` exposes an absolute owned staging pathname to external
producers through the existing publication engine. Filesystem code owns exclusive
creation, descriptor lifetime, identity checks, cleanup and sync/rename outcomes.
The callback owns executable admission, process completion, bounded output
validation and any domain interpretation. Its original error and cleanup evidence
remain distinct, as does a final sync failure after publication. Parent hierarchy
control and exclusion of concurrent namespace mutation are caller prerequisites;
this is neither arbitrary-file adoption nor a child sandbox.

`durable::write_validated_with` adds a closed-writer admission phase to that same
engine. Read-only descriptor custody preserves inode identity while permitting
prepublication executable probes. Consumers retain tool versions, distribution
hashes, probe arguments/environment and bundle layout. All writable clones and
probe processes must finish before publication; read-only custody does not
prevent an external writer from changing bytes.

The first import preserves implemented guarantees. It does not establish root
confinement for following pathname APIs, immutable source bytes, process sandboxing,
descendant cleanup, hardware-independent crash durability or safe paid-effect retries.
Query's capability confinement and refresh leases remain consumer-owned.
Backup's tree checksums, multi-file publication and journals remain consumer-owned.
Further adoption must retain each caller's independently established guarantees.

`ic_host_process::tool::capture_command` reuses that same execution engine with a
caller-configured `std::process::Command`. The caller's executable/version policy,
arguments, cwd, environment and platform setup are retained; stdin becomes null
and stdout/stderr are bounded pipes. Unlike the exact-byte admission performed by
`AdmittedTool`, this entry performs no executable identity or version checks.
Ambient inheritance and PATH resolution follow the supplied command. This avoids
reconstructing commands and dropping consumer-owned setup. Neither entry owns
process groups, retries or paid-operation recovery. `ToolError::evidence` and
`execution_error` borrow the existing capture and failure objects for consumer
error projection; no parallel error model or automatic diagnostic output is added.

`ic_host_process::child::OwnedChild` shares child spawn/reap/cleanup mechanics
with the direct-child capture engine. Its public spawn establishes a new group
while preserving caller IO and command settings. It reserves an exited leader
until group signalling finishes, then reaps it. The caller drives polling,
natural waiting or cancellation; Drop provides best-effort cleanup, including
unwinding. Typed cleanup evidence remains separate from the original operation
failure. No application readiness, global signal handler, lifecycle controller,
retry, descendant-exit barrier or process-tree containment is introduced.
Exclusive reaping and an unchanged child group are required. Synchronous cleanup
has no wall-clock bound and only reaps the direct child.

The sole-zombie macOS check uses one private fixed-buffer libproc call, explicitly
approved by the maintainer. Unsafe code remains denied outside that function.
All other process syscalls use safe rustix APIs; the existing selected libc
version becomes a macOS-only direct dependency, without a bindgen toolchain.

`AdmittedTool::admit_version` shares the exact admission engine for tools whose
installation the caller already trusts. It takes a `VersionSpec`, records the
bounded executable identity and admits exact version output; later runs enforce
that observed digest. It removes consumer-side hash/admit composition without
claiming that observed bytes are a trusted published pin. Existing `ToolSpec`
admission still compares its supplied digest before executing the version probe.
Installation provenance, per-host pins and selection of either contract remain
consumer policy; no host catalog or version-range resolver is introduced.

Additional reusable mechanics are extracted from committed sibling source:

- `artifact::hash_gzip` and `hash_gzip_or_raw` reuse bounded hashing over the
  existing single-member gzip decoder. `gzip_matches` reuses `MatchingWriter`
  to compare exact decoded bytes without retaining another payload copy. Input
  admission, CRC/length and trailing-data checks have one owner shared with
  `decode_gzip`. Complete compressed/raw input is still a caller-owned slice;
  Wasm admission, representation selection and deployment policy stay local.
- `ic_host_artifacts::artifact::chunk_digests` computes ordered fixed-size chunk
  digests and the complete raw input identity through the existing bounded read
  traversal. Callers select a nonzero chunk size, input-byte allowance and maximum
  digest count; compression interpretation and upload policy stay local.
  `CopyError` can project into `io::Error`, preserving native input/output causes
  and retaining non-I/O copy failures as typed causes at publication boundaries.
- `ic_host_artifacts::artifact::HashingWriter` hashes accepted writes through the
  existing bounded writer, returning the original sink and raw size/digest. It
  needs no complete serialized buffer. Short writes count and hash only accepted
  bytes; a failed producer yields at most an observed prefix, never a completion
  receipt. Encoding, producer success, flush/sync and publication stay local.
- `ic_host_artifacts::artifact::MatchingWriter` compares a produced byte stream
  with borrowed expected bytes. It keeps consuming after a mismatch so serializer
  errors remain visible; callers own successful completion and canonical ordering.
- `ic_host_artifacts::artifact::encode_gzip` writes one zero-timestamp gzip member
  through a bounded caller-owned sink. Callers select compression, source admission,
  compressed byte budgets and publication; output compatibility is bound to the
  selected compression backend rather than promised across dependency upgrades.
- `ic_host_fs::durable::write_with` accepts a streaming producer and returns its
  result after the existing durable replacement sequence completes. Byte-based
  writes share that same commit engine. A returned producer error preserves the
  old destination and attempts to remove only owned staging. Panics or process
  interruption can retain staging. Post-rename sync failure still needs caller
  reconciliation.

- `ic_host_fs::path::canonicalize_allow_missing` resolves existing symlinks and
  normalizes missing suffixes against an explicit absolute base for relative
  paths, including dangling symlink targets. It resumes existing-component
  resolution after parent traversal and retains native directory requirements.
  `canonicalize_allow_missing_with_symlink_limit` additionally accepts the
  consumer's limit on simultaneously active missing-target expansions. Neither
  entry confines a root or creates files. Hard-link identity, opening and
  concurrent pathname revalidation remain consumer-owned.
- `ic_host_fs::durable::lock_exclusive_with_wait` acquires a caller-owned regular
  descriptor with explicit polling and contention observations, retrying
  interrupted acquisition. The existing pathname progress helper delegates to
  it with its one-second report cadence. Opening, deadlines, lock namespace,
  descriptor clones and final-owner unlock remain consumer-owned.

The exact source revisions, file digests and selected mechanics are recorded in
[source provenance](../ci/extraction-sources.json). These APIs add no JSON schema,
cache policy, root confinement, tool pin or process-tree ownership. Backup's private
parent/file modes and crash barriers are stronger than ordinary durable replacement;
they must not be replaced by `write_with` without preserving those contracts.

The maintainer authorized retiring the original local checkout after verification
of a full backup. Existing consumers retain their prior dependency selections;
adoption of these split crates remains separate. The manifests permit crates.io
publication through the [explicit publication procedure](publishing.md).
The breaking removal of generic APIs from ic-host-tools requires a coordinated
pre-1.0 minor release. This contract assigns ownership; GitHub issues own follow-up tracking.
