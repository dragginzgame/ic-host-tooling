# IC Host Tooling

Host-side Rust libraries for artifact inspection, local filesystem safety,
admitted process execution and IC-specific format handling.

| Crate | Implemented source ownership |
| --- | --- |
| ic-host-artifacts | Bounded streams and hashing writers, exact byte comparison, SHA-256, gzip encoding/decoding, verified tar members and generic Wasm facts |
| ic-host-fs | Regular/no-follow reads, missing-suffix path resolution, streamed durable publication, private files and observed descriptor locks |
| ic-host-process | Executable resolution, digest/version admission, bounded capture, explicit child/group cleanup and Git observations |
| ic-host-tools | Candid extraction/normalization, ICP CLI response decoding and optional IC resource reports |

These run locally outside canisters. The ic-host prefix identifies their ecosystem;
the generic crates have no IC runtime dependency. Compression, archive and Wasm
support are optional features of ic-host-artifacts. Filesystem/process consumers
select the small default artifact dependency.
Response-only consumers can disable `ic-host-tools`' default features to exclude
the artifact/filesystem/process extraction dependencies. The default
`candid-extraction` feature preserves the existing Unix Candid API. Response
decoding is always available and needs only serde/serde_json.
The optional `ic-limits` feature compares Wasm facts with explicitly selected,
revision-bound IC resource limits, without filesystem/process dependencies when
defaults are disabled. See the [0.5 contract changes](docs/changelog/0.5.md).

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
`communicate_child` for an existing owner. It reports only retained bytes, keeps
the same hard limits and cleanup, and lets callers project progress events.
The cancellation callback can also emit caller-scheduled heartbeats while silent.
Callback unwinding attempts cleanup even when the borrowed owner survives the
caller's panic handler.

Both communication functions accept `CommunicationLimits { stdout_bytes,
stderr_bytes, timeout: None }` for long builds or foreground services without
an elapsed-time deadline. Output bounds, cancellation and cleanup still apply;
callers choose how to end the wait. Existing `OutputLimits` arguments keep their
finite deadlines. Capture and tool-version admission still require those finite
limits.

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

`durable::write_named_with` lets an external tool write to an owned absolute
staging path, then shares the normal durable publication engine. Callers validate
bounded output before success and retain typed producer/cleanup errors. See the
[named-output contract](docs/changelog/0.4.md#named-external-output) before adopting it.

`durable::write_validated_with` closes the staging writer before a caller's
prepublication admission callback, enabling executable probes while preserving
the working destination on rejection. See the
[closed-writer contract](docs/changelog/0.5.md#closed-writer-executable-admission).

`durable::write_typed_with` retains serializer errors while streaming through
the same engine, with explicit replace/create-only options and file permissions.
`durable::write_at_with` borrows an admitted directory descriptor so publication
stays anchored to it even when its original path moves. Consumers retain path
admission, budgets and recovery; see the
[publication contract](docs/changelog/0.4.md#typed-and-descriptor-relative-publication).

With the artifact `gzip` feature, `hash_gzip` and `hash_gzip_or_raw` identify
decoded payloads without a full decoded allocation. `gzip_matches` compares
decoded bytes exactly with an expected slice. All retain explicit input/payload
bounds and strict single-member integrity checks; see the
[gzip contract](docs/changelog/0.4.md#gzip-identities-and-exact-representation-comparison).

The four-crate workspace permits crates.io publication; a Git release does not
establish registry availability. Release qualification is recorded in the handoff.
The original local checkout has been removed after a verified full backup.
Consumers adopt and qualify registry selections independently.

Read [the extraction contract](docs/extraction.md), [host qualification](docs/hosts.md),
[the handoff](docs/status/current.md) and [agent rules](AGENTS.md).
Human and agent contributions follow the [PR contribution rules](rules/contributions.md).

Use cargo test -p ic-host-artifacts --all-features --lib --locked --offline
or select the corresponding filesystem, process or IC adapter package.
make check, make clippy and make docs-check accept PACKAGE=<crate>.
make test-artifacts-minimal exercises the artifact library without optional
features; native CI runs it alongside the all-feature package checks.
make test-tools-response checks the response-only library, and
make tools-features-check verifies its actual production dependency exclusions
and the default extractor's required owners.
make install-host-tools and make install-ic-tools are explicit setup commands;
their offline checks never install tools. make install-hooks activates the
repository-local formatter after provisioning cargo-sort 2.1.4.
The optional `make install-rust-tools` bundle has its own offline
`make rust-tools-check`; neither runs as part of the aggregate setup/check targets.
The common `make/tools.mk` owns setup/check commands and selects pinned jq, yq,
ripgrep with PCRE2 and cloc. CI uses those same targets. Run
`make install-host-tools` after this snapshot update to prepare the expanded set;
`make host-tools-check` verifies it offline without installing anything.
`make cloc` reports this workspace's Rust runtime/test lines. Run fleet tooling
reports from Shared Tooling; this snapshot omits that optional reporter.
Direct shell commands
need their own PATH setup as described in [local setup](docs/local-setup.md).

Shared Tooling owns rules, hooks, installers and the common release runner.
This workspace consumes an exact snapshot; library ownership stays here.
make ci is the complete configured gate, not an automatic development command.
The standard release entry points use the shared runner for version preparation,
commit, tag and atomic branch/tag push with `RELEASE_DELIVERY=direct`.
This consumer rejects other delivery selections before starting a command;
Shared Tooling's optional PR release flow needs separate adapter qualification.
Registry publication is separate:
make publish-check performs a Cargo dry run, and make publish uploads the
committed workspace to crates.io. Read [the publication procedure](docs/publishing.md)
for prerequisites, retained evidence and partial-upload recovery.
Prepare the pinned host set with make install-host-tools before release version
queries or release entry points. make release-version uses the shared read-only
Cargo/TOML reader; parser checks never install missing tools. Release preflight
refreshes only host tools. The IC executable bundle remains available through
make install-ic-tools and its offline check.
