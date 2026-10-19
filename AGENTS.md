# IC Build Agent Rules

Read [docs/status/current.md](docs/status/current.md), then
[DRAGGINZGAME.md](DRAGGINZGAME.md) and the
[ownership design](docs/design/build-ownership.md).
The baseline is Shared Tooling 0.25.0 revision
`2a79f5f93383f3d2a8a38d65406c7d52c2f77913`, recorded with exact digests in
[.shared-tooling.snapshot](.shared-tooling.snapshot).

- Mutate only this repository unless the maintainer expressly authorizes another
  target. Existing sibling repositories remain read-only.
- The maintainer authorized extracting generic code from IC Testkit and Canic.
  `crates/ic-build/` owns the published engine; the handoff records release identities.
  Do not claim completed consumer migration, native qualification or a release
  until the corresponding changes are applied and verified.
- Extract one implementation from its existing owners; do not copy a second
  production engine or leave compatibility reexports and parallel owners behind.
- IC Build owns generic build/artifact mechanics. Consumers own recipes, budgets,
  tool selections, credentials, source custody, release identities and lifecycle.
  Keep Canic release admission and Testkit's PocketIC lifecycle outside this crate.
- Use published IC Platform for required external IC-specific library capabilities;
  `make check-boundary` enforces the direct dependency policy. The current engine
  needs none. Native extractor execution preserves opaque evidence through Host
  and does not introduce a Candid parser, codec or canister runtime dependency.
- Preserve retained artifacts, errors, cancellation, input-change rejection,
  feature-safe batching and artifact lifetime. Builds have no finite default
  elapsed-time deadline. Maintenance and cleanup require explicit policy.
- For Rust implementation, use edition 2024 and MSRV 1.91.0, qualifying the
  selected dependency graph before advertising support. Use a virtual root
  workspace and `crates/ic-build/`.
  Do not add Python tooling.
- Linux x86-64 and native macOS 15 Intel/Apple Silicon are the initial required
  hosts. Linux or substitute checks do not establish native macOS qualification.
- Check active Cargo/rustc processes before edits or builds. Preserve dirty source
  and artifacts. The full release validation is `make check` (also `make ci`), plus
  `make lint check-secrets` on Linux with explicitly prepared validation tools.
  The gates cover formatting, dependency inheritance, compilation, rustdoc,
  behavioral/process tests and the actual declared MSRV. Preserve the selected
  graph and distinguish native artifact mechanics from synthetic Wasm producers.
  Keep the root Cargo.lock tracked under the shared rules.
- Follow [contribution rules](rules/contributions.md). Staging, commits, pushes,
  package versions, releases and publication are user-owned unless delegated.
- GitHub issues are the follow-up tracker. Keep the design and handoff as evidence
  and architecture; do not create local issue queues.

Local execution follows the adopted [shared validation policy](DRAGGINZGAME.md#validation-and-evidence).
Use focused package, script and caller checks for development and handoff;
the full commands above remain the release gate.
