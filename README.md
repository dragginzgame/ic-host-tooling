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

The [ownership design](docs/design/build-ownership.md) distinguishes current
owners from the historical extraction audit. The [artifact API guide](docs/artifacts.md)
owns detailed contracts and upgrade actions; the [current handoff](docs/status/current.md)
records source-bound qualification and outstanding consumer acceptance.

## Use

The released dependency line is 0.16, requiring Rust 1.91.0 or newer:

```toml
[dependencies]
ic-build = "0.16"
# Add matching direct Host dependencies when passing Host values through Build.
ic-host-process = { version = "0.27", default-features = false }
```

The working tree targets pending 0.17; its examples use the pending API and are
compiled as Cargo example targets rather than duplicated in this README. Do not
apply them to the released 0.16 API unchanged. In particular, pending Wasm specs
require the complete environment at construction; the released silent build
entrypoint is retained. See the [upgrade guidance](docs/artifacts.md) for the full cut.

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
Cargo owns compiler freshness; Build admits exact artifacts from Cargo messages
without filename inference. Native Candid extraction uses an admitted tool and
retains raw bounded evidence; consumers own normalization, grammar and publication.
The [deployable-Wasm LTO guidance](docs/artifacts.md#deployable-wasm-lto) records
measurements without imposing a shared application optimization profile.

## Development

Shared Tooling 0.23.0 is pinned in
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

Release preparation uses the shared workspace-version renderer and does not
require an additional Cargo editor. Global Cargo plugins are not setup
prerequisites.

[rust-toolchain.toml](rust-toolchain.toml) selects the existing Rust 1.99.0
development compiler, rustfmt, Clippy and its Wasm target independently of the qualified Linux
library MSRV. Explicit setup reads that target from the toolchain file. The complete
common host/Cargo toolset is prepared and checked on Linux x86-64. This repo has
no installed IC-executable caller; IC setup remains an explicit opt-in through
the shared ordered local target lists when such a caller is added. Existing IC
installations remain retained. Interactive shells use
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

`make cloc` reports workspace Rust LOC. The transferred tests execute real
filesystem and process boundaries, Cargo metadata and Wasm compilation, alongside
synthetic producer fixtures for failure and concurrency cases. The
[release and publication guide](docs/publishing.md) describes the consumer
adapters, exact tagged-source admission and retained recovery evidence.
The adapter fixtures qualify command behavior; they do not establish publication
or native macOS support. Published-version custody is recorded in the handoff.

Testkit's v1 stamp format and `.ic-testkit` cache/lock names remain intact.
Release 0.16 uses typed Cargo metadata; pending 0.17 selects the explicit-
environment fingerprint domain, causing cold builds under the changed semantics. Old retained entries remain available to their
owners; no migration reader or automatic cleanup is added. Returned records must
remain alive until their artifact paths have finished being consumed.

The [transactional transform example](crates/ic-build/examples/transactional_artifact_cache.rs)
composes a caller-selected, admitted tool with the generic cache; it chooses its
own recipe and maintenance policy. Follow [AGENTS.md](AGENTS.md) and the ownership
design for further convergence of Canic's complete-build mechanics.

## Coordination

The design owns architectural decisions, not issue status. GitHub issues in the
owning [GitHub repository](https://github.com/dragginzgame/ic-build) remain the
sole follow-up tracker.
Consumer acceptance remains separate from producer delivery. Follow the adopted
[contribution rules](rules/contributions.md) for isolated cross-repository work;
explicit task limits still apply.
