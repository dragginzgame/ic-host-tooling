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

The [ownership design](docs/design/build-ownership.md) distinguishes current
owners from the historical extraction audit. The [artifact API guide](docs/artifacts.md)
owns detailed contracts and upgrade actions; the [current handoff](docs/status/current.md)
records source-bound qualification and outstanding consumer acceptance.

Batch label errors use `ic_host_artifacts::batch::BatchLabelError` in
0.18; replace imports from the retired `ic_build::batch` module.

## Use

The published 0.18 API requires Rust 1.91.0 or newer:

```toml
[dependencies]
ic-build = "0.18"
# Add matching direct Host dependencies when passing Host values through Build.
ic-host-process = { version = "0.27", default-features = false }
```

The examples use the current API and are compiled as Cargo example targets.
Pending 0.19 replaces public raw Cargo-child communication with typed
`CargoInvocation` / `execute_cargo`, including explicit compiler selection and
execution-bound tool observations. It is not published yet; see the
[migration contract](docs/artifacts.md#cargo-observations).
Wasm specifications require the complete environment at construction and an
explicit spawner at acquisition. See the [artifact guide](docs/artifacts.md) for
contracts and upgrade actions from earlier versions.

With the toolchain prepared and source writers excluded for the whole invocation:

```text
cargo run --locked --example wasm_build -- /checkout package
cargo run --locked --example native_executable -- /checkout package example-name
```

The [Wasm example](crates/ic-build/examples/wasm_build.rs) supplies an explicit
consumer-selected environment and keeps the returned record through byte
consumption. Paths alone do not retain cache entries. The
[native example](crates/ic-build/examples/native_executable.rs) selects a package
and example structurally, streams diagnostics and reports Cargo's executable path;
mutable target custody remains with the caller. Both select their Host spawner
explicitly and add no elapsed-time deadline. Applications supply their own
cancellation, process-group custody and source admission where required.

Align direct Host dependencies with the 0.27 family when exchanging Host values.
`cargo::discovery::declared_workspace_root_from` finds the nearest explicit
`[workspace]` from an existing file or directory without invoking Cargo. It follows
file symlinks and refuses invalid manifests, non-file markers and dangling links.
This is distinct from Cargo membership or `package.workspace` resolution through
`cargo::observation::workspace_root_for`; consumers retain no-match fallback.

Cargo owns compiler freshness; Build admits exact artifacts from Cargo messages
without filename inference. Native Candid extraction uses an admitted tool and
retains raw bounded evidence; consumers own normalization, grammar and publication.
The [deployable-Wasm LTO guidance](docs/artifacts.md#deployable-wasm-lto) records
measurements without imposing a shared application optimization profile.

## Development

Shared Tooling 0.25.0 is pinned in
[.shared-tooling.snapshot](.shared-tooling.snapshot); commands use this reviewed
local snapshot. The [Make interface](docs/make-interface.md) owns common setup,
formatting, hook installation and release entrypoints.

The virtual root [Cargo manifest](Cargo.toml) owns the workspace version, edition 2024,
MSRV 1.91.0, common metadata and lints. The
[ic-build package](crates/ic-build/Cargo.toml) inherits them and permits registry
publication through the explicit tagged-release command. The [library root](crates/ic-build/src/lib.rs)
exports the generic engine. The root [lockfile](Cargo.lock) records its selected
dependencies; none depend on Testkit, PocketIC or Canic.

```text
Cargo.toml                  # Virtual workspace and shared metadata
Cargo.lock                  # One selected graph
crates/ic-build/Cargo.toml   # Inherited package metadata; publish = true
crates/ic-build/src/lib.rs   # Generic engine modules
```

Prepare the declared development compiler and common tools explicitly:

```bash
rustup toolchain install --profile minimal --component rustfmt --target wasm32-unknown-unknown --no-self-update
make install-tools
cargo fetch --locked
make install-hooks
```

Release preparation and the isolated release-adapter checks use the pinned
published Build editor, prepared with `make install-release-tools`; see
[release setup](docs/publishing.md). Local commands select the prepared tool and
document allowance without manual exports. Common setup does not install that
optional tool. Global Cargo plugins are not setup prerequisites.

[rust-toolchain.toml](rust-toolchain.toml) selects Rust 1.99.1
development compiler, rustfmt, Clippy and its Wasm target independently of the qualified Linux
library MSRV. Explicit setup reads that target from the toolchain file. The complete
common host/Cargo toolset is prepared and checked on Linux x86-64. Native optimizer
qualification additionally requires explicit IC-tool setup and admission; ordinary
tests do not choose or install those executables. Existing IC installations remain
retained. Interactive shells use
the PATH selection in [local developer setup](docs/local-setup.md).
Fetch the selected workspace dependencies before the offline validation gate;
installing developer tools does not populate this project cache. CI performs the
same locked fetch on each host and retains its preparation log on failure.
`make test` resolves the repository-selected compiler and passes its absolute
path as `RUSTC` to the test command. Independent Cargo/Wasm fixtures therefore
use the prepared compiler even outside this repository's toolchain selection.

| Command | Purpose |
| --- | --- |
| `make help` | List the repository commands. |
| `make version` | Read the current workspace version. |
| `make tasks` | Read the shared maintenance catalog; scheduling remains explicit. |
| `make tools-check` | Verify the seven common host/Cargo tools offline. |
| `make fmt` / `make fmt-check` | Sort Cargo manifests and format / check all workspace Rust source. |
| `make install-hooks` | Activate the shared pre-commit formatter for selected staged files. |
| `make check-package` / `make docs-check` | Compile the engine and build rustdoc with warnings denied. |
| `make clippy` | Check the library, tests and examples with all features and warnings denied. |
| `make test` | Run behavioral, cross-process retention/locking and documentation tests. |
| `make msrv` | Compile the engine using the minimum compiler declared in Cargo.toml. |
| `make check` / `make ci` | Run the full gate: snapshot, tools, links, metadata, dependency/pin checks, formatting, compilation/Clippy/rustdoc/tests/MSRV and isolated release routing. |
| `make install-validation-tools` | Prepare pinned Linux x86-64 ShellCheck, actionlint and Gitleaks. |
| `make lint check-secrets` | Check shell/workflow sources and scan committed history for secrets. |
| `make check-release-adapters` | Test release metadata, recovery and publication routing in isolated fixtures. |
| `make check-boundary` | Enforce the direct Candid/CDK dependency ownership policy. |
| `make release-patch` / `make release-minor` / `make release-major` | Validate and deliver the selected release through Shared Tooling. |
| `make publish-check` / `make publish` | Dry-run / upload the clean, tagged release to crates.io. |

The full local delivery suite is `make ci` plus `make lint check-secrets` on
Linux after explicit tool setup. [CI](.github/workflows/ci.yml) runs lint/security
on Linux and the package gate on Linux x86-64, macOS 15 Apple Silicon and
macOS 15 Intel. It explicitly prepares the minimum compiler from Cargo.toml.
Failed native jobs use the shared evidence action. Native qualification is bound
to its exact source and graph; the handoff records that evidence separately from
local candidate checks.

Keep Cargo.lock tracked: the full shared pin checker enforces lockfile custody
as well as root/member dependency inheritance. Each dependency graph is checked
with its selected lockfile; validation never changes package versions.

The catalog labels production dependencies and test/tooling-only uses under
[Shared Tooling's role rules](rules/cargo-dependencies.md#dependency-roles-and-deployment-checks).
`ic-build` is the sole production package; every root dependency is used by its
normal or Unix-specific normal code. Development edges add Host `test-support`
features. Grouped `cargo sort --workspace --grouped` ordering applies in formatting hooks,
CI and release metadata checks, including checks of selected committed source.
The pin checker enforces selections and inheritance, not transitive role exclusions.
This repository publishes a host library and has no deployable canister or
deployment admission gate. Deployment consumers must check their own target/feature
graphs and exclude Build and other host/test tooling from canister artifacts.

IC Platform is the boundary for any Candid/CDK library capability. The published
0.1.3 archive is the reviewed baseline; Build currently uses none of its APIs and
adds no unused platform dependency. `artifacts::candid` runs a caller-admitted
extractor and retains opaque output; it does not parse or decode Candid or run a
CDK executor. Application protocols stay with IC Wire. `make check-boundary`,
also in CI/release validation, uses the shared checker and
[local policy](ci/cargo-dependency-boundaries.json) to reject named direct
Candid/parser/CDK dependencies, including aliases and normal/build/dev/target
declarations regardless of feature activation. The checker covers this workspace,
not transitive reachability, future unlisted family names or deployed Wasm bytes.
Inspect resolved graphs separately when dependencies or supported targets change.

`make cloc` reports workspace Rust LOC. The transferred tests execute real
filesystem and process boundaries, Cargo metadata and Wasm compilation, alongside
synthetic producer fixtures for failure and concurrency cases. The
[release and publication guide](docs/publishing.md) describes the consumer
adapters, exact tagged-source admission and retained recovery evidence.
The adapter fixtures qualify command behavior; they do not establish publication
or native macOS support. Published-version custody is recorded in the handoff.

The extracted v1 stamp format and `.ic-testkit` cache/lock names remain intact.
The 0.17 explicit-environment fingerprint domain causes cold builds when upgrading
from inherited-environment execution. Old retained entries remain available to their
owners; no migration reader or automatic cleanup is added. Returned records must
remain alive until their artifact paths have finished being consumed.

The [transactional transform example](crates/ic-build/examples/transactional_artifact_cache.rs)
composes a caller-selected, admitted tool with the generic cache; it chooses its
own recipe and maintenance policy. Follow [AGENTS.md](AGENTS.md) and the ownership
design for further convergence of Canic's complete-build mechanics.

Native optimizer qualification is owned here, with the transferred WAT and
JavaScript oracle under `crates/ic-build/tests/fixtures/wasm-optimizer/`. After
explicit tool setup/admission, prepare the Node version in `.node-version` and
the locked Cargo cache, then run:

```bash
optimizer="$(bash scripts/dev/install-ic-tools.sh --check)"
bash scripts/ci/qualify-wasm-opt.sh "$optimizer/wasm-opt" /absolute/new-evidence
```

The ignored integration target runs real `-O3`, `-Os` and `-Oz` transforms through
retained cache transactions, checks repaired materialization and reuse, and holds
the records through Wasm execution. The CI matrix explicitly runs this target on
Linux and both macOS hosts. Failures retain inputs, outputs and logs. Ordinary
test runs do not implicitly install or select an optimizer. Shared Tooling keeps
archive authentication and executable-installation checks.

## Coordination

The design owns architectural decisions, not issue status. GitHub issues in the
owning [GitHub repository](https://github.com/dragginzgame/ic-build) remain the
sole follow-up tracker.
Consumer acceptance remains separate from producer delivery. Follow the adopted
[contribution rules](rules/contributions.md) for isolated cross-repository work;
explicit task limits still apply.

The read-only [Cargo release candidate editor](docs/cargo-release-candidates.md)
provides `ic-build-release-metadata` and pure manifest/lock renderers. Consumers
prepare the executable explicitly and retain release policy and recovery.
