# IC Host Tooling

Host-side Rust libraries for artifact inspection, local filesystem safety,
admitted process execution and IC-specific format handling.

| Crate | Implemented source ownership |
| --- | --- |
| ic-host-artifacts | Bounded streams and hashing writers, exact byte comparison, SHA-256, gzip encoding/decoding, verified tar members, generic Wasm facts and custom-section encoding |
| ic-host-fs | Regular/no-follow reads, missing-suffix path resolution, streamed durable publication, private files and observed descriptor locks |
| ic-host-process | Executable resolution, digest/version admission, bounded capture, explicit child/group cleanup and Git observations |
| ic-host-text | Pure terminal-safe escaping and Unicode display-width tables with explicit formatting choices |
| ic-host-tools | Candid text/extraction, optional service and IC export facts, ICP response decoding and IC resource reports |

These run locally outside canisters. The ic-host prefix identifies their ecosystem;
the generic crates have no IC runtime dependency. Compression, archive and Wasm
support are optional features of ic-host-artifacts. Filesystem/process consumers
select the small default artifact dependency.
For test fixtures, opt in to `ic-host-fs`'s `test-support` feature in a
dev-dependency and use `testing::create_fixture_directory(parent, prefix)`.
It allocates beneath an existing physical parent, skips occupied names and
returns the retained path. Callers own labels, custody and explicit cleanup;
the allocator supplies no Drop guard or application lifecycle.
With `wasm`, `wasm::encode_custom_section(CustomSection { name, data }, max_bytes)`
encodes one complete custom section with an explicit output-byte allowance.
Callers supply exact names/data and own module assembly, aggregate budgets,
duplicates, append order and subsequent Wasm validation.
`ic-host-tools` has no default features. Response decoding and
`candid::text::normalize` are always available without artifact/filesystem/process
dependencies or the Candid parser. Enable `candid-extraction` explicitly for
the Unix `candid::extraction` API.
The optional `ic-interface` feature adds `interface::exported_methods` over
generic Wasm facts without the Candid parser. Enable `candid-service` for
parser-backed `candid::service::inspect`; it also enables `ic-interface`.
Neither feature introduces Host filesystem/process dependencies on its own.
The parser remains absent from the extraction graph. See the
[0.17 feature migration](docs/changelog/0.17.md),
[0.16 extraction migration](docs/changelog/0.16.md) and
[0.15 API usage](docs/changelog/0.15.md).
The optional `ic-limits` feature compares Wasm facts with explicitly selected,
revision-bound IC resource limits, without filesystem/process dependencies when
defaults are disabled. See the [0.5 contract changes](docs/changelog/0.5.md).

The optional tools `certificate` feature exposes
`certificate::{decode_certificate, decode_hash_tree}` with explicit input-byte and
CBOR-depth limits. It returns owned `ic-certification` values and rejects trailing
data, malformed trees and invalid certificate fields. Parsing authenticates
nothing: callers retain Agent verification, trust roots, combined evidence
budgets, witness commitments and application leaf meanings. The feature adds no
Agent transport, filesystem/process execution or Candid parser.

`ic-host-text::escape::terminal_text` makes control and bidirectional formatting
characters visible without choosing output or colour. `table::render` accepts
headers, fixed-column rows, alignments and explicit `Options { column_gap,
header_separator }`, returning a String. It measures escaped cells in Unicode
display columns. Reports retain their row/header/column decisions; CLIs retain
stdout, terminal detection, colour and progress. The crate depends only on the
Unicode-width library, with no other Host or IC dependencies.

`ic-host-text::timestamp` formats unsigned Unix seconds and parses canonical UTC
second timestamps through `i64::MAX`, without clocks or timezone selection.
It preserves expanded positive years, strict spelling and impossible-date refusal;
callers retain freshness and collection-time policy.

The compiled [IC resource-report example](crates/ic-host-tools/examples/inspect_install_limits/main.rs)
inspects once and compares code-body bytes, defined functions and all globals
against the explicitly selected reference revision. Run it with:

```sh
cargo run --locked --offline -p ic-host-tools --no-default-features --features ic-limits \
  --example inspect_install_limits -- module.wasm 20000000 1000 10000 1000 15000000 --json
```

The numeric arguments select the hard raw-byte, section, export and custom-section
bounds, followed by a soft raw-byte warning threshold. Text and JSON share the
same failure exit decision; warnings alone do not reject. This is a composition
recipe, not complete Wasm validation or evidence of deployed subnet limits.

`tool::capture_command` accepts a caller-configured `std::process::Command` when
the consumer owns executable admission. It shares bounded capture and direct-child
cleanup with `AdmittedTool`, whose exact digest/version checks remain in place.
`ToolError::evidence` and `execution_error` borrow retained diagnostics without
copying output; callers own presentation and recovery.
Capture and communication return `CompletedExecution` with a required exit
status, including nonzero and signalled exits. Use `require_success()` when zero
exit is required. Interrupted execution retains optional status in `ExecutionEvidence`.

`child::OwnedChild::spawn` starts a caller-configured command in a new owned
process group. It preserves command IO and supports polling, waiting, explicit
termination and cleanup during unwinding. Callers retain admission, readiness,
cancellation and application lifecycle policy. For deliberate background startup,
`poll_exit` retains cleanup ownership through result admission; explicit `handoff`
then reaps a successful leader without stopping the background group. See the
[handoff contract](docs/changelog/0.5.md#explicit-successful-background-handoff) and the
[child cleanup contract](docs/changelog/0.5.md#explicit-child-and-process-group-cleanup)
for exclusive ownership and descendant limitations.

`spawn_with_cleanup` selects `CleanupPolicy::TermThenKill` for caller-owned TERM
grace and bounded reaping. Communication failures and Drop use that same policy;
the default remains immediate KILL with synchronous reaping. Reap timeout retains
the owner for explicit recovery and does not start a background reaper. See the
[bounded cleanup contract](docs/changelog/0.8.md#caller-selected-termination-timing).

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
the artifact/file compositions `inspect_archive`, `inspect_artifact`, `inspect_gzip`;
`ic-host-process` has `inspect_git` and `resolve_tool`. `ic-host-tools` retains
`extract_candid`, `inspect_install_limits` and `inspect_response`; the response
example runs with `--no-default-features`. Example-only dependencies do not widen
the tools crate's production feature graph.

`AdmittedTool::admit_version` accepts a `VersionSpec` for caller-trusted installed
tools without a published binary digest. It records the installed identity,
checks the exact version and rejects byte drift before later execution. That
observed identity is not authentication; installation provenance stays with the
caller. `admit` with `ToolSpec` continues to require an exact digest pin.

Filesystem reads live under `ic_host_fs::read`, including optional bounded
no-follow reads and typed private-file admission. Durable publication and locks
remain under `durable`. Upgrading from 0.3 requires the
[0.4 consumer changes](docs/changelog/0.4.md).

On Unix, `read::hash_file_no_follow(path, max_bytes)` hashes regular files with
constant working memory, rejecting final symlinks and opened special files.
It shares the bounded stream hasher and preserves typed errors. Callers retain
trusted ancestors, concurrent-writer custody and any exact expected-length check.

`durable::try_lock_regular_file_with_parents` shares regular-file admission and
durable creation with the blocking lock API, but returns a native `WouldBlock`
I/O cause on contention. It preserves existing bytes and holds the lock through
the returned close-on-exec descriptor. Callers retain retry and waiting policy.

`durable::open_regular_lock_file_with_parents` exposes that same admitted file
without acquiring a lock. Consumers can select shared locking or compose it with
`lock_exclusive_with_wait`, keeping wait timing and explicit unlock policy local.
Existing lock files need no staging or parent write permission; durable creation
runs only when the entry is missing.
The [0.13 migration](docs/changelog/0.13.md) replaces the fixed-cadence pathname
progress helper with this composition; callers keep polling and reporting policy.

`durable::open_private_regular_lock_file` instead creates a missing sidecar with
owner-only permissions in an existing parent. It preserves existing bytes, inode
and permissions, returns an unlocked close-on-exec descriptor, and performs no
durability syncs. Callers retain locking, trusted ancestors and descriptor custody.

`durable::write_named_with` lets an external tool write to an owned absolute
staging path, then shares the normal durable publication engine. Callers validate
bounded output before success and retain typed producer/cleanup errors. See the
[named-output contract](docs/changelog/0.4.md#named-external-output) before adopting it.

`durable::write_validated_with` closes the staging writer before a caller's
prepublication admission callback, enabling executable probes while preserving
the working destination on rejection. See the
[closed-writer contract](docs/changelog/0.5.md#closed-writer-executable-admission).

`durable::write_with(path, options, producer)` retains serializer errors while streaming through
the same engine, with explicit replace/create-only options and file permissions.
Byte writers also return `NamedWriteError<io::Error>`, preserving publication
state and separate cleanup failures. See the [0.10 migration](docs/changelog/0.10.md).
`durable::write_at_with` borrows an admitted directory descriptor so publication
stays anchored to it even when its original path moves. Consumers retain path
admission, budgets and recovery; see the
[publication contract](docs/changelog/0.4.md#typed-and-descriptor-relative-publication).

With the artifact `gzip` feature, `hash_gzip` and `hash_gzip_or_raw` identify
decoded payloads without a full decoded allocation. `gzip_matches` compares
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

The standalone application composes artifact/filesystem owners, keeping the five
library boundaries and response-only defaults unchanged. It prints exact SHA-256
and raw/structural facts after complete admission; budgets and interpretation
stay caller-owned. It executes no Wasm and estimates no cycles or instructions.
Select the published package, version and binary through the standard
Shared Tooling Cargo installer; no extra feature selection is required.
