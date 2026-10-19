<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

```toml
[dev-dependencies]
ic-testkit = "0.55"
ic-platform = { version = "0.1.15", default-features = false, features = ["candid"] }
```

The published MSRV is Rust 1.91, and the selected PocketIC line is 16.
Canisters using benchmark markers can add the crate under `[dependencies]`.
The Wasm library uses Platform's raw counters and marker logging,
without a canister SDK or Candid; `Fake` uses the shared Principal type through `ic_platform::Principal`.
`pocket_ic` and `pic` are host-only. `performance` is Wasm-only;
`benchmark` counters, parsing and aggregation, and `Fake` are available on both
targets. Native CSV and Markdown directory output lives in the repository's
`benchmark_report` example. Candid calls require an explicit principal and caller-selected
`ic_platform::candid::DecodeLimits` and return contextual errors with typed decode
causes; add `ic-platform` with its `candid` feature directly to select those budgets. Choose caller-side
`expect` when a fixture must panic. Standalone recipe builders return
`std::io::Result<StandaloneCanisterFixture>`.

`ic-testkit-candid` is retired and is no longer released from this workspace.
For Testkit 0.54 and later, use `ic_platform::candid` with Platform's `candid`
feature. Upgrade Testkit and replace the decoder dependency together, preserving
caller-selected limits. Existing registry releases serve older Testkit consumers;
they are not a second maintained decoder implementation.

This is the PocketIC package in the `ic-testkit` workspace. It provides:

- direct re-exports of `PocketIc` and `PocketIcBuilder`
- Platform's explicit host-only client surface at `ic_testkit::pocket_ic`
- typed Candid query/update helpers with contextual, structured errors
- `try_pocket_ic_operation` for recognized synchronous transport panics, preserving
  upstream results and unrelated panics without automatic replay
- canister install and retry helpers
- explicit PocketIC startup, retained server/instance ownership, fallible async construction, and
  structured child/readiness failures
- explicit authenticated `ic-testkit-server setup`, offline `check`, and a shared
  environment startup contract with `ic-testkit-server run -- COMMAND`
  for suites spanning several test processes
- one recipe baseline pool for single- and multi-canister PocketIC reuse
- deterministic fake principals
- controller-aware, caller-labeled collect-all diagnostics
- compact benchmark marker parsing, aggregation and comparison
- canister-side `Performance::measure` marker emission

`ic-testkit` does not wrap the PocketIC simulator API or serialize independent
instances. `pic::server_assets` owns explicit server provisioning and offline
verification for Rust callers and the published CLI; tests never install implicitly.
Rust fixtures call `server_assets::check(&directory)` and pass the returned
absolute executable path to `PocketIcStartupConfig::Spawn`, without invoking the CLI.
Explicit `server_assets::setup` takes a caller-owned cancellation predicate.
Tests normally retain one
`PocketIcInstance<PocketIc>` and use `owned.instance` for simulator operations;
focused extension traits provide reusable harness behavior. Less common
upstream client and DTO identities come from `ic_platform::pocket_ic`, forwarded
at `ic_testkit::pocket_ic`. Platform owns additions to that explicit surface.

```bash
cargo install --locked ic-testkit
ic-testkit-server setup
ic-testkit-server check
ic-testkit-server run -- cargo test --locked
```

Setup selects Testkit's reviewed PocketIC 16.1.0 archive for Linux x86-64,
macOS Intel or macOS Apple Silicon. It authenticates the archive's pinned digest,
bounds decompression and admits executable bytes/version before publication.
Setup/check print the absolute executable path; diagnostics go to stderr.
`--directory DIRECTORY` selects a physical root (default
`.tools/ic-testkit-server` in the working directory). Checks and run never
download, update or repair a server. Failed attempts and previous bundles remain
retained; a changed installation requires a fresh selected root.

Run uses that prepared selection when neither `POCKET_IC_BIN` nor
`IC_TESTKIT_POCKET_IC_URL` is selected, and exports the executable and managed
URL to the command. Explicit binary overrides admit stable 16.x servers,
independently of the Rust client's latest download constant. URL mode borrows an
existing server. For long pre-client work, select `run --idle-ttl SECONDS -- COMMAND`
or the `Spawn` variant's `server_idle_ttl` field. This selects operation-idle
lifetime independently of `--ttl`'s hard limit; omitted idle selection retains
PocketIC's default. Both options require owned-server mode and positive seconds.

Most users should read the
[repository README](https://github.com/dragginzgame/ic-testkit#readme) for
setup, examples, local checks, and release notes.
The [documentation index](https://github.com/dragginzgame/ic-testkit/blob/main/docs/README.md)
links current usage guidance and historical design records.

Startup failures expose `PocketIcStartupError::failure()`
and `PocketIcStartupFailure` for cause matching. Read bounded server excerpts
through `output()`, and inspect typed secondary failures through
`command_cleanup()` and `server_cleanup()`. The original cause remains primary;
raw output files remain caller-owned.

Use direct `ic-host-artifacts`, `ic-host-fs` and `ic-host-process` dependencies
for generic artifact streams, filesystem operations and process custody. Testkit
selects gzip for its reviewed PocketIC server assets; its normal dependency graph
does not select Host's Wasm-inspection feature. Consumers select the capabilities
they use rather than inheriting them through Testkit.

The published archive includes [`CHANGELOG.md`](CHANGELOG.md) for historical
upgrade actions. Current build and artifact-cache contracts belong to IC Build;
historical Testkit artifact imports are not current entry points.

The repository also includes a complete
[multi-canister baseline recipe](https://github.com/dragginzgame/ic-testkit/blob/main/crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
compiled by the crate's normal all-target checks. The
[transactional external-artifact example](https://github.com/dragginzgame/ic-build/blob/main/crates/ic-build/examples/transactional_artifact_cache.rs)
is compiled and maintained by IC Build.

Benchmark aggregate/comparison rows and aggregate errors expose
`scope: BenchmarkScope::{Suite, All}` for their identity; `average(scale)` returns
checked floor-rounded `u128` means through
IC Metrics. `compare_benchmark_aggregates(current, previous, scale)` returns
current/previous means and run counts, preserving missing rows and measured zero.
Approximate percentage fields are removed. Aggregate CSV/text uses scale one in
original counter units; comparison CSV includes both scales. Exact totals remain
available, and zero runs/scaled overflow return typed errors. Write the changed
columns to new report directories; retained reports are not reinterpreted.
Baseline recipes construct `ResetRequirements { funding, policies }` with the
fixed optional fields in `ResetPolicies`. Their resets independently report
`ResetReceipt { policies }`; absence differs from an explicit irrelevant policy. The pool
restores the captured snapshots with the selected funding; successful manual
restoration returns unit. `ResetReceipt` covers non-snapshot guarantees. These are source API hard cuts.

Generic Cargo/Wasm builds, retained artifact caches and freshness APIs belong to
[IC Build](https://github.com/dragginzgame/ic-build). Depend on it directly and use
`ic_build::artifacts`; Testkit keeps PocketIC lifecycle and benchmark policy.

Pass `fixture.pocket_ic()` and `fixture.canister_id()` explicitly to
`query_candid_args` or `update_candid_args`. The standalone fixture owns installation
and lifetime; bounded argument-sequence calls own encoding, invocation and decoding.
Use `(T,)` for one reply and `()` for no replies; a single tuple reply uses `((T, U),)`.
See the [packaged migration notes](CHANGELOG.md#0530---2026-10-17).

Benchmark commands own run identities, provenance, history selection and any
metadata sidecars. The repository's `benchmark_report` example renders supplied
measurements as CSV and Markdown; it has no metadata schema, reader or
history-discovery policy.
