<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

```toml
[dev-dependencies]
ic-testkit = "0.64"
ic-platform = { version = "0.6", features = ["pocket-ic"] }
```

For typed application calls, enable the `wire` feature and depend directly on
`ic-wire` for method contracts. Testkit does not own Candid codecs or call helpers.

The published MSRV is Rust 1.91, and the selected PocketIC line is 16.
Canisters using benchmark markers can add the crate under `[dependencies]`.
The Wasm library uses Platform's raw counters and marker logging,
without a canister SDK or Candid; `fake_principal` retains PocketIC’s principal identity through
`ic_platform::principal::ExternalPrincipal`, without enabling Candid on Wasm.
`pic` is host-only. `performance` is Wasm-only;
`benchmark` counters, parsing and aggregation, and `fake_principal` are available on both
targets. Native CSV and Markdown directory output lives in the repository's
`benchmark_report` example. Standalone recipe builders return
`std::io::Result<StandaloneCanisterFixture>`.

External interfaces that require Candid remain caller-owned: use a direct
Platform dependency for encoding/decoding and PocketIC's raw byte operations.
PocketIC retains its upstream Candid dependencies; Testkit's Wasm profile remains
Candid-free.

The optional native `wire` feature adds `pic::query_wire` and `pic::update_wire`.
Enable it on your Testkit dependency and declare `ic-wire = "0.6"` for your native
Serde contracts. Each helper takes an explicit target/caller, a `WireCallSpec`
with the exported name, method ID and request/response limits, and a borrowed
request. It executes one PocketIC operation without update retries; a refused reply
does not prove absence of effects. It preserves
application-level results. `WireCallError` retains typed codec causes, complete
structured rejections and the existing transport classification. Unrelated panic
payloads resume. Display omits payloads; raw calls remain available for malformed
requests and original-byte evidence. Reply admission follows PocketIC's allocation,
so it supplies no network pre-allocation or peak-heap guarantee. The feature is off
by default and adds no Wire dependency to the canister library graph.
Requests may use borrowed bounded fields. Responses must implement
`serde::Serialize` and `serde::de::DeserializeOwned` because Wire verifies
canonical bytes and the helper releases the received buffer before returning.
Call `query_wire::<Request, Response>(pic, canister, caller, spec, &request)` or
`update_wire` with the same arguments. The caller owns the Serde types and
per-direction limits; finite-codec budgets need requalification for native Serde.
See the [0.65 migration](../../docs/changelog/0.65.md).

This is the PocketIC package in the IC Tooling workspace. It provides:

- optional typed Wire query/update helpers with contextual, structured errors
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
- explicit diagnostic reporting to a caller-selected writer, containing secondary
  collection and output panics
- compact benchmark marker parsing, aggregation and comparison
- canister-side `performance::measure` marker emission

`ic-testkit` does not wrap the PocketIC simulator API or serialize independent
instances. `pic::server_assets` owns explicit server provisioning and offline
verification for Rust callers and the published CLI; tests never install implicitly.
Rust fixtures call `server_assets::check(&directory)` and pass the returned
absolute executable path to `PocketIcStartupConfig::Spawn`, without invoking the CLI.
Explicit `server_assets::setup` takes a caller-owned cancellation predicate.
Tests normally retain one
`PocketIcInstance<PocketIc>` and use `owned.instance()` for simulator operations;
focused extension traits provide reusable harness behavior. Import all upstream
client and DTO identities directly from `ic_platform::pocket_ic`; Testkit owns no
parallel import path. Platform owns additions to that explicit surface.
The client and managed server remain privately paired. Use `server()` for shared
server inspection and `into_parts()` only when taking over teardown ordering.
Wrap a client on an externally owned server with
`PocketIcInstance::from_borrowed_server(client)`.
See the [0.66 migration](../../docs/changelog/0.66.md).

```bash
cargo install --locked --features cli ic-testkit
ic-testkit-server setup
ic-testkit-server check
ic-testkit-server run -- cargo test --locked
```

The `cli` feature builds the server command; library-only consumers omit it.
For Shared Tooling's lockfile-selected installer, also pass `--features cli`:

```bash
bash scripts/dev/install-rust-tools.sh --consumer "$PWD" \
  --package ic-testkit --lockfile Cargo.lock --bin ic-testkit-server \
  --profile release --features cli
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
[repository README](https://github.com/dragginzgame/ic-tooling#readme) for
setup, examples, local checks, and release notes.
The [documentation index](https://github.com/dragginzgame/ic-tooling/blob/main/docs/testkit/README.md)
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
[multi-canister baseline recipe](https://github.com/dragginzgame/ic-tooling/blob/main/crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
compiled by the crate's normal all-target checks. The
[transactional external-artifact example](https://github.com/dragginzgame/ic-tooling/blob/main/crates/ic-build/examples/transactional_artifact_cache.rs)
is compiled and maintained by IC Build.

Benchmark aggregate/comparison rows and aggregate errors expose
`scope: BenchmarkScope::{Suite, All}` for their identity; `average(scale)` returns
checked floor-rounded `u128` means with Testkit's `BenchmarkAverageError`.
`compare_benchmark_aggregates(current, previous, scale)` returns
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
`query_wire` or `update_wire` with the `wire` feature. The standalone fixture owns
installation and lifetime; Wire owns method contracts and codec limits.
See the [packaged migration notes](CHANGELOG.md).

Benchmark commands own run identities, provenance, history selection and any
metadata sidecars. The repository's `benchmark_report` example renders supplied
measurements as CSV and Markdown; it has no metadata schema, reader or
history-discovery policy.

### Raw reply canister fixtures

Enable `ic-testkit`'s `reply-fixtures` feature to use
`pic::fixtures::raw_reply_canister`. Supply exact raw reply bytes and explicit
aggregate method-count, UTF-8 method-name and reply-byte ceilings. Empty or
control-containing names, duplicate exports and unsupported Wasm32 extents are
rejected before encoding. Method names must be unique across query and update exports.
The generated memory has a fixed maximum and can hold replies spanning pages.
This helper does not decode Candid, install canisters, select principals or fund
instances; keep product reply catalogs and assertions in the consuming tests.

### Held provider replies

Enable `held-reply-fixtures` for
`pic::fixtures::held::{HeldReplyFixture, HeldRequest, HeldReplyError}`.
Create and fund the provider canister, then construct `HeldReplyFixture::new`
with its principal, the PocketIC instance, an update-only `RawReplyMethod` catalog,
`RawReplyLimits`, cycles per barrier call and a compiled-module byte allowance.
Install `fixture.wasm().to_vec()` into that canister yourself.

Each catalog entry is a single-use request slot with immutable original reply
bytes. Repeated method names consume slots in catalog order; other methods can
arrive in any order. `limits.methods` bounds total lifetime calls, and the name
and reply allowances sum every slot. Exhaustion rejects instead of recycling
identities. Incoming application payloads are ignored; keep protocols, caller
authority and expected results in your tests.

Submit the consumer call with PocketIC's `submit_call`. Tick with your own finite
bound until `fixture.pending(&pic)?` reports the expected slot; `request.slot()`
identifies its original catalog position. Observation does not tick. Invoke the
cancellation/revocation endpoint and assert its committed state while the original
ingress is still outstanding. Call `fixture.release(&pic, &request)?`, then await
the original ingress and assert the stale callback's application-specific result.
Different slots can be released out of order. Tokens bind the exact HTTP request,
fixture, subnet, server and instance; unknown/repeated release refuses.

This uses PocketIC's supported manual HTTP mock facility as a barrier, without a
live provider or a second application engine. Use only manual progress: never
enable `auto_progress`/`make_live`, advance simulated time, or upgrade/reinstall/restore
the fixture with held calls. Constructor, observation and release refuse automatic
progress, but cannot prevent a caller enabling it independently. Automatic progress
can make actual HTTP requests. Keep time fixed to avoid the management HTTP
60-second timeout. Timeout/rejection fails the original call; it does not return
the configured reply. Never await a held call before releasing it.

Do not clone/reconstruct a controller over live calls. A transport failure during
release is uncertain and consumes its token; abandon the isolated instance rather
than retrying. Dropping the controller performs no cleanup or simulator operation;
the caller owns instance teardown and any diagnostic retention. No state survives
a fresh fixture installation. Native Platform management types/encoding adapt the
external Candid HTTP interface; application replies remain uninterpreted. Host's
WAT compiler is selected only by this feature. Deployed Wasm graphs contain none
of these native fixture, encoder, Host or Candid dependencies.

### Cycle-accepting provider replies

Enable `held-reply-fixtures` and supply an explicit `u128` acceptance cap:

- `pic::fixtures::cycle_reply_canister(methods, limits, accept_cycles, module_bytes)`
  generates immediate replies.
- `HeldReplyFixture::new_with_cycle_acceptance(pic, canister, methods, limits,
  call_cycles, accept_cycles, module_bytes)` adds acceptance to the existing held
  controller. `call_cycles` funds its HTTP barrier independently of acceptance.

Use update-only `RawReplyMethod` slots with caller-owned raw reply bytes and
`RawReplyLimits`. Each slot is one admitted invocation, including repeated method
names; the same cap applies independently to each slot. Actual IC acceptance is
`min(attached_available, accept_cycles)`. Zero explicitly accepts nothing; a cap
above the available amount accepts only what was attached. Exhaustion rejects
before acceptance. Construction and installation accept nothing. Existing
`raw_reply_canister` and `HeldReplyFixture::new` continue accepting no cycles.

Immediate acceptance commits with the reply. For held responses, acceptance
commits in the original update before the HTTP barrier is observable. Releasing
the slot only returns its original bytes; it never accepts more cycles. A later
HTTP timeout/rejection does not undo that earlier committed acceptance. A trap
before the barrier commits rolls back acceptance and leaves the slot unused.
Unaccepted cycles refund through the actual IC
callback; malformed/application-error reply bytes do not undo acceptance either.
Keep reply decoding, accounting, authorization, duplicate-payment prevention and
recovery policy in the consumer. The held lifecycle restrictions above still apply.

Native qualification reads actual callback refunds and both canister balances.
System-subnet tests isolate exact transfers from execution fees; a separate
application-subnet case proves the same refund path with normal fees. Gross
application-subnet balance deltas also include execution/transport/HTTP costs.
These fixtures never mint cycles or fund/dispatch a live provider automatically.
No new dependency is selected, and deployed Wasm graphs remain unchanged.

## Toolchains and checks

Testkit is part of the IC Tooling workspace. Use the root
[host guide](../../docs/hosts.md) and [publication guide](../../docs/publishing.md)
for setup and coordinated releases.

- `make testkit-check`: Testkit libraries and private driver with the prepared server.
- `make testkit-wasm-check` and `make testkit-msrv`: deployed profiles.
- `make testkit-package-check`: all coordinated public archives, including tests.
- `make testkit-browser-check`: browser-session support.

The [0.62 migration](../../docs/changelog/0.62.md) replaces the old independent
Testkit release flow. The root changelog owns new releases; the package changelog
retains published history. Neither package checking nor ordinary validation
authorizes downloads, publication or artifact cleanup.
