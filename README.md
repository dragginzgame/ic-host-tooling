# IC Host Tooling

Host-side Rust libraries for artifact inspection, local filesystem safety,
admitted process execution and IC-specific format handling.

| Crate | Implemented source ownership |
| --- | --- |
| ic-host-artifacts | Bounded streams, SHA-256, gzip, verified tar members and generic Wasm facts |
| ic-host-fs | Regular/no-follow file reads, durable atomic publication, private files and descriptor locks |
| ic-host-process | Executable resolution, digest/version admission, bounded capture and Git observations |
| ic-host-tools | Candid extraction/normalization and ICP CLI response decoding |

These run locally outside canisters. The ic-host prefix identifies their ecosystem;
the generic crates have no IC runtime dependency. Compression, archive and Wasm
support are optional features of ic-host-artifacts. Filesystem/process consumers
select the small default artifact dependency.

This is an unpublished extraction workspace. Package metadata remains 0.2.0,
matching the imported library source; all packages currently set publish = false.
The pending 0.3.0 notes describe the breaking crate-boundary cut, not a release.
The original local checkout has been removed after a verified full backup.
Existing consumers retain their prior dependency selections.

Read [the extraction contract](docs/extraction.md), [host qualification](docs/hosts.md),
[the handoff](docs/status/current.md) and [agent rules](AGENTS.md).

Use cargo test -p ic-host-artifacts --all-features --lib --locked --offline
or select the corresponding filesystem, process or IC adapter package.
make check, make clippy and make docs-check accept PACKAGE=<crate>.
make test-artifacts-minimal exercises the artifact library without optional
features; native CI runs it alongside the all-feature package checks.
make install-host-tools and make install-ic-tools are explicit setup commands;
their offline checks never install tools. make install-hooks activates the
repository-local formatter after provisioning cargo-sort 2.1.4.

Shared Tooling owns rules, hooks, installers and the common release runner.
This workspace consumes an exact snapshot; library ownership stays here.
make ci is the complete configured gate, not an automatic development command.
The standard release entry points use the shared runner for version preparation,
commit, tag and atomic branch/tag push. Packages with publish = false can have Git
releases; registry publication remains a separate, disabled operation.
Release preflight prepares only pinned host tools. The IC executable bundle
remains available through make install-ic-tools and its offline check.
