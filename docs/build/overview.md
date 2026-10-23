# IC Build

IC Build owns generic host-side Cargo/Wasm builds and retained artifact caches.
Consumers supply recipes, tools, complete environments, output budgets, source
custody and maintenance policy. Build never installs tools, starts PocketIC or
admits an application release.

The public entry points are:

- `ic_build::artifacts`: exact Cargo input resolution, Wasm acquisition,
  transactional artifact publication, retained records, staging/capture,
  prepared snapshots, batches, explicit maintenance and native Candid extraction.
- `ic_build::cargo`: typed selections, configuration/metadata observations,
  feature-safe package grouping, Cargo communication and exact artifact selection,
  native executable builds, package qualification and lock projection.
- `ic_build::batch`: labels shared by Build's batch and grouping APIs.

The dependency direction is consumer adapters → IC Build → Host Artifacts,
FS and Process. Host owns byte/hash primitives, filesystem admission/durable
writes/locks and child execution/framing/cleanup. Build composes those capabilities
with build and cache policy. Canic owns recipes, transforms and complete-release
admission; IcyDB owns optimizer/interface policy; Testkit owns testing and PocketIC
lifecycle. Build depends on none of those consumers. Shared Tooling owns the
reviewed development, installation and release helpers used by this checkout.

Release 0.18 removes `ArtifactStagingDirectory`; use published Host FS 0.27.5
fresh-directory allocation and `OwnedDirectory` directly for staging. Capture
records retain their artifact lifetime and explicit cleanup behavior.

The [ownership design](design/build-ownership.md) distinguishes current
owners from the historical extraction audit. The [artifact API guide](artifacts.md)
owns detailed contracts and upgrade actions; the [current handoff](status/current.md)
records source-bound qualification and outstanding consumer acceptance.

Batch label errors use `ic_host_artifacts::batch::BatchLabelError` in
0.18; replace imports from the retired `ic_build::batch` module.

## Use

IC Build requires Rust 1.91.0 or newer:

```toml
[dependencies]
ic-build = "0.66"
# Add matching direct Host dependencies when passing Host values through Build.
ic-host-process = { version = "0.66", default-features = false }
```

The coordinated Build 0.66 selects Host 0.66. Consumers passing Host values
through the execution/artifact APIs must align their direct Artifacts, FS and
Process dependencies to 0.66. The command/byte observation interfaces do not
exchange Host values. Defaults stay disabled; Build needs no Platform dependency.

The examples use the current API and are compiled as Cargo example targets.
Release 0.19 replaces public raw Cargo-child communication with typed
`CargoInvocation` / `execute_cargo`, including explicit compiler selection and
execution-bound tool observations; see the
[migration contract](artifacts.md#cargo-observations).
Wasm specifications require the complete environment at construction and an
explicit spawner at acquisition. See the [artifact guide](artifacts.md) for
contracts and upgrade actions from earlier versions.

With the toolchain prepared and source writers excluded for the whole invocation:

```text
cargo run --locked --example wasm_build -- /checkout package
cargo run --locked --example native_executable -- /checkout package example-name
```

The [Wasm example](../../crates/ic-build/examples/wasm_build.rs) supplies an explicit
consumer-selected environment and keeps the returned record through byte
consumption. Paths alone do not retain cache entries. The
[native example](../../crates/ic-build/examples/native_executable.rs) selects a package
and example structurally, streams diagnostics and reports Cargo's executable path;
mutable target custody remains with the caller. Both select their Host spawner
explicitly and add no elapsed-time deadline. Applications supply their own
cancellation, process-group custody and source admission where required.

Current source selects the published Host 0.31 family; match direct Host
dependencies when exchanging Host values with the execution/artifact engine.
Build 0.23 renders metadata/tree commands through each specification's
`command` method and exposes canonical decoding. Workspace discovery uses
`workspace_root_command` and `decode_workspace_root`; feature-safe grouping
accepts a tree observer. All these observations execute through the caller's
selected runner, preserving its Host generation, budgets and typed failures.
Admit successful, complete output before decoding.
Build uses neither Host FS's `capability` nor `temporary-file` features; both
remain disabled. Consumers using those optional APIs select them explicitly.
`cargo::discovery::declared_workspace_root_from` finds the nearest explicit
`[workspace]` from an existing file or directory without invoking Cargo. It follows
file symlinks and refuses invalid manifests, non-file markers and dangling links.
This is distinct from Cargo membership or `package.workspace` resolution through
`cargo::observation::workspace_root_command`; consumers retain no-match fallback.

Cargo owns compiler freshness; Build admits exact artifacts from Cargo messages
without filename inference. Native Candid extraction uses an admitted tool and
retains raw bounded evidence; consumers own normalization, grammar and publication.
The [deployable-Wasm LTO guidance](artifacts.md#deployable-wasm-lto) records
measurements without imposing a shared application optimization profile.

## Development

Build shares IC Tooling's [host qualification](../hosts.md) and
[release/publication flow](../publishing.md). For focused work use
`make check PACKAGE=ic-build`, `make test PACKAGE=ic-build`,
`make clippy PACKAGE=ic-build` and `make msrv PACKAGE=ic-build`.
The separate optimizer workflow retains real Wasm transformation qualification.

Use `make testkit-package-check` for coordinated archive qualification.
Consumer recipes, tool selections, budgets and lifecycle stay with their owners.
The [source inventory](../design/tooling-consolidation.md) records original
histories and the remaining external issues.
