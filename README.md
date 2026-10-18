# IC Host Tooling

Host-side Rust libraries for artifact inspection, local filesystem safety,
admitted process execution and IC-specific format handling.

| Crate | Implemented source ownership |
| --- | --- |
| ic-host-artifacts | Bounded streams and hashing writers, exact byte comparison, SHA-256, gzip encoding/decoding, generic Wasm facts and custom-section encoding |
| ic-host-fs | Regular/no-follow reads, missing-suffix path resolution, streamed durable publication, private files and observed descriptor locks |
| ic-host-process | Executable resolution, digest/version admission, bounded capture, explicit child/group cleanup and Git observations |
| ic-host-text | Pure line normalization, decimal/UTC text, terminal-safe rendering, Unicode tables and progress frames |
| ic-host-tools | Candid service facts, optional IC export facts, ICP command syntax, response decoding and certificate decoding |

These run locally outside canisters. The ic-host prefix identifies their ecosystem;
the generic crates have no IC runtime dependency. Gzip and Wasm
support are optional features of ic-host-artifacts. Filesystem/process consumers
select the small default artifact dependency.

`ic_host_artifacts::artifact::visit_reader` visits bounded chunks using nonempty
caller-owned scratch and a typed callback. Framing, callback effects and blocking
deadlines remain caller-owned. `ic_host_fs::directory::create_unique_directory`
allocates a fresh child with explicit permissions and collision allowance;
it preserves parent aliases and existing entries, and supplies no cleanup or sync.

`ic_host_fs::durable::SharedFileLock::acquire(file)` holds a blocking shared
lock on a caller-opened regular file. Drop explicitly unlocks before close,
preventing duplicates from extending the guard's lock; independent acquisitions
remain separate. Paths, shared lifetimes, maintenance and cleanup stay with callers.
`ic_host_fs::cache_tag::ensure(directory, suffix, permissions)` preserves valid
regular `CACHEDIR.TAG` files or durably replaces invalid entries with the standard
signature and exact suffix. It uses ordinary durable parent creation. Callers
retain root admission, namespace custody, suffix budgets and cache maintenance.
`ic_host_fs::owned_directory::OwnedDirectory` holds caller-asserted cleanup
custody without creating an entry. Explicit preservation/disarming transfers
recovery; armed Drop attempts removal. Consuming `cleanup()` makes one attempt,
including on failure; `try_cleanup()` preserves the existing Drop state on failure.
Callers retain
namespace custody,
destructive-operation policy and cache/application lifecycle.

For private invocation arguments, `ic_host_fs::scratch::create_private_bytes`
creates a complete, closed file without disk synchronization. Select the parent,
prefix, bytes and collision-attempt allowance explicitly. Successful files remain
caller-owned; failed writes retain the original error and any failed removal.
The parent and candidate namespace must remain under the caller's custody.

For test fixtures, opt in to `ic-host-fs`'s `test-support` feature in a
dev-dependency. `testing::FixtureDirectory::new(parent, prefix)` allocates beneath
an existing physical parent and removes the owned directory on normal Drop.
During panic it retains the directory and reports its path. Callers retain
namespace custody. `testing::create_fixture_directory(parent, prefix)` remains
available for explicit caller-owned retention without a Drop guard.
For bounded synthetic HTTP peers, opt in to `ic-host-process`'s `test-support`
feature and use `testing::http::{capture_http_request, read_http_request_line,
read_http_request_with_line}`. Whole capture retains partial bytes and typed
failures; callers select required or optional Content-Length and own responses,
byte budgets, deadlines and evidence files.
With `wasm`, `wasm::encode_custom_section(CustomSection { name, data }, max_bytes)`
encodes one complete custom section with an explicit output-byte allowance.
Callers supply exact names/data and own module assembly, aggregate budgets,
duplicates, append order and subsequent Wasm validation.
With the same `wasm` feature, `wasm::code_statistics(bytes, module_bytes)` counts
encoded indirect calls, including tail calls, and data payload bytes without a
subprocess. These counts differ from encoded section sizes and need not match
optimizer reachability metrics. Type/feature validation and report policy remain
caller-owned; the structural `wasm::inspect` contract is unchanged.
`ic-host-tools` has no default features. Response decoding, Candid service
inspection and shared method identities are always available; `ic-platform`
owns the Candid codec and native parser. The optional `ic-interface` feature adds
export projection over generic Wasm facts for service/export comparisons.
The default graph has no other Host library dependency; `ic-interface` adds
Artifacts without introducing filesystem or process execution.

Native Candid extraction belongs to IC Build's `artifacts::candid` module;
Host Tools no longer supplies an extraction feature or invocation wrapper.
Consumers select tools, source custody, output limits and optional normalization.
See the [0.24 migration](docs/changelog/0.24.md) for the required Build release.

`ic_host_tools::candid::decode::decode_one` decodes a reply with explicit
`DecodeLimits` and a mutable aggregate byte allowance. Oversized replies refuse
without charging; admitted malformed replies consume their bytes. Candid's
compatibility rules and skipping quotas apply to extra arguments. Callers retain
authentication, schemas, domain errors and all resource-policy choices.
Platform owns the bounded codec and its compact diagnostics; Host retains the
aggregate-accounting adapter and preserves original typed decoder causes.
Earlier feature/import changes remain in the
[0.21 migration notes](docs/changelog/0.21.md) and release history.
Replica install thresholds and their reports belong to the consuming application.
Canic selects its replica revision and enforces its resource limits; Host Artifacts
retains generic Wasm inspection. See the [0.25 migration](docs/changelog/0.25.md).

The optional `icp-project` feature exposes
`icp_project::environment_network` over bounded `icp project show` YAML.
ICP resolves implicit environments and recipes; the parser preserves exact names
and refuses malformed or ambiguous facts. Callers retain executable selection,
capture limits, funding and lifecycle. The YAML parser is absent from defaults.

The optional tools `certificate` feature exposes
`certificate::{decode_certificate, decode_hash_tree}` with explicit input-byte and
CBOR-depth limits. It returns owned certification values through Platform,
preserving `ic-certification` type identity, and rejects trailing
data, malformed trees and invalid certificate fields. Parsing authenticates
nothing: callers retain Agent verification, trust roots, combined evidence
budgets, witness commitments and application leaf meanings. The feature adds no
Agent transport, filesystem/process execution or Candid parser.

`ic-host-tools::response::decode` accepts explicitly selected JSON envelopes or
plain hex. CLI presentation labels, preambles and any broader whitespace policy
belong to callers; the codec does not infer or strip them. See the
[0.22 response migration](docs/changelog/0.22.md).

`ic-host-tools::icp` builds call/status commands from an already-configured
`std::process::Command`, preserving native argument bytes and caller context.
It supports textual or binary-file arguments, query/update selection and status
output selection without dispatching a process. `decode_installation_status`
projects null/string `module_hash` values only from successful caller-captured
JSON; missing/invalid fields never establish absence. Executable/version admission,
credentials, targets, capture limits, retries and lifecycle remain caller-owned.

`ic_host_artifacts::batch::validate_labels` admits caller-selected batch labels
before work, rejecting the first empty or exact duplicate with occurrence indices.
Empty batches are valid. Callers retain ordering, execution and failure policy;
iteration must not perform batch effects.

`ic-host-text::escape::terminal_text` makes control and bidirectional formatting
characters visible without choosing output or colour. It returns `Cow<str>`,
borrowing safe input; use `.into_owned()` when independent ownership is required.
`table::render` accepts
headers, fixed-column rows, alignments and explicit `Options { column_gap,
header_separator }`, returning a String. It measures escaped cells in Unicode
display columns. Reports retain their row/header/column decisions; CLIs retain
stdout, terminal detection, colour and progress lifecycle. `width::fit_line`
escapes controls
and clips with an ellipsis to caller-selected columns, using conservative scalar
widths. It preserves UTF-8 boundaries without promising grapheme-cluster boundaries.
Clipping consumes escapes as needed without allocating the complete escaped line.
`progress::ProgressLine` returns escaped update, finish and clear frames with
scalar-column padding. Callers prevent wrapping and own writing, flushing,
output failures and cleanup timing; dropping the helper performs no IO.
The crate depends only on the Unicode-width library, with no other Host or IC
dependencies.

`ic_host_text::decimal` renders exact fixed or compact two-place decimal text
with explicit power-of-ten scaling and half-up rounding, including values beyond
`u128`. It uses no floating point. Callers retain input normalization, unit/symbol
selection, sentinel handling and raw report fields.

`ic_host_text::timestamp` formats unsigned Unix seconds and parses canonical UTC
second timestamps through `i64::MAX`, without clocks or timezone selection.
It preserves expanded positive years, strict spelling and impossible-date refusal;
callers retain freshness and collection-time policy.

`ic_host_text::normalize::normalize_lines(bytes, max_bytes)` separately admits
UTF-8, trims each line's trailing Unicode whitespace and emits LF endings while
preserving blank lines. It bounds both input and normalized output and reserves
storage fallibly. It does not escape controls or validate a document grammar.
See the [0.23 import migration](docs/changelog/0.23.md).

`tool::capture_command` accepts a caller-configured `std::process::Command` when
the consumer owns executable admission. It shares bounded capture and direct-child
cleanup with `AdmittedTool`, whose exact digest/version checks remain in place.
`ToolError::evidence` and `execution_error` borrow retained diagnostics without
copying output; callers own presentation and recovery.
`tool::format_failure_evidence(context, &error)` explicitly renders status, lossy
stdout/stderr and cleanup failures once when raw diagnostics are requested. Callers
retain terminal escaping and redaction; ordinary error display omits captures.
Capture and communication return `CompletedExecution` with a required exit
status, including nonzero and signalled exits. Use `require_success()` when zero
exit is required. Interrupted execution retains optional status in `ExecutionEvidence`.

`child::OwnedChild::spawn(command, cleanup)` starts a caller-configured command
in a new owned process group. It preserves command IO and supports polling,
waiting, explicit termination and cleanup during unwinding. Callers retain admission, readiness,
cancellation and application lifecycle policy. For deliberate background startup,
`poll_exit` retains cleanup ownership through result admission; explicit `handoff`
then reaps a successful leader without stopping the background group. See the
[handoff contract](docs/changelog/0.5.md#explicit-successful-background-handoff) and the
[child cleanup contract](docs/changelog/0.5.md#explicit-child-and-process-group-cleanup)
for exclusive ownership and descendant limitations.

Unix `custody::CommandCustody` instead retains a caller-selected sidecar lock
through one command's inherited descriptor and cooperating descendants. Persist
its `ic_host_fs::identity::FileIdentity` with original intent before `spawn`,
selecting the descriptor environment name explicitly. `finish` closes owner
custody and probes exact existing-sidecar quiescence using caller-selected grace
and polling. A missing or
replaced sidecar never proves quiescence. `QuiescenceGuard` cannot spawn and
explicitly unlocks on drop. Callers own namespace stability, descendant descriptor
preservation, output/reaping, records, remote effects and recovery; no sidecar is
deleted and a failed spawn consumes the local allowance. Group cleanup and
inherited custody are distinct contracts.

For `OwnedChild::spawn`, select `CleanupPolicy::KillAndWait` for immediate KILL
with synchronous reaping, or `CleanupPolicy::TermThenKill` for caller-owned TERM
grace and bounded reaping.
Communication failures and Drop use that same explicit policy. Reap timeout retains
the owner for explicit recovery and does not start a background reaper. See the
[bounded cleanup contract](docs/changelog/0.8.md#caller-selected-termination-timing).

`framing::read_line(reader, maximum)` returns one newline-terminated byte frame
without consuming later buffered messages. `LineFramer::feed` instead assembles
frames across arbitrary output chunks and emits complete frames by callback;
`finish` reports a retained unterminated tail for caller-selected EOF policy.
The inclusive allowance includes the newline. Size/EOF refusals expose bounded
prefixes; I/O failures retain their original causes. Display text is redacted and
an incremental refusal is terminal. Schemas, diagnostics, deadlines, stream
lifetime and recovery stay local.

`tool::capture_group_command` combines bounded stdout/stderr capture with that
group cleanup for commands whose descendants must be signalled on exit or
failure. Ordinary `capture_command` retains direct-child cleanup. Both share one
capture engine; callers retain budgets, admission and external-effect recovery.

`OwnedChild::spawn_direct` preserves inherited or caller-selected process groups
for foreground communication and cleans up only the direct child. It uses the
same I/O engine; consumers retain terminal control, interruption handling and
descendant lifetime, including any descendant-held output pipes.

`tool::communicate_child_with_observer` adds live stdout/stderr observation to
`communicate_child` for an existing owner. Each stream chooses
`OutputLimit::Terminate(bytes)` to fail on overflow or `OutputLimit::Truncate(bytes)`
to forward all drained output while retaining only a bounded prefix.
The cancellation callback can also emit caller-scheduled heartbeats while silent.
Callback unwinding attempts cleanup even when the borrowed owner survives the
caller's panic handler.
Output observers return `io::Result<()>`; a sink error retains its original cause
and triggers the same child cleanup without a caller-side cancellation workaround.

All capture, communication and admission functions accept one `OutputLimits`
type with `stdout`, `stderr` and `timeout: Option<Duration>`. Select `None` for
long builds without an elapsed-time deadline, or `Some(duration)` for a positive
deadline. Retention and cleanup still apply; communication also polls cancellation.
Version, Git and Candid admission reject truncated captures. Other consumers can
use `CompletedExecution::require_complete` before parsing retained output.

Examples live with their owners: `ic-host-artifacts` has `hash_serialized_json`;
`ic-host-fs` has `inspect_regular_file` and
the artifact/file composition `inspect_gzip`;
`ic-host-process` has `inspect_git` and `resolve_tool`. `ic-host-tools` retains
`inspect_response`; the response
example runs with `--no-default-features`. Example-only dependencies do not widen
the tools crate's production feature graph.

`AdmittedTool::observe_version` accepts a `VersionSpec` and observes
installed bytes plus one nonempty, complete UTF-8 version result when no expected
version is selected. It supplies provenance, not installation authenticity; later
runs retain byte-drift rejection.

`AdmittedTool::admit_version` accepts a `VersionSpec` and a separate required
version identity for caller-trusted installed tools without a published binary
digest. It records the installed identity, checks the exact version and rejects byte drift before later execution. That
observed identity is not authentication; installation provenance stays with the
caller. `admit` and `admit_no_follow` take the same `VersionSpec` with a separate,
mandatory digest pin, checked before executing the version probe. The expected
version is a separate mandatory argument on both pinned entrypoints as well.
The [0.25 upgrade notes](docs/changelog/0.25.md) describe the shared probe
configuration and required arguments.

Filesystem reads live under `ic_host_fs::read`, including optional bounded
no-follow reads and typed private-file admission. Durable publication and locks
remain under `durable`. Upgrading from 0.3 requires the
[0.4 consumer changes](docs/changelog/0.4.md).

`read::directory_logical_size` sums metadata lengths without reading file content
or following final symlinks. Sparse files count logical bytes; callers retain
namespace custody, traversal resources and cache policy.

On Unix, `read::open_directory_no_follow` opens an explicit root capability;
`read::open_regular_file_beneath` selects a regular file relative to that borrowed
descriptor without following symlink components. `read::open_entry_beneath`
also admits a directory descriptor for a caller-owned tree walk. Use
`read_opened_file` to read a selected regular inode under an explicit byte
allowance. Root custody, content validation, deadlines and concurrent writers
remain caller-owned.

On Unix, `read::hash_file_no_follow(path, max_bytes)` hashes regular files with
constant working memory, rejecting final symlinks and opened special files.
It shares the bounded stream hasher and preserves typed errors. Callers retain
trusted ancestors, concurrent-writer custody and any exact expected-length check.

`durable::open_regular_lock_file_with_parents` prepares an admitted regular file
without acquiring a lock. Consumers then select `lock_shared`, `lock_exclusive`,
`try_lock_exclusive` or `lock_exclusive_with_wait` on its close-on-exec descriptor.
The immediate exclusive attempt returns native `WouldBlock` on contention and
performs no waiting or retry. Keep the descriptor alive through the protected
operation; callers retain polling, clone lifetime and explicit unlock policy.
Existing lock files need no staging or parent write permission; durable creation
runs only when the entry is missing.
The [0.13 migration](docs/changelog/0.13.md) replaces the fixed-cadence pathname
progress helper with this composition; callers keep polling and reporting policy.
The [0.23 migration](docs/changelog/0.23.md) removes the two combined pathname
open-and-lock shortcuts in favor of this explicit preparation/acquisition boundary.

`durable::open_private_regular_lock_file` instead creates a missing sidecar with
owner-only permissions in an existing parent. It preserves existing bytes, inode
and permissions, returns an unlocked close-on-exec descriptor, and performs no
durability syncs. Callers retain locking, trusted ancestors and descriptor custody.

`durable::write_named_with` lets an external tool write to an owned absolute
staging path with explicit `WriteOptions`, then shares the normal durable
publication engine. Callers validate bounded output before success and retain
typed producer/cleanup errors. See the
[current publication contract](docs/changelog/0.20.md#named-publication) before adopting it.

On Unix, `durable::write_non_aliasing(path, bytes, protected, symlink_limit)`
checks protected path/inode aliases before in-place truncation, then writes and
syncs the output. Existing inode/permissions and unrelated symlinks are preserved.
Callers select protected paths and own namespace custody and partial-output
recovery. This operation does not provide atomic replacement or synchronize
new parent-directory links; `validate_non_aliasing` performs observation only.

`durable::write_validated_with` closes the staging writer before a caller's
prepublication admission callback, enabling executable probes while preserving
the working destination on rejection. See the
[closed-writer contract](docs/changelog/0.5.md#closed-writer-executable-admission).

`durable::write_with(path, options, producer)` retains serializer errors while streaming through
the same engine, with explicit replace/create-only options and file permissions.
`durable::write_bytes(path, options, bytes)` shares that engine and returns
`NamedWriteError<io::Error>`, preserving publication state and separate cleanup
failures. Select mode and permissions explicitly; see the
[0.22 migration](docs/changelog/0.22.md).
`durable::copy_verified` composes a regular source, bounded streaming copy and
expected digest admission before publication. Callers select paths, the byte
allowance and `WriteOptions`; source symlinks are followed. Existing destination
bytes survive digest, source and limit failures.
`durable::copy_from` consumes an already-selected reader from its current
position. Callers retain input admission, byte/time bounds and choose `WriteOptions`.
Optional exact permissions are applied after copying; otherwise the creation
mode and umask apply. Failed reads preserve prior output through the same engine.
`durable::write_existing_parent_validated_with` refuses missing parents and opens
the selected parent once as a directory. Parent aliases are followed and the held
directory stays selected across renames. `durable::write_at_validated_with` borrows
an admitted directory descriptor without reopening its path. Both require an
explicit result-admission callback after file sync and before publication; use
`|()| Ok(())` for a unit result needing no further checks. Consumers retain
namespace custody, budgets and recovery. See the [0.25 migration](docs/changelog/0.25.md).

`path::canonicalize_allow_missing(path, base, max_symlink_depth)` resolves existing
components and dangling symlinks while normalizing a missing suffix. Choose the
simultaneously active missing-target depth explicitly; `usize::MAX` preserves
unlimited expansion. This supplies no root confinement or filesystem deadline.
See the [0.23 resolver migration](docs/changelog/0.23.md).

`path::validate_physical_directory_allow_missing(path)` checks existing components
without creating missing directories. Symlinks, non-directories and `..` reject.
It shares admission with private-parent preparation; callers retain pathname
policy and namespace custody through later operations.

With the artifact `gzip` feature, `copy_gzip` streams a single member into a
caller-owned sink and returns its decoded identity. Partial output can remain
after integrity, limit or sink failures; callers own staging and publication.
`hash_gzip` identifies decoded payloads
without a full decoded allocation. Callers select raw versus gzip handling
explicitly; raw streams use `hash_reader`. `gzip_matches` compares
decoded bytes exactly with an expected slice. All retain explicit input/payload
bounds and strict single-member integrity checks; see the
[gzip contract](docs/changelog/0.4.md#gzip-identities-and-exact-representation-comparison).

The five crates support Linux x86-64 and macOS 15 on Intel and Apple Silicon,
with Rust **1.88.0** as their MSRV. See [host qualification](docs/hosts.md) for
validation commands and native evidence requirements.

For development, prepare the toolchain selected in
[rust-toolchain.toml](rust-toolchain.toml) and the
[bootstrap prerequisites and shell PATH](docs/local-setup.md), then run:

```sh
make install-tools
make tools-check
make install-hooks
make test PACKAGE=ic-host-artifacts
```

Setup is explicit; `tools-check` verifies prepared tools offline. Hook installation
separately activates the repository-local formatter. Shared Tooling owns these
[common commands](docs/make-interface.md) and executable installation; this
workspace owns the five libraries, package checks and publication policy.

Use `make check`, `make clippy`, `make docs-check`, `make test` or `make msrv`
with `PACKAGE=<crate>` for focused work. Follow the
[shared validation policy](DRAGGINZGAME.md#validation-and-evidence) for when to
select those checks or full `make ci msrv-workspace` validation. The
[host guide](docs/hosts.md) describes feature-specific and hook checks;
`make help` lists the available commands.

Git release and crates.io publication are separate operations; a tag does not
establish registry availability. Read the [release and publication guide](docs/publishing.md)
for prerequisites, explicit command effects and retained-evidence recovery.
Consumers adopt and qualify registry selections independently.

Read [the extraction contract](docs/extraction.md), [the handoff](docs/status/current.md)
and [agent rules](AGENTS.md). Human and agent contributions follow the
[PR contribution rules](rules/contributions.md).

### Bounded Wasm inspection CLI

```sh
cargo run -p ic-host-wasm-inspect --locked -- input.wasm 1048576 1000 1000 1000
```

The standalone application composes artifact/filesystem owners, retaining the five
library boundaries without depending on Host Tools or the Candid parser. It prints exact SHA-256
and raw/structural facts after complete admission; budgets and interpretation
stay caller-owned. It executes no Wasm and estimates no cycles or instructions.
Select the published package, version and binary through the standard
Shared Tooling Cargo installer; no extra feature selection is required.
Append `--expect-sha256 SHA256` for digest verification before reporting, or
`--details` for export/custom-section records after the TSV summary. These options
replace the retired `ic-host-fs` `inspect_artifact` example; default CLI output
is unchanged. See the [inspector guide](apps/ic-host-wasm-inspect/README.md).
