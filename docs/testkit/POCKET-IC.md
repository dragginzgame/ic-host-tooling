<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# PocketIC upstream boundary

This is maintained against Platform 0.1.16's selected `pocket-ic` 16.1 client and Testkit's
current public interfaces. Recheck the relevant capabilities when changing that
dependency. Historical decisions remain in the
[design records](README.md#historical-design-records) and
[qualification receipts](hosts.md#historical-qualification-receipts).
Those records do not define current APIs or imply that a proposed upstream change
has shipped.

## Ownership

PocketIC owns the simulator, management operations, client runtime types and
its default server discovery/download path. Platform owns the explicit native
client and DTO exports and their upstream dependency selection. Testkit forwards
that surface through `ic_testkit::pocket_ic`; the convenience exports in `pic`
preserve those same types. Testkit does not mirror the upstream simulator API.
Additional upstream types must be exposed by Platform rather than copied here.

Testkit owns explicit server provisioning and fixture lifecycle policy. It adds
bounded startup, server/instance custody, contextual calls, installation policy,
controller-aware snapshots, reusable baseline contracts, diagnostics and benchmark
markers. Application topology, reset coverage, funding amounts, byte/work budgets,
readiness predicates and performance thresholds remain caller-owned.

The [repository ownership map](overview.md#scope-boundaries) identifies the other
owners: IC Build supplies generic build/cache and package qualification mechanics;
Host supplies streams, filesystem and process primitives; IC Metrics supplies
measurement arithmetic. Those capabilities are not PocketIC workarounds and are
not reimplemented here.

## Server selection and provisioning

The upstream implicit builder may discover or download a server. Testkit's bounded
startup path requires an explicit selection instead:

- `server_assets::setup` and `ic-testkit-server setup` explicitly provision one
  release-bound, reviewed archive. Testkit owns the version/digests and bundle
  layout; Host supplies no-follow reads, gzip hashing/copying, durable publication
  and executable admission.
- `server_assets::check` and the CLI's `check` admit an existing bundle offline.
  They do not repair it or download a missing binary. Failed setup attempts and
  retained installations remain evidence.
- `PocketIcStartupConfig::Spawn` receives an already selected binary. Its caller
  owns byte admission and namespace custody. `from_env` gives an explicitly set
  `IC_TESTKIT_POCKET_IC_URL` precedence over `POCKET_IC_BIN`; binary mode performs
  a bounded version probe, not archive authentication.
- `PocketIcStartupConfig::Connect` borrows an existing server without claiming
  process ownership. It does not infer the remote server's executable identity.

`POCKET_IC_SERVER_VERSION` selects the provisioning release. The separate
`supports_pocket_ic_server` predicate admits stable 16.x probe output, matching
the client protocol range. The upstream `LATEST_SERVER_VERSION` is available as
an upstream fact; it does not silently choose Testkit's provisioning artifact.

An upstream resolver could replace part of this integration if its published
contract preserves explicit download policy, byte authentication, offline admission
and usable failure evidence. Testkit's release pins and caller-owned trust policy
would still need an explicit owner.

## Bounded startup and process lifetime

The selected upstream server launcher waits for port publication without Testkit's
complete readiness deadline and child-exit observation. Testkit's `build_instance`
and `build_instance_async` use the explicit configuration, preserve a typed
`PocketIcStartupFailure`, and return `PocketIcInstance<T>` with the upstream
instance and its server lifetime. Startup cancellation/failure cleans up the
owned child; borrowed-server failure leaves ownership with its caller.

`start_managed_server` exposes the same launcher without building an instance.
Its `PocketIcManagedServer` can support several independent instances through an
explicit URL. On Unix, Host's lifecycle engine keeps custody of the exact process
group through descendant termination and leader reaping. Caller-owned output files
remain available after failure; bounded excerpts and secondary cleanup errors are
available through the structured startup error.

The port path must be absent before spawning PocketIC. The launcher owns a private
directory and waits for a newline-terminated port while also checking child exit.
The server's default operation-idle lifetime remains distinct from an optional
hard lifetime; neither creates a default deadline for arbitrary caller commands.

`PocketIcStartupConfig::run_command` scopes a command to an owned or borrowed
server and supplies `IC_TESTKIT_POCKET_IC_URL`. The CLI's `run -- COMMAND` delegates
to that implementation. Cancellation or owned-server exit terminates the owned
command group and preserves failure evidence. External server processes remain
caller-owned. No process-global server registry or instance lock is involved.

Upstream typed, deadline-aware construction could retire local orchestration when
it supplies equivalent cancellation, custody and error contracts. The existing
synthetic and real instance-lifetime tests qualify those obligations separately
from ordinary simulator behavior.

### Bounded instance teardown

The construction deadline ends when startup returns. PocketIC 16.1's synchronous
instance destructor sends HTTP DELETE through a client without a request timeout;
its operation retry budget does not bound that wait. Testkit returns the upstream
instance and preserves this limitation. Catching a panic cannot bound a blocked
request. The nonblocking instance instead requires an explicit
`instance.drop().await` while the server remains available.

The controlled loopback test observes DELETE, confirms the child is still waiting,
then acknowledges it and verifies completion. The parent bounds its own waits and
owns child termination/reaping. It needs no PocketIC binary:

```bash
cargo test -p ic-testkit --locked --test pocket_ic_teardown \
  instance_drop_waits_for_the_deletion_response -- --exact --nocapture
```

### Temporary-directory ownership

Keep temporary directories available for the lifetime of any server using them.
The managed launcher retains its startup directory until its child has been
terminated and reaped. Borrowed or upstream background servers remain their
caller's responsibility; validation does not remove their files or discover and
terminate unrelated processes. CI retains the caller's temporary-directory
environment instead of adding a scratch lifetime shorter than those servers.
Earlier server-directory failure observations remain in the historical host
receipts; they are not fresh qualification of every selected server release.

## Calls, installation and diagnostics

PocketIC already has typed Candid helpers. Testkit's `query_candid_args` and
`update_candid_args` add explicit callers, finite reply-decoding budgets and
contextual encode/decode/rejection/transport errors. They delegate reply decoding
to `ic-platform` with its `candid` feature. Top-level reply arity stays explicit: `(T,)` requests one
reply, `((),)` one unit value, and `()` no reply values. Unrelated panic payloads
continue unwinding rather than becoming transport failures.

Some upstream operations still panic instead of returning structured failures.
Testkit owns `try_pocket_ic_operation` as the maintained boundary for recognized
synchronous transport panics. Candid calls use that same boundary; callers can
also apply it to raw operations without mirroring the simulator API. It returns
the original successful value, preserves upstream returned rejections and resumes
unrelated panic payloads. The closure runs once, with no automatic update replay.
Disconnects do not prove whether an update was accepted or an instance died.

The selected client's panic shapes are maintained here. Upstream has already
discarded the typed request cause, so this remains heuristic classification;
it cannot bound a blocked operation or catch asynchronous future polling. The
separate internal startup/install/diagnostic capture boundary intentionally
retains every panic for cleanup or collect-all behavior. Neither helper changes
the process panic hook. A supplying upstream API could eventually replace this
recognizer, but local support does not depend on that producer change.

### Resolver constraint

PocketIC 16.1.0 itself requires exactly `thiserror` 2.0.18, preserving that
selection in Testkit's native graph. The combined catalog uses a compatible
`2.0.18` requirement so unrelated Host-only consumers do not inherit a direct
PocketIC-specific exact pin. The lockfile still selects 2.0.18; no client patch
or dependency upgrade accompanies consolidation. Qualify the declaration with
`make dependency-pins-check` and the selected packages with
`cargo check -p ic-testkit -p ic-testkit-dev --locked`.

Routing the client through Platform retains that transitive requirement. Cargo
cannot combine it with an incompatible same-major requirement such as `^2.0.21`
in the same graph. Testkit's operation adapter cannot alter this constraint;
revisit the pin when deliberately changing the selected client. See
[#100 — PocketIC exact thiserror pin forces consumer lockfile downgrades](https://github.com/dragginzgame/ic-testkit/issues/100).

`CanisterInstallExt` preserves create, funding and installation failures. Retry
policy uses `ErrorCode::CanisterInstallCodeRateLimited` and caller-selected
attempt/cooldown values. Diagnostics are a separate explicit operation:
`PocketIcDiagnosticsExt` retains status, controller-aware logs and bounded
rendering. Installation does not silently fetch or print diagnostic logs.

Upstream error enrichment could reduce these adapters if it preserves context,
original rejects and bounded decoding. Merely adding another typed-call spelling
would not remove their current responsibilities.

## Snapshot baselines and reuse

PocketIC owns canister snapshots. Testkit captures/restores a declared snapshot
set and retains original rejection/operation causes. A baseline pool owns the
recipe and canonical restoration; recipes own non-snapshot reset work, readiness
and invariant checks. Reuse is admitted only when achieved reset coverage matches
requirements. Canister snapshots do not automatically restore PocketIC time,
extra canisters, pending work or external resources.

Each pool bounds its own leases and resource use. Independent PocketIC instances
are not serialized through a host-wide guard. Failed reuse disposes of the stale
slot and may rebuild once under the recipe's failure policy; caller panics still
unwind. These are test-fixture guarantees, separate from IC Build's persisted
artifact caches and application deployment recovery.

## Benchmarking and provenance

Testkit owns the `ICTK` marker format, span pairing and counter interpretation.
Canister-side `Performance` reads the system API through `ic0`; the private probe
uses CDK entrypoint macros to qualify actual marker emission and trap handling.
IC Metrics owns ratio/statistical arithmetic. The native report example and
private benchmark programs own output schemas, files and workload policy.

Callers that need runtime provenance should retain the exact selected path,
admission evidence and observed version. A borrowed URL does not establish those
facts. The private benchmark driver records its explicit server selection and
consumes IC Build's retained Wasm rather than reconstructing mutable build paths.

## Qualification and future retirement

Use the [current host matrix](hosts.md#current-focused-qualification) for
focused commands and declared native support. Source review and Linux execution
do not establish native macOS qualification.

Retire a local workaround only after checking the selected upstream implementation,
its actual consumers and success/failure/cancellation behavior. Keep supported
fixtures and caller policy unless an equivalent published interface gives them a
clearer owner. Historical proposals and similarly named functions alone are not
proof that the local obligation has expired.
