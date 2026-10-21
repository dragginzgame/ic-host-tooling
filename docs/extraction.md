# Crate ownership and extraction contract

## Coordinated tooling workspace

The Host primitives retain the existing boundaries below, plus the explicitly
approved optional HTTP boundary described here. `ic-build` now shares
this repository and owns generic Cargo execution, build input evidence and
retained artifact caches. `ic-testkit` and `ic-testkit-native` own simulator
integration and native test measurement. `ic-testkit-dev` and
`ic-testkit-perf-probe` are private qualification packages. Build and Testkit
depend on Host; Host does not depend on either. The private Testkit driver
may also depend on Build. One workspace and release identity do not merge
these capability boundaries or enable optional features for consumers.

The maintainer approved `ic-host-http` under
[#117](https://github.com/dragginzgame/ic-tooling/issues/117). It accepts an
already-created native reqwest response and owns bounded body accumulation and
typed partial-failure evidence. It reuses Artifacts' bounded allocation primitive.
It never creates requests or selects status, TLS, compression, redirect, retry,
timeout, decoding, redaction or retention policy. Existing Host/Build/Testkit
packages do not depend on it. Query and Blob retain their distinct status and
failure-evidence contracts; consumer adoption is separate from this implementation.

## Fleet Candid/CDK boundary

The maintainer's 2026-10-18 direction is one controlled dependency boundary for
Candid and the CDK family, with IC Wire owning controlled application protocols.
Host now consumes published `ic-platform` 0.5.0 for bounded Candid decoding,
native parsing and optional certification types. Platform's `docs/migration.md`
owns the fleet adoption contract. Host retains service-to-Wasm projections,
strict certificate CBOR admission and the released aggregate-byte adapter.
Per-reply decoder configuration belongs to Platform; the adapter maps limits and
typed failures while preserving debit-before-decode behavior. Diagnostics follow
Platform's compact policy. The historical observations below describe the wider
migration, not proof of completed sibling adoption.

The dedicated repository has one consumer-facing library, `ic-platform`:
its scope includes runtime integration as well as Candid. Keep
codec, host parser and executor/runtime capabilities separately selectable, with
no default activation. An executor-only consumer must not acquire the parser or
codec. Platform 0.2 owns the maintained Candid-free CDK adapters and explicit-codec
entrypoints, while preserving the registry executor and raw system-call identities.
Host enables none of those runtime features. Any necessary proc-macro implementation
crate belongs to this same boundary repository, not to a second runtime owner.

The boundary owns dependency selection, bounded Candid encoding/decoding,
external-protocol type integration and the supported CDK/executor interface.
It must expose explicit supported operations rather than a blanket upstream
re-export. Where generic external DTOs need Candid traits or derives, a deliberately
supported trait/macro surface is necessary and remains upstream-coupled until
replaced. Identity-only consumers use Platform's unconditional owned `Principal`.
Registry shard APIs use that type; external SDKs and required Candid DTOs retain
`principal::ExternalPrincipal`, with explicit byte-preserving conversions. Neither
the owned principal nor the new owned Account selects a Candid codec or trait.
Canic retains dispatch, authentication and lifecycle policy; Timers retains
scheduling; Query retains NNS/SNS/ledger operations and schemas; Testkit retains
PocketIC lifecycle; Build retains native extraction execution. Host's five crates
retain their existing host responsibilities. The new boundary must not depend on
Host Tools or Testkit and create cycles when those consumers adopt it.

Historical evidence from working trees inspected before Platform adoption on
2026-10-18 (source identities below):

- Testkit's `crates/ic-testkit-candid/src/lib.rs` already owns bounded single-value,
  argument-list and exact-single-value decoders. Query selects that package in
  `crates/ic-query/Cargo.toml`; Testkit calls it from `src/pic/calls.rs`. Consolidate
  this code and Host's pending `candid/decode` under the boundary, preserving their
  distinct arity, diagnostics and caller-owned aggregate-accounting contracts.
- Host's `crates/ic-host-tools/src/candid/service` owns parser-backed service
  inspection. Move parser mechanics to the boundary's host capability; keep
  necessary projection into Host's Wasm method identities in Host, without making
  the boundary depend on Host types. Retire the old decoder owner after migration.
- Canic's `crates/canic-macros/src/adapter/mod.rs` emits `::candid` paths;
  `crates/canic-core/src/cdk/mod.rs` re-exports Candid and `infra/ic/call.rs` uses
  CDK calls directly. Update generators and compile fixtures with imports and
  manifests. Cached Candid derive 0.10.38 accepts `candid_path`; CDK macros 0.20.3
  accept a runtime `cratename` but still emit hard-coded `::candid` paths. A simple
  crate re-export does not establish that consumer manifests can omit both names.
- Timers' `crates/ic-timers/Cargo.toml` uses `ic-cdk-executor` and `ic0`, without
  Candid in its production dependency list. Its `src/platform` uses executor
  method contexts and trap cleanup. Preserve one executor/context identity across
  entrypoints, calls and timers; qualify cancellation and trap behavior together.
- Agent, PocketIC and external management/ledger type packages can bring Candid
  transitively. A ban on direct workspace dependencies does not remove those
  paths, published upstream types, or linked Wasm code. Record selected dependency
  graphs separately from installed-canister size and behavior measurements.

Reviewed checkout HEADs were Host `c8cd434`, Testkit `55ac397`, Canic `5583496bf`,
Query `fdcb0be3`, Timers `194204d`, Wire `ce56d5e` and Storage `009412d`.
Working-tree observations can include pending edits; these identities do not
claim the observations are committed, published or adopted.

The migration acceptance sequence is: qualify a boundary prototype with real
derive/endpoint consumers and host/Wasm feature graphs; publish a compatible
dependency; move the existing codec/parser owners and their callers together;
then migrate runtime integrations and enable direct-dependency bans per migrated
workspace. Include examples, tests, generated templates and independent fixture
workspaces. Shared's `scripts/ci/check-cargo-dependency-boundaries.sh` already checks
canonical package names despite aliases, across normal/build/dev/target tables.
Adopt it through a reviewed snapshot rather than writing a parallel checker.
Its scope is direct workspace dependencies, so separately enumerate CDK-family
packages, inspect transitive graphs and check unused root catalog declarations.

External management, NNS/SNS and ledger calls retain their specified wire formats.
Changing Rust dependency ownership must preserve request/reply bytes, errors,
decode limits, persisted records and retry identities. Protocol replacement for
owned endpoints is a separate IC Wire cutover with both ends qualified. Required
evidence includes malformed/quota/arity cases, macro expansion without forbidden
direct dependencies, installed query/update/management calls, async/trap cleanup,
and supported host checks. No fleet-wide source migration is qualified by this audit.

Storage is a separate boundary: current IC Storage delegates collection formats
to `ic-stable-structures` and allocation/bootstrap/exclusive handles to IC Memory.
Removing those dependencies requires an explicitly selected replacement for both
roles and retained-data qualification. The existing Shared stable-memory rule
requires IC Memory handles; retiring that owner also needs a coordinated baseline
change. No allocation or stored-format migration is implied by the Candid facade.

## Current implementation

The initial library source is committed IC Host Tools 0.2.0 at
be7d73908225e73ab5babf8fbb9fe959f40d439b. Pending sibling tooling edits
were excluded. Canic's clean durable module and its tests are bound to their
observed committed bytes in [source provenance](../ci/extraction-sources.json).

Dependencies point down from ic-host-process to ic-host-fs and from each to
ic-host-artifacts. Tools selects Artifacts only for explicit Wasm profiles;
its production graph has no filesystem or process dependency. No generic crate
depends on Tools, Canic, IcyDB or an IC runtime. Test/example artifact features
do not become production filesystem dependencies. Tools' default response
interpretation adds no generic Host dependencies; `icp-project` selects Text.
Candid service inspection requires its explicit `candid` feature.
Native extraction moves to IC Build's `artifacts::candid` for the 0.24 hard cut;
the five Host crates retain their generic mechanics and IC interpretation roles.
The 0.23 hard cut moved generic line normalization into
`ic_host_text::normalize`.
Shared method identities remain unconditional. Candid service inspection and
bounded reply decoding require `candid`; Platform and its native parser are
optional. The independent `certificate` feature selects certification without
Candid. `ic-interface` adds generic Wasm facts and export projection without
Platform, Candid or filesystem/process dependencies. Select both `candid` and
`ic-interface` for service/export comparisons.
Replica revision selection, install thresholds and resource-report policy belong
to the consuming application. Host Artifacts retains generic Wasm facts; Tools
no longer owns an `ic-limits` profile or a reference-policy CLI example.

- ic-host-artifacts owns pure ordered nonempty/unique batch-label admission,
  declared-file directory checksums, stream identities,
  bounded read/write/copy mechanics,
  including allocation-free visitation with caller-owned scratch and callbacks,
  optional gzip encoding/decoding, optional Wasm
  structural facts, encoded code/data statistics, bounded custom-section framing,
  full core-module validation, generic fixed-memory call-module encoding and
  optional core WebAssembly Text compilation.
- ic-host-fs owns no-follow directory hashing and ordered sync/checksum traversal
  over caller-held descriptors,
  held directory capabilities with private cap-* traversal and bounded enumeration,
  fresh child-directory allocation with explicit permissions and collision bounds,
  standard cache-directory tag recognition and durable replacement with explicit
  suffix bytes and permissions,
  pathname/descriptor reads, explicit invocation-directory
  cleanup custody and the durable module's replace,
  create-new, private-file, parent synchronization and descriptor-lock operations.
  `durable::SharedFileLock` explicitly unlocks on guard drop before closing;
  consumers retain shared guard lifetimes, paths and cache/maintenance policy.
  Its `scratch::create_private_bytes` shares private, non-durable invocation-file
  creation; consumers retain byte budgets, successful cleanup and effect recovery.
  Its separate `scratch::TemporaryFile` owns scoped named-file cleanup through
  private tempfile allocation; retained files, unnamed files and directory custody
  are separate contracts. The [capability guide](capability-fs.md) identifies
  Query's remaining cache-specific adapter and discovery responsibilities.
  Read-only and ordered-sync visitors share checksum collection while retaining
  their released descriptor interfaces and distinct effects.
- ic-host-process owns executable resolution/admission, installed-version
  observation, explicit execution context,
  output limits, deadlines, direct-child/explicit group cleanup evidence, native
  process-table/RSS observations and separate Git queries.
  Its `framing` module owns bounded newline admission for buffered readers and
  incremental output chunks. Schemas, EOF-tail policy, deadlines, diagnostics and
  recovery remain with each process consumer.
  Its Unix `custody` module also owns inherited regular-sidecar descriptors and
  exact existing-sidecar quiescence. Callers retain paths, environment names,
  durable identity records, wait settings, command/effect admission and recovery;
  custody neither kills processes nor implies a remote effect outcome.
- ic-host-text owns bounded UTF-8 line normalization, pure terminal-safe escaping,
  and optional typed YAML decoding with input-byte and tag/merge admission.
  The private YAML backend is owned here; schemas, filesystem/includes and
  application configuration rules remain with consumers. Tools' `icp-project`
  selects this Text feature rather than another parser implementation. Its private
  serde_yaml backend matches ICP CLI; all Text profiles retain Rust 1.91. Text also owns
  explicit column clipping, Unicode display-width tables and pure single-line
  progress frames. Its `timestamp` module owns canonical UTC second spelling and
  Gregorian conversion, formatting unsigned Unix seconds and parsing zero through
  `i64::MAX`. Callers retain clocks, freshness and collection-time interpretation.
  Its `decimal` module owns exact two-place rounding/rendering with explicit
  scaling; consumers retain normalization, units, sentinels and raw report fields.
  Its `duration` module owns positive whole-second parsing, rounded single-unit
  display and truncating two-unit display. Clock selection, deadlines, freshness,
  persisted units and output remain caller-owned. Source and six tests are retained
  from Host PR #162 at `58e03fbb0b2c2e13816a1c34fb7da6965d68942e`.
  Progress callers retain terminal detection, wrapping limits, output, flushing,
  failure policy and cleanup timing; the helper never selects a sink or writes.
  Escaping borrows unchanged input through `Cow<str>` and owns changed text;
  tables keep safe cell borrows and return one owned rendered string.
  Cells, alignments and formatting options are explicit; reports and CLIs retain
  headers, rows, column choices, size/truncation policy, output, terminal detection,
  colour and progress lifecycle. It depends on no other Host or IC owner. This fifth generic
  boundary was explicitly selected by the maintainer under
  [#110 — Host text ownership](https://github.com/dragginzgame/ic-host-tooling/issues/110).
- ic-host-tools owns Candid service-method interpretation,
  reply decoding with explicit byte/work and caller-owned aggregate limits,
  IC method export projection over generic Wasm facts, JSON/plain-hex ICP responses
  and bounded certificate decoding. Its `icp` module owns pure call/status argv
  construction and successful status JSON's null/string installation projection;
  executable/context admission, capture bounds, execution, targets, full status
  reports and lifecycle stay with callers. The implementation and four tests are
  taken from reviewed Host PR #101 at `38b114bb30f12e9a6e255ccd85d6b7344331d763`.
  CLI labels, preambles and any Unicode whitespace normalization remain with
  consumers rather than a second presentation adapter in the codec.

IC Build owns native Candid extraction orchestration over Host's hashing and
execution primitives. It retains complete bounded raw evidence and source/tool
identities without normalization or publication policy. Consumers needing text
call `ic_host_text::normalize::normalize_lines` explicitly with an output allowance;
UTF-8, normalization, grammar and expected-service admission remain separate decisions.
The [0.24 migration](changelog/0.24.md) records producer delivery and consumer actions.

The optional tools `registry` feature supplies focused Registry Protobuf
projections and canonical canister-range shard-key/interval admission. It adds
Candid and Prost only; no Agent, certificate parser, filesystem or process owner.
Callers must enforce input/work budgets and own discovery, trusted versions,
certification, full range ordering, completeness, replay and report policy.
The projections preserve Query 0.57.0's tags/discriminants and are not a complete
upstream Registry schema; unknown fields retain normal Prost behavior.

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

The optional native Tools `agent` feature shares response-bounded SDK construction
and lazy exact endpoint/ceiling binding using Platform's supported Agent API.
The caller retains endpoint admission, trust roots, identity, historical policy,
deadlines and every network call. Defaults/certificate exclude transport, and
the Wasm graph excludes it even with the feature selected. Process and Tokio
are dev-only fixtures; no lifecycle owner or automatic network activity is added.

The optional tools `encoding` feature decodes a complete canonical unsigned
`u64` LEB128 value without allocation or new dependencies. It rejects truncated,
overflowing, nonminimal and trailing encodings while inspecting at most ten bytes.
Callers retain field names, semantic limits, transport and trust policy.

The optional tools `certificate` feature owns strict CBOR decoding into owned
certificate/delegation/hash-tree values supplied by `ic_platform::certification`.
These retain upstream `ic-certification` type identities. It admits complete
input bytes before decoding with the IC Agent's `serde_cbor` codec and its
built-in recursion limit, returning one fully consumed value. Host has no depth
override or competing CBOR parser. Direct Serde visitors enforce actual array/byte kinds, exact tree tags
and arities and 32-byte pruned digests without a generic CBOR payload tree.
Certificate/delegation fields reject unknown, duplicate, missing or invalid
values. These stricter schema adapters remain necessary: upstream storage
deserialization also accepts text/integer-array byte projections and does not
detect every duplicate-null field. Errors distinguish byte allowance, trailing
data and CBOR/shape failures.
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

The independent `wasm-encoding` feature owns `wasm_encoding::encode` and its
Host records, backed privately by `wasm-encoder`. Its demonstrated consumer is
Testkit's raw-reply fixture builder: imports, fixed Wasm32 memory, active data and
exported functions that call imports with i32 constants. Host encodes those generic
mechanics. Testkit retains IC method-name admission, query/update prefixes,
reply limits, offset/page selection, error translation and PocketIC lifecycle.
No upstream encoder types or raw instruction escape hatch are exposed. The
output-byte allowance checks input lower bounds before encoding and complete
framing afterwards; it is not a hard allocation or CPU quota. This feature does
not imply inspection or validation. The existing custom-section encoder retains
its stronger pre-allocation admission and independent minimal feature contract.

`wasm-validation` adds `wasm::validate` and owned `ValidationError`, using the
private parser's full validator rather than structural inspection. It implies
`wasm`, leaving inspection-only graphs free of validation machinery. Consumers
retain target/proposal admission, execution and installation policy. Testkit can
replace its direct full-validation calls without weakening fixture coverage;
its adoption and dependency removal require the supplying Host release.

The independent `wasm-text` feature owns `wasm_text::compile`, using `wat` with
component support disabled. It admits UTF-8 source length before compilation,
then checks complete output length and core-module framing. These are byte
allowances, not CPU/heap quotas, and compilation is not full type/instruction
validation. Backend diagnostics are owned strings and may contain source excerpts.
Backup retains its stateful fixture sources and lifecycle; it can call this API
without a direct WAT dependency. Query's raw-reply fixture fits Testkit's existing
fixture builder, so it can eliminate its generated WAT instead. Neither route
requires a general instruction API or upstream encoder/parser re-export.

`read::open_regular_file_no_follow` exposes the existing Unix read-only opener's
regular descriptor admission without consuming bytes. Backup can sync its
reference descriptor and Canic can inspect a lock using that same standard handle.
Paths, permission admission, error mapping and locking decisions stay local.
Process's existing `signal::SignalActions` handles installation/restoration for
Testkit/Query-style exclusive registrations. Canic's cooperative signal-hook
callbacks and terminal restoration have distinct semantics; they are not replaced
by an exclusively owned disposition guard. No second signal abstraction is added.

`candid::service::inspect` uses the Candid parser for self-contained grammar and
type checking, returning method names and modes without expanding argument/return
types into a presentation tree. Imports are rejected rather than loaded or ignored.
Consumers select source-byte and returned-method limits; these are not a hard
heap, stack or CPU quota for the parser. `interface::exported_methods` borrows
function export facts without reparsing Wasm. It preserves internal methods,
empty names and mode differences for caller admission. Oneway remains distinct
in Candid and maps explicitly to the update export namespace through `as_export`.
Shared method identities remain available without Wasm or Candid support.
Select `candid` for parsing and `ic-interface` for export projection.
Expected interfaces, CDK allowlists, type compatibility, payload annotations and
Canic endpoint presentation remain consumer policy. IC Build owns build/cache
orchestration separately; these libraries do not install tools or build canisters.

Within `ic-host-fs`, `read` is the sole owner of required/optional/private reads.
Its metadata-returning hash variants retain the initial metadata from the same
descriptor used for bounded hashing. Identity-only hash calls project that result;
Process consumes the paired observation for its executable permission policy,
after byte/digest admission. Metadata can become stale under concurrent writers;
neither paired observation nor hashing establishes atomic pathname execution.
Its Unix `tree` module owns no-follow descriptor traversal and bounded directory
hashing; `ic-host-artifacts::artifact::tree` owns exact checksum framing. Ordered
sync/checksum traversal belongs to FS `durable::tree`. Backup retains checksum
records, expected-identity admission, complete artifact publication policy and
retained byte custody. Callers admit and hold the root, select
depth/byte limits, and prevent concurrent namespace or content changes where
stable evidence is required.
Observed directories always use directory-only opens and descriptor type checks;
callers do not choose kernel flags. Native open errors may precede type callbacks
when an entry is replaced during traversal.
Following paths, final-component no-follow admission and already-open descriptor
reads remain distinct contracts. `read_file_prefix` permits a larger regular
file and retains length from the same admission observation, returning bounded
raw bytes. Consumers retain decoding and omitted-byte presentation; concurrent
writers can invalidate the observed length. Optional reads reuse the same bounded
stream engine and return `ArtifactError`; private reads add typed permission, link-count
and fixed-length rejection. `hash_file_no_follow` combines the same Unix opener
and descriptor admission with the artifact owner's constant-memory hash traversal;
metadata and streamed bytes have independent bounds. It rejects final symlinks,
while existing `hash_file` continues to follow them. Ancestor confinement,
concurrent-writer custody and exact expected-length checks remain caller-owned.
`open_entry_beneath` selects a regular file or directory through held root and
parent descriptors, refusing traversal, observed symlinks and special entries.
It does not guarantee immutable bytes or continuing pathname containment.
Durable publication and locks do not own a second
reader hierarchy. Lock errors retain typed admission and original I/O causes;
durable creation errors retain `NamedWriteError` in `RegularFileLockError::Publication`.
The durable directory owner sequences a caller-verified existing tree through
no-replace sibling publication and parent synchronization. It does not create,
clean, hash or traverse that tree. The verification callback owns complete
child/root synchronization and exact retained-byte comparison before the rename;
the consumer owns namespace exclusion, recovery meaning and retry authority.
Post-rename failures are explicitly distinct from before-publication failures.
`durable::write_retained_at_with` completes create/produce/sync for a caller-named
entry without unlink, rename or retry. Typed phase failures and panics preserve
the original entry, including partial evidence whose durability is unconfirmed.
`try_lock_exclusive` makes one nonblocking attempt on a caller-owned regular
descriptor; contention retains native `WouldBlock`. Blocking callers use
`lock_exclusive` on that descriptor. Observed waits compose the opener with
`lock_exclusive_with_wait` and caller-owned polling/reporting intervals. No filesystem latency
deadline, retry controller or stale-lock removal is introduced.
`open_regular_lock_file_with_parents` is that single admission owner, exposed
without acquisition so consumers choose shared, blocking, immediate or observed locks.
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
writer. `write_bytes(path, options, bytes)` uses it with explicit publication mode
and permissions, returning `NamedWriteError<io::Error>` rather than discarding
publication state or cleanup evidence. Consumers retain error projection
and recovery decisions; an after-publication failure requires reconciliation.
`write_existing_parent_validated_with` reuses descriptor publication after
directory-only opening of an existing selected parent. It refuses missing or
special-file parents without creating them, follows parent aliases and selects
the directory once across renames. `write_at_validated_with` accepts the held
parent directly. Both require explicit result admission after file sync and
before publication; no-op admission is caller-selected. The forwarding variants
retire without aliases. Neither supplies root confinement or lifecycle policy.
The [0.10 migration](changelog/0.10.md) replaces the old two-argument writer and
removes the `write_typed_with` name without a compatibility alias.

`durable::write_named_with` exposes an absolute owned staging pathname to external
producers through the existing publication engine. Callers supply `WriteOptions`
for publication mode and staging permissions. Filesystem code owns exclusive
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
Backup's checksum records and admission policy, multi-file publication and
journals remain consumer-owned; generic hashing and framing use the Host owners.
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
membership. Group-owning `spawn` requires an explicit cleanup policy. Terminal
control, signal handling and descendants remain consumer-owned; communication cannot force EOF
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
Exclusive reaping and an unchanged child group are required. Group spawning
requires `CleanupPolicy`: `KillAndWait` has no wall-clock bound and only reaps
the direct child; `TermThenKill` uses caller-owned grace and reap limits.
TERM leaves the leader reserved until group KILL, even if it exits during grace.
Repeated termination and Drop share the original reap allowance rather than
restarting it. Timeout retains the child for explicit caller recovery; dropping
it may leave an unreaped child until parent exit. No reaper thread is installed.
Explicit natural waiting and successful background handoff keep their contracts.
TERM, group KILL, direct-child fallback and reap errors remain separate from the
original execution failure. See the [0.8 contract](changelog/0.8.md).

The sole-zombie macOS check uses one private fixed-buffer libproc call, explicitly
approved by the maintainer. Explicit Unix `signal::SignalActions` additionally
permits scoped FFI for caller-selected action installation, rollback and
restoration. Its unsafe entrypoint requires async-signal-safe callbacks and
exclusive process-global registration custody; cancellation and shutdown policy
remain consumer-owned. Unsafe code stays denied elsewhere. Other process
syscalls use safe rustix APIs; libc is Unix-scoped without a bindgen toolchain.

`AdmittedTool::admit_version` shares the exact admission engine for tools whose
installation the caller already trusts. It takes a `VersionSpec` and a mandatory
exact version identity, records the bounded executable identity and admits exact
version output; later runs enforce that observed digest. It removes consumer-side
hash/admit composition without
claiming that observed bytes are a trusted published pin. Pinned admission takes
the same `VersionSpec` and exact version plus a mandatory digest, checked before
executing the version probe.
Installation provenance, per-host pins and selection of either contract remain
consumer policy; no host catalog or version-range resolver is introduced.

Additional reusable mechanics are extracted from committed sibling source:

- `artifact::hash_reader` captures a bounded stream identity once;
  `ArtifactIdentity::verify` admits that identity against caller-selected authority.
  The combined `verify_reader` wrapper is retired. Hash/limit failures precede
  digest comparison; a mismatch retains both authority and observed identity.
- `artifact::hash_gzip` reuses bounded hashing over the existing single-member
  gzip decoder. Callers select representation and admit input/payload bounds;
  the unused mixed raw/gzip auto-detection wrapper is retired.
  `gzip_matches` reuses `MatchingWriter`
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
  Its third argument requires the consumer's limit on simultaneously active
  missing-target expansions; `usize::MAX` explicitly selects no added depth bound.
  It neither confines a root nor creates files. Hard-link identity, opening and
  concurrent pathname revalidation remain consumer-owned.
- `ic_host_fs::durable::lock_exclusive_with_wait` acquires a caller-owned regular
  descriptor with explicit polling and contention observations, retrying
  interrupted acquisition. Pathname callers first use
  `open_regular_lock_file_with_parents` and retain that descriptor through the
  wait and protected operation. Polling and report cadence, opening, deadlines,
  lock namespace, descriptor clones and final-owner unlock remain consumer-owned.
  The [0.13 migration](changelog/0.13.md) removes the fixed-cadence pathname wrapper.

Unix native device/inode observations belong to `ic_host_fs::identity`, shared
by filesystem publication and Process custody. File-kind admission, locks,
namespace stability, retention formats and lifetime authority stay with callers;
the identity neither freezes content nor prevents inode reuse after close.

The FS package also owns the [native file command adapter](file-cli.md), which
uses its existing no-follow reads and durable writer. It selects no batch limits,
installation pins, product schemas or recovery policy. Auth retains its browser
contracts and lockfile adapter; its file commands retire only after published
consumer qualification.

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

Consumers depend directly on each crate owner. Historical extraction provenance
does not select current consumer versions or authorize dependency upgrades.
The manifests permit crates.io publication through the
[explicit publication procedure](publishing.md). This contract assigns ownership;
GitHub issues own follow-up tracking.

The standalone `apps/ic-host-wasm-inspect` application composes artifact Wasm facts
with bounded filesystem reads, without process execution or the Candid parser.
The library boundaries stay independent. Explicit paths, four budgets and
performance interpretation stay caller-owned. The standard selected Cargo
installer can prepare its published binary without feature flags. Consumers own
binary selection, installation and qualification; Host supplies no Metrics
integration or canister build workflow.
Optional expected-digest verification and export/custom-section details are also
owned by this CLI; the duplicate filesystem `inspect_artifact` example is retired.
Default TSV output remains stable, and the filesystem development profile no
longer selects the Wasm parser for that example.

The optional `icp-project` tools profile parses bounded effective YAML produced by
`icp project show` and projects exact environment/network facts. It selects Text's
optional YAML decoder; the default tools graph excludes Text and that parser. ICP
owns implicit-environment and recipe resolution. Callers retain command execution,
input-byte budgets, selected identities, local funding and lifecycle. This module does
not reconstruct effective configuration from raw manifests.

The opt-in `ic-host-process::testing::http` module owns bounded synthetic request
reading, including phased request-line admission and remaining-deadline socket
reads. Its one-request `HttpFixture` composes that reader with HTTP 200 framing,
explicit caller-selected response bytes, request budgets and one total deadline;
it cancels socket IO and joins its worker on Drop. `testing::net::accept_tcp_until`
shares bounded acceptance with custom-response fixtures. Payload schemas,
application assertions, simulator lifecycle and evidence files stay consumer-owned.
HTTP fixtures are portable; executable fixtures and crash barriers remain Unix-only.

## Logical directory sizes from IC Build

The metadata-only walker and its two filesystem regressions come from
[IC Build 0.12.0](https://github.com/dragginzgame/ic-build/blob/2598a417a96834d3139751adf549126add666c4b/crates/ic-build/src/artifacts/cache_fs.rs),
through [PR #173 — Extract logical directory-size observation](https://github.com/dragginzgame/ic-host-tooling/pull/173).
The algorithm retains iterative traversal, metadata lengths, saturating sums
and original I/O failures. Host owns this pathname observation; callers retain
namespace custody, resource admission, cache policy and cleanup. Build caller
replacement requires published APIs and its own qualification.
