# Crate ownership and extraction contract

The initial library source is committed IC Host Tools 0.2.0 at
be7d73908225e73ab5babf8fbb9fe959f40d439b. Pending sibling tooling edits
were excluded. Canic's clean durable module and its tests are bound to their
observed committed bytes in [source provenance](../ci/extraction-sources.json).

With the explicit `candid-extraction` feature, dependencies point down from
ic-host-tools to ic-host-process and ic-host-fs,
from ic-host-process to ic-host-fs, and from each to ic-host-artifacts.
No generic crate depends on ic-host-tools, Canic, IcyDB or an IC runtime.
Test/example artifact features do not become production filesystem dependencies.
The empty tools defaults retain response decoding and pure Candid text
normalization while excluding all three generic extraction dependencies and the
optional Candid parser from its normal graph. The 0.15 hard cut moves extraction
to `candid::extraction` and normalization to `candid::text`, without reexports.
The optional `ic-interface` feature adds generic Wasm facts without the Candid
parser. `candid-service` adds parser-backed service inspection and implies
`ic-interface`. Neither adds filesystem/process dependencies unless extraction
is also selected.
The optional `ic-limits` profile adds only generic artifact inspection to the
response-only graph; IC reference values and comparison remain in the tools owner.

- ic-host-artifacts owns stream identities, bounded read/write/copy mechanics,
  optional gzip encoding/decoding, verified tar member bytes, optional Wasm
  structural facts and bounded custom-section framing.
- ic-host-fs owns pathname/descriptor reads and the durable module's replace,
  create-new, private-file, parent synchronization and descriptor-lock operations.
- ic-host-process owns executable resolution/admission, explicit execution context,
  output limits, deadlines, direct-child/explicit group cleanup evidence, native
  process-table/RSS observations and separate Git queries.
- ic-host-text owns pure terminal-safe escaping and Unicode display-width tables.
  Cells, alignments and formatting options are explicit; reports and CLIs retain
  headers, rows, column choices, size/truncation policy, output, terminal detection,
  colour and progress. It depends on no other Host or IC owner. This fifth generic
  boundary was explicitly selected by the maintainer under
  [#110 — Host text ownership](https://github.com/dragginzgame/ic-host-tooling/issues/110).
- ic-host-tools owns Candid text/extractor and service-method interpretation,
  IC method export projection over generic Wasm facts, ICP response formats
  and explicit revision-bound IC resource comparisons.

Host Text's `timestamp` module owns pure canonical UTC second spelling and calendar
conversion. It reads no clock, chooses no freshness window and authenticates no
collection time. Formatting covers `u64`; parsing admits zero through `i64::MAX`.

`sampling::snapshot(record_bytes, command_limits)` keeps native RSS observation
in the Process owner. Linux selects the per-record procfs allowance; macOS uses
caller-selected `OutputLimits` through the existing direct-child capture engine.
An optional deadline remains explicit, with no default elapsed-time bound.
Incomplete tables refuse before parsing; command failures retain typed capture,
diagnostic and cleanup evidence instead of printing stderr. Workload lifetime,
sampling cadence, process visibility interpretation and aggregate table memory
remain consumer-owned. See the [0.19 hard cut](changelog/0.19.md).
Linux requires each stat record to fit completely within its allowance before
parsing; bounded artifact-reader refusals retain their typed cause through
`SamplingError::Io`, while native disappearance/access skip rules stay intact.

The optional tools `certificate` feature owns strict CBOR decoding into owned
`ic-certification` certificate/delegation/hash-tree values. It admits complete
input bytes and selected CBOR nesting before returning a single fully consumed
value. Direct Serde visitors enforce actual array/byte kinds, exact tree tags
and arities and 32-byte pruned digests without a generic CBOR payload tree.
Certificate/delegation fields reject unknown, duplicate, missing or invalid
values. Errors distinguish byte allowance, trailing data and CBOR/shape failures.
Parser allocations use ordinary upstream Vec/Box storage; a byte bound is not a
heap/CPU quota. No authentication is implied. Agent verification, network trust
roots, combined certificate/witness budgets, witness commitment and application
leaf interpretation remain consumer-owned. No Agent, transport, filesystem,
process or text dependency enters this profile
([#109 — Owned certificate decoding](https://github.com/dragginzgame/ic-host-tooling/issues/109)).

Source tests move with their owner; public artifact projection tests belong to
ic-host-artifacts. Runnable examples retain explicit dependencies on each owner.
No generic compatibility reexports remain in ic-host-tools.

`wasm::encode_custom_section` encodes an isolated custom section from the existing
borrowed `CustomSection` view. Name and complete payload lengths must fit Wasm's
u32 domain; the complete encoded-byte allowance is checked before fallible
allocation. The optional Wasm feature reuses the existing locked `leb128` integer
writer; no filesystem/process or Candid dependency is added. Names, raw payloads,
metadata schemas, duplicate/append policy and aggregate module budgets stay with
callers. They assemble and validate the containing module. Independent malformed-
parser fixtures remain separate from encoder round-trip coverage.

`candid::service::inspect` uses the Candid parser for self-contained grammar and
type checking, returning method names and modes without expanding argument/return
types into a presentation tree. Imports are rejected rather than loaded or ignored.
Consumers select source-byte and returned-method limits; these are not a hard
heap, stack or CPU quota for the parser. `interface::exported_methods` borrows
function export facts without reparsing Wasm. It preserves internal methods,
empty names and mode differences for caller admission. Oneway remains distinct
in Candid and maps explicitly to the update export namespace through `as_export`.
Expected interfaces, CDK allowlists, type compatibility, payload annotations and
Canic endpoint presentation remain consumer policy. IC Build owns build/cache
orchestration separately; these libraries do not install tools or build canisters.

Within `ic-host-fs`, `read` is the sole owner of required/optional/private reads.
Its Unix `tree` module exposes a no-follow descriptor traversal for caller-owned
entry handling, but does not own Backup's tree checksum framing, multi-file
publication or retained byte custody. Callers admit and hold the root, select
depth/byte limits, and prevent concurrent namespace or content changes where
stable evidence is required.
Observed directories always use directory-only opens and descriptor type checks;
callers do not choose kernel flags. Native open errors may precede type callbacks
when an entry is replaced during traversal.
Following paths, final-component no-follow admission and already-open descriptor
reads remain distinct contracts. Optional reads reuse the same bounded stream
engine and return `ArtifactError`; private reads add typed permission, link-count
and fixed-length rejection. `hash_file_no_follow` combines the same Unix opener
and descriptor admission with the artifact owner's constant-memory hash traversal;
metadata and streamed bytes have independent bounds. It rejects final symlinks,
while existing `hash_file` continues to follow them. Ancestor confinement,
concurrent-writer custody and exact expected-length checks remain caller-owned.
Durable publication and locks do not own a second
reader hierarchy. Lock errors retain typed admission and original I/O causes;
durable creation errors retain `NamedWriteError` in `RegularFileLockError::Publication`.
`try_lock_regular_file_with_parents` adds one nonblocking acquisition attempt
through the same path opener; contention retains native `WouldBlock`. Blocking
callers retain their existing behavior. Observed waits compose the opener with
`lock_exclusive_with_wait` and caller-owned polling/reporting intervals. No filesystem latency
deadline, retry controller or stale-lock removal is introduced.
`open_regular_lock_file_with_parents` is that single admission owner, exposed
without acquisition so consumers can choose shared locks or descriptor waiting.
It preserves existing bytes and returns a close-on-exec read/write descriptor.
Consumers retain stable namespace, clone lifetime and explicit unlock policy;
opening does not establish root confinement or lock ownership.
Existing lock entries skip publication staging and synchronization; only missing
entries invoke durable creation. Final descriptor checks still enforce regular
no-follow admission, and namespace removal may return a native error without retry.
`open_private_regular_lock_file` reuses that descriptor admission for direct
owner-only sidecar creation in an existing parent, without staging or durability
syncs. It preserves existing bytes, inode and permissions. The caller owns lock
acquisition, namespace trust, descriptor inheritance and descendant custody;
adding this opener does not transfer command lifecycle or recovery to Host.
The [0.4 notes](changelog/0.4.md) describe the required consumer changes.

`durable::write_with(path, options, producer)` is the single generic pathname
writer. Byte helpers use it and return `NamedWriteError<io::Error>` rather than
discarding publication state or cleanup evidence. Consumers retain error projection
and recovery decisions; an after-publication failure requires reconciliation.
The [0.10 migration](changelog/0.10.md) replaces the old two-argument writer and
removes the `write_typed_with` name without a compatibility alias.

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
`ic-host-process` opt-in testing also supplies an acknowledged direct-child crash
barrier. It owns marker timing and direct-child reaping; consumers retain crash
point selection, fixture evidence and recovery assertions.
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
Completed invocation APIs return `CompletedExecution` with a required exit status,
including nonzero and signalled exits. `ExecutionEvidence` retains optional status
for failures. Strict callers use `require_success`; version, Git and Candid
admission apply it before interpreting output. A completion does not imply that
external effects succeeded or transfer a retained child. Nonzero/signalled leaders
always undergo the selected ownership's cleanup and reaping, even in retain mode.

`capture_group_command` opts the same capture loop into `OwnedChild` group
cleanup. It establishes a new group and signals its remaining members before
reaping on leader exit or failure. Original failures retain separate group,
direct-child fallback and reap errors. This adds no retries, inherited input,
cancellation controller, implicit handoff or paid-operation recovery. Escaped
processes remain outside its scope; signalling does not prove descendant exit.

`communicate_child` uses that same IO engine with an existing `OwnedChild`.
Public `OwnedChild::spawn_direct` exposes the existing direct-child constructor
used by ordinary capture. It preserves inherited or configured process groups
and cleans up only the direct child, allowing foreground callers to retain group
membership. Group-owning `spawn` remains unchanged. Terminal control, signal
handling and descendants remain consumer-owned; communication cannot force EOF
from descendant-held pipes under direct ownership.
Callers configure stdin/stdout/stderr before spawn, select borrowed input and
output budgets, and supply a prompt cancellation predicate. Piped IO is fair,
bounded and nonblocking; absent output pipes preserve inherited/file/null IO.
There are no IO worker threads, global signal handlers or retries. Errors retain
the canonical execution/cleanup evidence. Invalid deadlines leave the existing
owner and pipes untouched. Communication time excludes earlier spawning;
consumers retain end-to-end timing and input-size policy.
Capture, communication and executable admission use `OutputLimits`, with
`timeout: Some(duration)` or `None`. No deadline still enforces retention,
polls cancellation during communication and applies cleanup; it may wait indefinitely
for child exit or descendant-held pipe EOF. Cleanup timing is independent.

`communicate_child_with_observer` projects nonempty byte chunks from
that same engine. Per-stream ordering is retained; chunk boundaries and ordering
between streams are unspecified. Each stream chooses `OutputLimit::Terminate(n)`
for hard overflow (only admitted bytes reach the observer), or
`OutputLimit::Truncate(n)` to keep a bounded prefix and forward every drained byte.
Truncation sets the corresponding evidence flag; zero retention stores no output.
Version, Git and Candid parsing require complete evidence even if truncation was
selected. Other consumers can call `CompletedExecution::require_complete`, retaining
their own child-disposition responsibility when using retained ownership.
Observers return `io::Result<()>`; the first sink error becomes
`ExecutionFailure::Observer` and stops observation through the same cleanup path.
Retained bytes, observed truncation and secondary cleanup failures remain available.
Observer rejection takes precedence over hard overflow in the same chunk.
Callbacks run synchronously and must return
promptly. The cancellation callback can project caller-owned heartbeat events
even while output is silent. Callback unwinding closes pipes and attempts the
owner's cleanup before resuming the panic; an unreaped child remains owned for
explicit recovery. There is no separate reader loop, event schema or scheduler.

`SuccessfulExit::Cleanup` preserves ordinary group cleanup on leader exit.
`SuccessfulExit::Retain` keeps the successful leader reserved until the consumer
admits output and rechecks cancellation/deadlines, then explicitly chooses
wait/termination or handoff. Failure still cleans the group. No automatic
background ownership transfer occurs. Broken stdin pipes preserve command
status/diagnostics and do not establish that input was consumed. The new failure
categories and removed obsolete output-pipe categories are detailed in the
[0.7 notes](changelog/0.7.md).

`ic_host_process::child::OwnedChild` shares child spawn/reap/cleanup mechanics
with the direct-child capture engine. Its public spawn establishes a new group
while preserving caller IO and command settings. Ordinary waits reserve an exited
leader until group signalling finishes, then reap it. `poll_exit` can instead
observe status without signalling/reaping while the caller admits IO and checks
cancellation. Only explicit `handoff` of a still-owned, successful leader reaps
without group cleanup; the consumer then owns readiness, stop and recovery.
Running/failed/already-reaped leaders cannot be handed off. Observing success
alone leaves Drop/termination cleanup intact. No reusable PID/group authority
is returned after reaping. The caller drives polling,
natural waiting or cancellation; Drop provides best-effort cleanup, including
unwinding. Typed cleanup evidence remains separate from the original operation
failure. No application readiness, global signal handler, lifecycle controller,
retry, descendant-exit barrier or process-tree containment is introduced.
Exclusive reaping and an unchanged child group are required. The default cleanup
has no wall-clock bound and only reaps the direct child. `spawn_with_cleanup`
can select `CleanupPolicy::TermThenKill` with caller-owned grace and reap limits.
TERM leaves the leader reserved until group KILL, even if it exits during grace.
Repeated termination and Drop share the original reap allowance rather than
restarting it. Timeout retains the child for explicit caller recovery; dropping
it may leave an unreaped child until parent exit. No reaper thread is installed.
Explicit natural waiting and successful background handoff keep their contracts.
TERM, group KILL, direct-child fallback and reap errors remain separate from the
original execution failure. See the [0.8 contract](changelog/0.8.md).

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
  existing single-member gzip decoder. Raw payload slices are hashed directly
  after both byte allowances. `gzip_matches` reuses `MatchingWriter`
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
  interrupted acquisition. Pathname callers first use
  `open_regular_lock_file_with_parents` and retain that descriptor through the
  wait and protected operation. Polling and report cadence, opening, deadlines,
  lock namespace, descriptor clones and final-owner unlock remain consumer-owned.
  The [0.13 migration](changelog/0.13.md) removes the fixed-cadence pathname wrapper.

The exact source revisions, file digests and selected mechanics are recorded in
[source provenance](../ci/extraction-sources.json). These APIs add no JSON schema,
cache policy, root confinement, tool pin or process-tree ownership. Backup's private
parent/file modes and crash barriers are stronger than ordinary durable replacement;
they must not be replaced by `write_with` without preserving those contracts.
`durable::prepare_private_parents` creates only missing owner-only directories and
synchronizes each new directory and parent link. Ancestor symlinks and `..`
refuse before creation, including through already-existing descendants. Callers
intentionally selecting an alias supply its canonical physical path; namespace
stability and root confinement remain caller-owned. Backup retains its own JSON
encoding, descriptor-relative publication and pre/post-publication crash barriers.

The maintainer authorized retiring the original local checkout after verification
of a full backup. Existing consumers retain their prior dependency selections;
adoption of these split crates remains separate. The manifests permit crates.io
publication through the [explicit publication procedure](publishing.md).
The breaking removal of generic APIs from ic-host-tools requires a coordinated
pre-1.0 minor release. This contract assigns ownership; GitHub issues own follow-up tracking.

The standalone `apps/ic-host-wasm-inspect` application composes artifact Wasm facts
with bounded filesystem reads, without process execution or the Candid parser.
The library boundaries stay independent. Explicit paths, four budgets and
performance interpretation stay caller-owned. The standard selected Cargo
installer can prepare its published binary without feature flags. Metrics
retirement uses published 0.17.4 and an admitted exact consumer selection; its
working-tree integration and native acceptance remain separately qualified.
