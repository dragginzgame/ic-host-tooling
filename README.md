# IC Host Tooling

Host-side Rust libraries for artifact inspection, local filesystem safety,
admitted process execution and IC-specific format handling.

| Crate | Implemented source ownership |
| --- | --- |
| ic-host-artifacts | Bounded streams and hashing writers, exact byte comparison, SHA-256, gzip encoding/decoding, verified tar members and generic Wasm facts |
| ic-host-fs | Regular/no-follow reads, missing-suffix path resolution, streamed durable publication, private files and observed descriptor locks |
| ic-host-process | Executable resolution, digest/version admission, bounded caller-command capture and Git observations |
| ic-host-tools | Candid extraction/normalization and ICP CLI response decoding |

These run locally outside canisters. The ic-host prefix identifies their ecosystem;
the generic crates have no IC runtime dependency. Compression, archive and Wasm
support are optional features of ic-host-artifacts. Filesystem/process consumers
select the small default artifact dependency.
Response-only consumers can disable `ic-host-tools`' default features to exclude
the artifact/filesystem/process extraction dependencies. The default
`candid-extraction` feature preserves the existing Unix Candid API. Response
decoding is always available and needs only serde/serde_json.

`tool::capture_command` accepts a caller-configured `std::process::Command` when
the consumer owns executable admission. It shares bounded capture and direct-child
cleanup with `AdmittedTool`, whose exact digest/version checks remain in place.
`ToolError::evidence` and `execution_error` borrow retained diagnostics without
copying output; callers own presentation and recovery.

The four-crate workspace has a Git release at 0.3.2. Package metadata permits
crates.io publication; a Git release does not establish registry availability.
The original local checkout has been removed after a verified full backup.
Consumers adopt and qualify registry selections independently.

Read [the extraction contract](docs/extraction.md), [host qualification](docs/hosts.md),
[the handoff](docs/status/current.md) and [agent rules](AGENTS.md).

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

Shared Tooling owns rules, hooks, installers and the common release runner.
This workspace consumes an exact snapshot; library ownership stays here.
make ci is the complete configured gate, not an automatic development command.
The standard release entry points use the shared runner for version preparation,
commit, tag and atomic branch/tag push. Registry publication is separate:
make publish-check performs a Cargo dry run, and make publish uploads the
committed workspace to crates.io. Read [the publication procedure](docs/publishing.md)
for prerequisites, retained evidence and partial-upload recovery.
Prepare the pinned jq/yq set with make install-host-tools before release version
queries or release entry points. make release-version uses the shared read-only
Cargo/TOML reader; parser checks never install missing tools. Release preflight
refreshes only host tools. The IC executable bundle remains available through
make install-ic-tools and its offline check.
