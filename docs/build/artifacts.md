# Artifact and Wasm build APIs

Examples target the current source API. Standalone Wasm acquisition provides two
entrypoints: `build_wasm_canisters_cached(&spec, spawn)` for silent execution,
and `build_wasm_canisters_cached_with_progress(&spec, config, observer, spawn)`
for observation. Both use the same internal acquisition engine. Progress remains
independent of the explicit spawner's custody and stdin policy.

`WasmBuildSpec::new(workspace, target, packages, environment)` requires the
complete child environment. Consumers capture and select PATH, HOME/CARGO_HOME,
Rustup settings, compiler/wrapper variables, flags and application build inputs
before construction. Probes, metadata and builds clear inheritance and use that
same map. Omitted variables stay absent. `with_environment` replaces the whole
map; `with_extra_env` and `with_inherited_env` are removed without aliases.
Recapture at the consumer when ambient changes should select a different recipe.
The constructor selects initial Cargo/Rustc programs from the supplied CARGO/RUSTC
values, or literal cargo/rustc defaults; explicit tool builders remain available.
Keep CARGO_TARGET_DIR absent: Build supplies its selected output directory after
applying the recipe environment. Debug output lists environment names only.

The 0.17 API uses the `ic-build-explicit-environment-fingerprint` domain so
old inherited execution cannot authorize a cleared-environment cache hit.
Recipes build under new keys; retained entries, stamp formats and namespaces
stay intact without a migration reader or automatic cleanup.

Consuming a Wasm batch entry retains `(index, label, result, failure_details, elapsed)`.
Failure details remain paired internally and are exposed separately by this
released projection. No tuple migration is required in 0.17.

Native `target_kind` takes
`cargo::native::NativeTargetKind::Binary` or `NativeTargetKind::Example`, replacing
string selectors without an adapter. Replace raw Cargo recipe arguments with
`cargo::selection::CargoSelection` and `with_cargo_selection`; native
`NativeExecutableSpec` uses `selection` instead of `build_args`. Select a
profile, `cargo::selection::CargoFeatureSelection` and explicit lock/network flags.
The feature value owns `all_features`, explicit feature names and default-feature
policy. The old `cargo::observation::CargoFeatureSelection` path is removed;
observations and builds use the same encoder, preserving typed recipe argument
fingerprint bytes.
Cargo input resolution projects semantic package fields from Cargo's typed
schema, including conservative whole-workspace recipes. The old
raw/typed JSON pairing is removed: schema defaults and version normalization
apply, while unknown dependency-kind fields are no longer retained in identity.
Unsupported dependency-kind values refuse instead of sharing a normalized key.
Declared package metadata, enabled features, dependency aliases/kinds/platforms
and selected lock checksums remain identity inputs. Conservative mutation guards
still observe complete raw workspace manifests and lockfiles. Retained artifacts,
stamp formats and namespaces stay unchanged; this does not remove old entries.
Metadata and builds receive the same feature and network selection. There is no
arbitrary CLI argument escape hatch. Align direct Host dependencies with the
0.32 family for pending Build 0.24 (0.31 for published Build 0.23) when passing
Host values through Build's execution/artifact APIs. Command/byte observations
do not exchange Host values. Cargo progress
events carry `ic_host_process::tool::OutputStream` directly; remove imports of
`WasmBuildOutputStream`.

Progress heartbeats, command errors and `WasmBuildFailureDetails::phase()` use
one `artifacts::WasmBuildPhase`. Replace `WasmBuildProgressPhase` and
`WasmBuildFailurePhase` imports directly, without aliases. Rename
`ExactCacheLock` to `ExactCacheCoordination` and `SharedTargetLock` to
`SharedTargetCoordination`. Identity failures distinguish `CargoIdentity` and
`RustcIdentity`; the timing summary still adds both to `tool_identity`.
Specification and cleanup retain their own phases.

In 0.18, artifact batch functions return `ic_host_artifacts::batch::BatchLabelError`
directly; replace imports from the removed `ic_build::batch` module. The earlier
`ArtifactCacheBatchContractError` is also removed. Match `Empty { index }` or
`Duplicate { label, first_index, duplicate_index }`. Wasm batch contract errors
wrap those values in `WasmBuildBatchContractError::Labels`; its error source
exposes the same typed label failure. Source-lease invalidation and unprepared
specification failures remain distinct. Label rejection still precedes callbacks,
progress events, source validation and command effects. Cargo grouping returns
the same downcastable label error; its diagnostic wording now uses shared label
terms. Individual build, cleanup and publication failures remain in their entries.
Release 0.16 artifact failures use `ArtifactCacheBatchFailure::Preparation`,
`Callback` or `Commit` directly. Replace `Cache { phase, .. }`/`Build` matches and
remove `ArtifactCacheBatchFailurePhase`/`phase()` usage. `Callback` retains its
primary error and optional abort-cleanup failure; timing and error-source access
remain available for every variant.

Use `CacheRetentionLimits` instead of `ArtifactCachePrunePolicy` or
`SharedIncrementalTargetPrunePolicy`. The limits supply thresholds, while the
selected operation still owns removal: exact-entry pruning uses age/LRU and
skips live retained entries; shared-target maintenance clears the entire mutable
target except its root and IC Build coordination metadata. Keep unrelated data
outside that target. Maintenance still requires explicit policy and scheduling.

These generic APIs are extracted from IC Testkit. Recipes, budgets, tool
selection, credentials, source custody and lifecycle remain with the caller.
The existing v1 retained formats and namespaces are unchanged.

## Cargo observations

Pending 0.19 replaces public `cargo::execution::communicate_cargo` with
`CargoInvocation` and `execute_cargo`. Describe the complete environment, selected
Cargo executable, absolute working directory, graph selection, packages, manifest
and library/example/compiler recipe. Build renders the command from that single
selection. Explicit profile configurations and trailing compiler arguments remain
consumer recipes; no arbitrary Cargo argument parser is introduced. Observed
execution requires a nonempty `RUSTC` environment entry so configuration cannot
silently choose another compiler. Explicit environment removals stay removed.

`execute_cargo` observes Cargo and that compiler before spawning, drains stdout
once into canonical messages, then rechecks tools after success. Inspect
`execution` and its exit status first, then `messages`, then `facts`.
Facts are `Ok(Some(_))` only for successful, protocol-valid execution with matching
tool observations and an unchanged command. A failed process/protocol returns
`Ok(None)` facts; a post-build observation refusal is retained separately as an
error, alongside original raw execution evidence. These observations do not guard
source/configuration mutations, retain artifacts or admit a release.

The caller's spawner selects child/group custody and may override stdio. Changing
its executable, environment, arguments or cwd refuses successful facts after
draining. Identity probes use bounded complete output and caller-selected Host
group cleanup, share the cancellation predicate and have no finite deadline.
Build retention/deadline limits remain explicit. Protocol refusal drains the child;
output-sink failure uses Host cleanup. Source custody, build-script interpretation
and provenance redaction remain caller-owned. Cached builds use the same renderer,
tool observer and decoder with their existing phase/error mapping and complete
post-build input revalidation; they do not add redundant before/after probes.
`select_cdylib_wasm` selects only reported outputs for the admitted manifest;
capture or retain those bytes before another build can replace them.

Wasm builds, batches and prepared readers take a final Host spawner
argument. Pass `ic_host_process::child::OwnedChild::spawn_direct` for direct-child
ownership, or a closure calling `OwnedChild::spawn(command, cleanup)` for an owned
process group and explicit cleanup. Progress never chooses custody or cleanup.
Cargo starts with null stdin; the spawner may override it. Input probes delegate
capture to Host Process with their separate caller-selected policy; compilation
has no finite default deadline.

`cargo::observation::CargoMetadataSpec::command` renders onto a caller-selected
standard `Command`, retaining its executable, launcher arguments and complete
environment. The specification selects the manifest,
dependency inclusion, optional target filter, lock/offline policy and features.
No-dependency catalogs and resolved graphs are distinct observations; a catalog
or union graph does not prove role-specific dependency activation.

`CargoTreeSpec::command` renders caller-selected package ID specifications,
target, edge kinds, format and features. Cargo produces depth-prefixed,
non-deduplicated text; consumers own UTF-8 decoding and alias/role correlation.
Feature-safe grouping also uses this renderer. Pass the recipe's existing
`CargoSelection` after `target` to `cargo::compatible_package_groups`, followed by
a closure taking `&CargoTreeSpec` and returning admitted UTF-8 tree text. The
closure selects the command, applies the consumer environment, captures through
its runner and requires success then completeness before UTF-8 decoding. Build
retains isolated/combined probes and feature-equivalence checks. Every
isolated and combined probe uses its features, locked and offline choices;
the profile is not a Cargo tree argument. Canic's grouping adapter must pass
the same graph selection that its compilation recipe uses when adopting 0.17.
Build 0.23 removes the combined capture/decoding functions. Consumers
execute the rendered command through their selected Host, keeping its limits and
typed errors in that generation. Require success before completeness, preserving
failed exits as primary, then decode complete JSON through `decode_metadata` or
tree bytes through `String::from_utf8`. Keep the existing output budgets and
optional timeout; rendering neither executes nor adds a deadline. For example:

```rust,ignore
use ic_build::cargo::observation::decode_metadata;
use ic_host_process::tool::capture_command;

let mut command = metadata_spec.command(selected_cargo_command)?;
let output = capture_command(&mut command, selected_limits)?
    .require_success()?
    .require_complete()?;
let metadata = decode_metadata(&output.stdout)?;
```

The command/byte interface allows different Host generations for these callers.
Grouping forwards the observer's boxed errors unchanged; no Build-owned process
limit or error mirror is introduced. Execution/artifact APIs still exchange Host
values. Empty tree package selections are refused with
`io::ErrorKind::InvalidInput` before execution.

Wasm publication carries each cache path, public path and verified digest together.
All public copies finish before public stamps are written, including failure
paths. Warm repair preserves matching ordinary outputs and stamps. Cargo output
counts are admitted before any cache copy; this does not change stamp bytes,
cache keys, namespaces or retained-artifact lifetime.

`cargo::observation::schema` exposes the upstream `cargo_metadata` representation
and build-message types. Consumers use those types directly rather than retaining
local metadata mirrors. Metadata is decoded from complete JSON using that schema.
[Cargo's `build-script-executed` messages](https://doc.rust-lang.org/cargo/reference/external-tools.html#build-script-output)
identify the exact `package_id` and `out_dir`, including cached build-script
results. They can bind consumer-owned
retained export evidence; unlabelled target-directory `output` files cannot prove
that binding. Package/recipe selection, retained receipts, source custody and
before/after input validation remain consumer responsibilities.

## Deployable Wasm LTO

Full LTO is the leading candidate for a shared deployable-Wasm release default.
Qualify it against complete applications, then apply the selected policy in their
common build adapter. A library's own profile does not control its consumers:
[Cargo reads profiles from the workspace root](https://doc.rust-lang.org/cargo/reference/profiles.html#lto),
and full LTO optimizes across the linked dependency graph. There is no requirement
for every application to maintain a separate manifest edit.

The existing API can apply a shared release selection explicitly:

```rust,no_run
# let environment = std::collections::BTreeMap::<std::ffi::OsString, std::ffi::OsString>::new();
use ic_build::artifacts::WasmBuildSpec;
use ic_build::cargo::selection::CargoSelection;
use std::path::Path;

let workspace = Path::new("/application/workspace");
let target = workspace.join("target/canister-builds");
let mut environment = environment;
environment.insert("CARGO_PROFILE_RELEASE_LTO".into(), "fat".into());
let spec = WasmBuildSpec::new(workspace, &target, &["application_canister"], environment.clone())
    .with_cargo_selection(CargoSelection {
        profile: Some("release".into()),
        locked: true,
        offline: true,
        ..Default::default()
    });
```

For a custom profile, select its matching output directory and Cargo argument,
and use its corresponding `CARGO_PROFILE_<NAME>_LTO` variable. Keep other explicit
selections in the caller-owned map before construction. `with_environment`
replaces that complete map; it does not merge or read ambient values.
[Cargo's profile environment configuration](https://doc.rust-lang.org/cargo/reference/config.html#profilenamelto)
overrides the manifest. IC Build fingerprints explicit child environment and
workspace profiles, so thin and fat selections have distinct cache identities.
Prefer this declared selection over an untracked ambient override. The engine
continues to execute the caller's recipe; it does not silently change profiles.

### Initial whole-canister evidence

IC Timers measured its four linked size-probe canisters at released source
`44935d6188a0da82314e00a56f5f68a9d0c59638` (0.21.4), using Rust 1.88.0,
`wasm32-unknown-unknown`, and the same frozen Cargo.lock (Metrics 0.8.3). The
`timer-probe` profile inherited release with `opt-level = "z"`, `panic = "abort"`
and `strip = "symbols"`. Only LTO changed. All variants used the same source
path and locked/offline inputs. These are complete probe artifacts containing
CDK, Candid and other dependencies, rather than an isolated library size.

| Canister cohort | Thin LTO bytes | Full LTO bytes | Reduction |
| --- | ---: | ---: | ---: |
| Baseline | 264,414 | 214,565 | 18.85% |
| Once | 324,104 | 263,405 | 18.73% |
| After-completion | 324,974 | 264,288 | 18.67% |
| Watchdog | 325,763 | 264,670 | 18.75% |

Export names matched. No Wasm optimizer or PocketIC execution was performed;
instruction cost, heap use and comparable build-time changes remain unmeasured.
These small canisters establish a promising candidate, not the best setting for
every production app or compiler. The comparison excludes the later incoming
Metrics 0.8.4 selection. The source and four feature selections are maintained
in [IC Timers](https://github.com/dragginzgame/ic-timers/tree/44935d6188a0da82314e00a56f5f68a9d0c59638/testing/crates/ic-timers-size-probe).

### Qualifying a common release default

Compare thin and fat at the same application source, lockfile, compiler, target,
features and optimization recipe. Use the current profile's optimization level;
do not attribute simultaneous flag changes to LTO. Keep both retained outputs
alive through inspection. Apply the normal final transforms before comparing
deployable bytes, required exports, Candid and required Wasm features. Measure
representative instruction costs and clean/warm build time; run the application's
existing PocketIC acceptance against the selected final artifact.

If full LTO wins the representative application matrix within its accepted build
time and instruction budgets, adopt it once in the common deployable-Wasm adapter
and reuse it across applications. Keep development, native tools and deliberately
different product recipes scoped separately. Revisit the evidence after compiler
or optimizer changes. Build identity and execution remain in IC Build; product
acceptance and release publication remain with consumers. No new comparison
framework or public configuration type is needed for this initial qualification.

## Standalone input evidence and depfiles

`ResolvedCargoBuildInputs::facts()` borrows the `CargoBuildFacts` already used by
the Cargo input fingerprint: typed selection, target, complete selected environment
before Build adds its target directory, and successful Cargo/compiler identity
probe bytes (stdout followed by stderr). It performs no new probes or ambient
reads. Clones and prepared snapshots retain these immutable observations after
sources or tools change. They describe admitted inputs, not successful compilation;
consumers must bind them to the execution and retain source/target custody.
Debug omits environment values and probe bytes. Explicit access still exposes
potentially sensitive data; provenance publication and redaction belong to callers.

`WasmBuildRecord::facts()` carries those same fingerprint-bound observations through
built and reused outcomes, clones, batches and prepared readers. Reading facts
does not rerun tools or inspect ambient inputs. A reused record reports admitted
input observations, not a new compiler execution. Keep product provenance schemas
and redaction local, and retain the record through downstream artifact consumption.

Since 0.17.3, `WasmBuildEvidence::acquire(spec, spawn)` supports consumers that
retain a compiled artifact through measurement and export bounded copies. It
composes the existing resolved snapshot, retained build record and capture owner;
there is no second cache, recovery lifecycle or automatic report publication.
`verify_current` requires the acquired build's exact fingerprint to match the
snapshot and rechecks both semantic identity and conservative raw input identity.
Call it after measurement before accepting a report. Source custody remains the
consumer's responsibility; guards cannot detect edits reverted between checks.
`export_artifact(index, destination, maximum_bytes)` captures outside the retained
entry, preserves prior output on admission/capture refusal and returns identities
for the exact exported bytes. Retention failures preserve their existing recovery
paths. Retain the evidence owner through consumption; a receipt carries no lease.
The receipt reuses the digest computed while durably capturing the bytes. Export
still refuses an unreadable staging file but does not independently reread it
to check storage contents. Caller custody excludes concurrent stage mutation.

`cargo::diagnostics` selects bounded library/proc-macro artifacts from complete
Cargo output and runs a passing compiler control before negative cases. Each case
must exit with code 1 and contain every nonempty expected literal in complete
stderr. Consumers own compiler commands, cases, output budgets and retained log
directories. No retry, tool installation or elapsed-time deadline is introduced.

`InputEvidenceSpec` declares absolute file/directory/missing roots, an opaque
caller-derived non-file identity, generated-root exclusions and optional exact
child-name exclusions. It selects no recipes, tools, output roots or time limits.
`InputEvidence::capture` uses the same tree traversal as cache fingerprints,
records complete non-excluded directory membership, hashes regular file bytes
through Host's no-follow reader, and refuses child links or special files. Caller
custody must cover sources and parent directories through capture, work and
validation; evidence does not provide a lock or release admission.

`entries()` supplies per-path `Missing`, `Directory`, or raw SHA-256 `File` facts
for consumer-owned records. File hashes use Host's
`ic_host_artifacts::artifact::Sha256Digest`; use `Display`/`to_string()` for lowercase
hex text. `InputDigest` remains the derived inventory/cache identity.
`digest()` retains the existing consumer encoding:
identity followed by UTF-8 paths in lexical string order and their values, each
field prefixed by its u64 big-endian byte length. Values are `absent`, `directory`,
or lowercase file SHA-256. Capture preserves native paths; this particular digest
encoding refuses non-UTF-8 paths. Existing cache domains and field encoding remain
unchanged. Consumers keep their record formats and namespaces.

IC Build 0.7.0 and newer provide `digest_relative_to(root)` for descriptive source
fingerprints that survive checkout relocation. It projects the same captured facts using
sorted UTF-8 relative labels with `/` separators and the existing field framing.
The root must be absolute; parent traversal, outside inputs and non-UTF-8 relative
names refuse. Standard path-component normalization applies. Projection reads no
filesystem, so it can run after the captured sources have been removed. Consumers
select complete inventories, source custody and rebuild directives; a descriptive
fingerprint does not attest that an executable was built from those sources.
The original `digest()` and cache formats remain unchanged.

`let after = before.validate_after(&after_spec)?` captures and validates a
replacement or expanded inventory with the same identity and exclusion policy.
Pass an `InputEvidenceSpec`, rather than capturing it first. The operation observes
the union of original and selected roots, reusing overlapping facts, and returns
only the selected inventory with its existing digest encoding. It rechecks
every original root, even if a later depfile no longer names it, and rejects
changed bytes, absence or directory membership. Newly discovered bytes cannot
gain retrospective authority. A newly named missing path is admissible only when
an earlier complete directory scan proved its first child absent; excluded or
unobserved paths cannot establish that proof. The caller retains source custody through this fresh
observation and handles refusal according to
its own build/release policy.

`read_cargo_depfile_inputs(current_dir, record, excluded_roots,
maximum_record_bytes)` reads one exact record through a caller-sized bounded,
regular-file/no-follow read. Malformed, oversized or unsupported records refuse;
unrelated records in the output directory are not inspected. Relative inputs bind
to the explicit working directory. Existing aliases resolve before output
exclusion; missing paths containing parent traversal remain conservative external
inputs. The record itself is not a source input: Cargo may replace it during work.
Consumers select records from their own Cargo graph, combine their input sets,
and decide whether `NotFound` permits a cold-build inventory. Release 0.16 removes
directory-wide discovery; migrate callers to exact record selection. This
selection never grants pre-build authority to newly named source bytes; capture
and validate the resulting inventory through `InputEvidence` as above.

`parse_cargo_depfile` accepts Cargo/rustc Make dependency rules, escaped spaces,
tabs, backslashes, hash/colon characters, doubled dollars, LF/CRLF continuations,
multiple rules and empty phony targets. It refuses other Make expressions instead
of interpreting them or returning partial evidence. Full-line comments are
ignored, so environment dependencies belong in caller identity. Profile/target
directory selection and generated-output policy remain consumer-owned. These APIs
do not invoke Cargo, create another artifact cache or manage maintenance.

## Temporary staging and captured outputs

`artifacts::produce_and_retain_artifact` handles a caller-executed
transform with explicit producer, log and retained-destination paths and a byte
allowance. Build creates producer/log exclusively, passes the owned log to the
callback, then captures/retains successful output before removing the producer.
Failed statuses, callback errors/panics and capture/retention errors preserve
existing evidence. The log always remains. Callers retain tool/process policy,
physical path distinctness, parent custody and publication; the helper starts no
process, selects no deadline and removes no unrelated artifact.

The 0.18 cut removes `ArtifactStagingDirectory`. For direct staging, call
`ic_host_fs::directory::create_unique_directory` with an existing parent, prefix,
permissions and collision allowance, then retain the returned path in
`ic_host_fs::owned_directory::OwnedDirectory`. Callers choose candidate filenames
and qualify them before publication. Consuming `cleanup()` attempts removal once
and reports failures; Drop performs best-effort cleanup while armed. Callers keep
custody of the parent and staging paths against unrelated writers. Build's private
capture allocation retains mode 0777 reduced by umask and a 64-collision allowance.

`CapturedArtifact::capture(source, parent, maximum_bytes)` copies a completed
producer output synchronously with the caller's byte budget. Keep the returned
record through every use of `path()`, including later Cargo commands that may
overwrite or delete the original output. Multiple captures remain independent,
including captures of identical source bytes. Closing or dropping a capture
releases only its temporary staging directory.
Capture streams through Host's bounded copy and durable writer with constant
scratch memory. Regular-file and initial size admission happen before staging;
the stream independently enforces the budget. Keep the source stable throughout
capture. An open descriptor retains its inode rather than freezing its contents.
`retain()` explicitly transfers the captured path and its staging child to
caller-owned evidence retention. It preserves the bytes after the record is
consumed; the caller owns later cleanup and custody. It creates no cache or
release admission. Normal close/Drop behavior remains unchanged.

`retain_to(destination)` transfers custody before moving the exact capture with
`std::fs::rename` and removes only its empty staging child on success. The caller
admits the destination and owns replacement policy, subsequent custody and
cleanup. `CaptureRetentionError` preserves the native I/O cause, the actual
retained file path and the staging directory: rename refusal leaves the original
capture, while cleanup refusal leaves the final file and any extra evidence.
Neither failure retries or deletes retained bytes. The API is published since
0.3.3; Jobs owns its consumer adoption at
[#26 — Capture finalization](https://github.com/dragginzgame/ic-build/issues/26).

These temporary owners create no cache entry, cache hit or release admission.
They do not prune retained artifacts, run transforms, install tools or select
an elapsed-time deadline. Canic retains transform order, validation, release
binding, build coordination and durable product publication. Its producer and
consumer adoption evidence is tracked in
[Canic #529](https://github.com/dragginzgame/canic/issues/529).

## Caller-executed Cargo examples

`WasmBuildSpec::cargo_build_command()` prepares the same Cargo command used by
cached acquisition, using the specification's explicit target directory. It
starts no process or cache and executes no maintenance. Resolve the specification
first and call `ResolvedCargoBuildInputs::artifact_paths()` to admit its selected
Wasm output. Generic snapshots can describe non-Wasm packages, but this output
accessor refuses absent, non-cdylib, renamed or colliding targets.

For Query's probe, select package `ic-query`, example `governance_probe`, profile
`release`, and arguments `--release --no-default-features --features canister
--locked --offline`. Explicit compiler/tool/environment selection remains in the
specification. The command's `CARGO_TARGET_DIR` overrides a configured Cargo
output directory; discovered configuration still contributes to input identity.

Keep the existing caller executor: Query owns its 600-second deadline,
cancellation, TERM grace, bounded reaping and original command/output/cleanup
errors. For bounded input probes, use
`resolve_cargo_build_inputs_with_probe_policy(spec, &mut policy)` and
`inputs.is_current_with_probe_policy(spec, &mut policy)`. `CargoInputProbePolicy`
selects one absolute `Instant` deadline, cancellation predicate and Host cleanup
policy across Cargo identity, compiler identity and metadata. Reuse the policy
to keep the same deadline during revalidation. Host Process owns pre-spawn
cancellation/deadline refusal, process-group capture and cleanup through
`tool::capture_group_command_with_cancellation`; Build selects probe phases,
output limits and error context. Probes retain the existing
complete-output bounds (64 KiB per identity stream; 16 MiB metadata stdout and
64 KiB stderr). Original failures, observed output and cleanup errors remain in
the Host `ToolError` inside `WasmBuildError::Io`; the operation identifies the
probe and the path identifies its program. Discovery and hashing are outside
the probe deadline. The original resolver and `is_current` impose no deadline.

Preserve the generated command's arguments and environment. Hold source
and target custody, capture pre-build `InputEvidence` from resolved inputs and
exclusions, execute, re-resolve identity and validate before capturing outputs.
Discover caller-selected example depfiles and validate their expanded inventory;
a newly discovered external input needs pre-build authority. Predeclare it or
perform a fresh build after inventory discovery. Command success alone does not
admit retained bytes or a product release.

Query's receipt encoding remains caller-owned: sorted relative paths, NUL,
raw file bytes, NUL. Use shared inventory facts to select its existing explicit
files and Rust sources, then retain that encoder. `InputEvidence::digest()`
uses different framing and is not a replacement for Query's `source_sha256`.
Query also retains DID selection, public metadata, smoke/deployment lifecycle and
publication. Host owns generic custom-section framing. This capability does not
establish Query adoption or native smoke qualification.

## External lock selection admission

IC Build provides the pure
`ic_build::cargo::packages::verify_external_lock_subset(original, candidate, max_bytes)`
API for callers that need the archive engine's external-selection check without
running a package/test workflow. Both byte slices must fit the explicit
per-document allowance before UTF-8/TOML decoding. It performs no file or process
I/O, dependency resolution, network access or mutation.

Every candidate external name/version/source identity and optional checksum must
match the original. Registry checksums require canonical lowercase SHA-256;
non-registry optional checksum fields compare exactly, including absent versus
empty. Duplicate external identities refuse in either document. Extra valid
original selections and an empty candidate package array are allowed. Non-table
records and malformed identity/checksum fields refuse with typed role/index
evidence; UTF-8 and TOML failures retain their original error sources.

The archive qualification workflow is available only on Unix hosts, using Host's
descriptor-based source traversal. The in-memory subset verifier has no Unix gate.

Unsourced tables are outside this check. Consumers retain exact local package
names, versions, paths, dependency edges, forbidden-feature policy, input custody
and release admission. Cargo archive qualification uses this same reader and
comparison, then independently requires a nonempty external graph. Valid archive
flows retain their selected-graph, evidence and lifetime contracts; malformed
records and changed non-registry checksum fields now refuse there too.

Auth's existing `apps/tooling` command boundary can read both locks with its
explicit file budget and invoke this check, leaving its library-graph admission
in the existing script. Publish the producer before adopting that import; a
source rehearsal or temporary path override does not establish registry adoption.

## Strict Cargo lockfile version projections

IC Build 0.7.0 and newer provide
`cargo::lock_projection::verify_workspace_version_projection`.
Pass saved before/after TOML and explicit `WorkspaceVersionProjection` entries
containing each unsourced package name, expected prior version and permitted next
version. The verifier compares the complete parsed lockfiles after projecting only
those package versions. Comments and table-field ordering may differ; dependencies,
source/checksum fields, array order and all unrelated values must remain identical.
Invalid/duplicate selections, missing or ambiguous packages and unexpected prior
versions return typed `LockProjectionError` evidence.

Callers retain input byte limits, saved-file custody, version policy, rendering,
receipts and release ordering. This pure verifier reads and writes no files.
Memory's adoption is tracked by [#96](https://github.com/dragginzgame/ic-memory/issues/96).
Canic's fixture normalization also rewrites version-qualified dependency references;
that broader transformation is deliberately refused by this strict contract.

## Inherited Cargo Rust flags

Release 0.16 uses `cargo::flags::combined_encoded_rustflags(encoded, ordinary, &extra_flags)`.
Pass each additional argument as a separate `String`; when it returns `Some`,
set that value as `CARGO_ENCODED_RUSTFLAGS` on the Cargo command. An empty slice
returns `None`, so leave the command's inherited flags unchanged.

Supply `Option<&str>` values from the command's selected environment; no ambient
environment is read. Treat non-Unicode values as unavailable. Encoded flags take precedence
over ordinary `RUSTFLAGS`, including an explicitly empty encoded value. It splits
ordinary flags on whitespace and preserves each addition's argument boundaries.
Resolve explicit command overrides before inherited values. Callers retain
target/configuration precedence, remap roots and destinations, and build policy.
Canic can pass its source, Cargo-home and sysroot remaps as three additions rather
than replacing inherited flags with their encoded concatenation.

## Cargo configuration observations

Published 0.6.1 exposes the engine's configuration walker through
`cargo::environment`. `configuration_directories` enumerates invocation ancestors
and the explicitly supplied Cargo home without reading ambient environment.
`preferred_configuration` selects `config` before `config.toml`;
`observe_configurations` follows selected include trees and returns lookup paths,
stable locations, raw SHA-256 identities and compiler-override field names.
Release 0.16 stores `ConfigurationFile.sha256` as Host's `Sha256Digest`;
use `as_bytes()` for its unchanged raw bytes or `Display` for hexadecimal text.
Within one call, recorded lookup paths are not read or expanded again; distinct
aliases retain their own include lookup directories. A new call observes fresh
bytes under the caller's source-custody contract.

For Canic adoption, retain its source-custody and release recipe. Keep absent
candidate paths in pre-build `InputEvidence`; observing existing configuration
files does not establish authority for a newly discovered include. Choose
effective versus conservative roots explicitly and validate the expanded input
set after execution. Preserve typed refusal and retained evidence; no retry or
cache/release admission comes from observation. Configuration contents are
excluded from returned facts and malformed-TOML diagnostics. Cargo itself still
owns which configuration syntax its selected version accepts.

## Wasm artifact helpers

The host-only `artifacts` module provides content-addressed build coordination,
retained outputs and exact generated-artifact freshness checks. Consumers select
their own test target directories.

`cargo::observation::workspace_root_command` renders an offline `locate-project`
onto the caller's selected Cargo command, preserving launcher arguments, working
directory and environment, while disabling toolchain auto-installation. It
canonicalizes the supplied crate directory before rendering its manifest path.
Execute with the caller's existing capture budgets and admit success then
completeness before `decode_workspace_root`, which validates the absolute Cargo
manifest response and returns `io::Result<PathBuf>`. It supports App layouts,
exclusions and independent workspaces without guessing from directory names or
resolving dependencies. Consumers retaining an I/O facade should keep the original
typed Host error as its source: preserve the source I/O kind for spawn failures,
use `InvalidInput` for nonzero exits and `Other` for other capture refusals.
Malformed workspace responses remain `InvalidData`. No compatibility discovery
executor remains in Build.
`WasmBuildSpec::new` shares `<workspace>/target/ic-testkit-incremental` compiler
state across fingerprints by default. Exact final artifacts remain independently
cached. Select `with_isolated_builds()` only when separate compiler state is
required; configure age/byte retention through the existing pruning and
shared-target maintenance builders. Existing isolated build directories are
preserved and need explicit maintenance if no longer wanted.

```rust,no_run
# let environment = std::collections::BTreeMap::<std::ffi::OsString, std::ffi::OsString>::new();
use ic_host_process::child::OwnedChild;
use ic_build::artifacts::{
    CacheRetentionLimits, WasmBuildOutcome, WasmBuildSpec,
    build_wasm_canisters_cached,
};
use ic_build::cargo::{
    observation::{workspace_root_command, decode_workspace_root},
    selection::CargoSelection,
};
use ic_host_process::tool::{OutputLimit, OutputLimits, capture_command};
use std::{process::Command, time::Duration};

let mut command = workspace_root_command(
    env!("CARGO_MANIFEST_DIR"),
    Command::new("cargo"),
)?;
let output = capture_command(
    &mut command,
    OutputLimits {
        stdout: OutputLimit::Terminate(64 * 1024),
        stderr: OutputLimit::Terminate(64 * 1024),
        timeout: None,
    },
)?
    .require_success()?
    .require_complete()?;
let workspace = decode_workspace_root(&output.stdout)?;
let target = workspace.join("target").join("pic-wasm");

let spec = WasmBuildSpec::new(
    &workspace,
    &target,
    &["counter_canister"], environment.clone())
.with_cargo_selection(CargoSelection {
    profile: Some("release".into()),
    locked: true,
    ..Default::default()
})
.with_additional_inputs(["config/counter-schema.json"])
.with_prune_policy(
    CacheRetentionLimits::new()
        .with_max_age(Duration::from_secs(14 * 24 * 60 * 60))
        .with_max_size_bytes(10 * 1024 * 1024 * 1024),
);

let outcome = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)?;
eprintln!("immutable cache: {}", outcome.record().exact_cache_path().display());
match &outcome {
    WasmBuildOutcome::Built(record) => {
        eprintln!("built {} in {:?}", record.fingerprint(), record.timings().total());
    }
    WasmBuildOutcome::Reused(record) => {
        eprintln!("reused {}", record.fingerprint());
    }
}

// Keep the outcome alive while reading its retained, read-only artifact.
let wasm = std::fs::read(&outcome.record().artifacts()[0])?;

if let Some(pruned) = outcome
    .record()
    .maintenance()
    .and_then(|maintenance| maintenance.prune_report())
{
    eprintln!(
        "removed {} builds ({} bytes)",
        pruned.entries_removed(),
        pruned.bytes_removed(),
    );
}
# let _ = wasm;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The fingerprint covers the selected package set and resolved dependency nodes,
enabled features, exact external source/checksum/revision identities, local
dependency sources, relevant inherited package fields, workspace profiles,
resolver and lint settings, Cargo configuration, Rust toolchain files, target
and profile arguments, Cargo and rustc identities, explicit child environment,
the complete explicit environment, and caller-declared additional inputs. This
validated semantic workspace projection lets an unrelated host-only workspace
dependency or lockfile change retain the same exact Wasm key.

For a Cargo shim that selects another compiler, pair `with_cargo_program(shim)`
with `with_rustc_identity_program(actual_compiler)` so the `-vV` probe fingerprints that
compiler. The latter selects identity only; it does not configure the shim's
build. When Cargo should use a selected compiler, use
`RUSTC` in the complete environment, which also takes precedence for
the identity probe. Keep the selected compiler consistent with the build command.
The [compiled post-link recipe](../../crates/ic-build/src/artifacts/mod.rs) combines
this spec with `ArtifactCacheSpec::with_cargo_build_inputs`, declared input/tool
bytes and bounded optimizer execution. It retains the source acquisition through
commit and reads the retained transformed output. The
[external transform example](../../crates/ic-build/examples/transactional_artifact_cache.rs)
also works for inputs supplied independently of a Wasm build.

The complete workspace manifest and lockfile remain part of a separate,
conservative validation digest. They are rehashed around builds and attached
external artifact transactions, so any raw workspace mutation during an
acquisition is rejected even when its semantic projection is unchanged.
`ResolvedCargoBuildInputs` exposes both `input_digest` (semantic cache identity)
and `validation_digest` (mutation guard). Projection falls back to the complete
manifest/lockfile identity when a selected local package is rooted at or cannot
be normalized beneath the workspace root. Cargo configuration discovery follows
the files Cargo can read from the invocation directory and its ancestors plus
the effective Cargo home. The extensionless `.cargo/config` takes precedence
when both supported names exist, and recursive required or optional `include`
entries are exact inputs. Build scripts that read application-specific
environment variables or files outside Cargo's package graph must still declare
them on the spec. Package inputs are content-hashed exactly but conservatively;
exact hashing does not promise that every file in the package closure is
semantically relevant to a build.

Calls sharing a target directory coordinate through one process lock. A cache
hit requires every expected nonempty Wasm artifact to carry the exact atomic
stamp for both the current fingerprint and artifact content. A missing or invalid
exact entry requires a new Cargo invocation even if matching public outputs
survive. Public outputs never reconstruct an exact entry; live acquisition records
still prevent replacement of retained entries. Exact builds are
retained under fingerprint-specific Cargo target directories, so a prior spec
can be materialized again after another spec used the caller-facing output.
`WasmBuildRecord::exact_cache_path` exposes the selected immutable directory;
CI cache collection does not need to reconstruct its private on-disk layout.
Both `WasmBuildRecord::artifacts()` and `ArtifactCacheRecord::artifacts()` return
read-only exact-cache paths retained by the record. Keep the outcome, batch
report, or a cloned record alive through post-link commit and final reading or
staging. A copied path or artifact descriptor alone does not retain ownership.
Configured compiler and post-link destinations remain mutable. Never transform
an exact artifact in place; write into the next transaction's staging outputs.
See the `artifacts` module documentation for a complete handoff example.

Pruning skips all retained entries, including those owned by another process.
The last record clone dropping or its process exiting releases retention; the
next maintenance pass can reclaim the entry. Cache bounds can be temporarily
exceeded while consumers hold records. Batch successes remain independently
retained when later entries fail, so keep their records instead of reducing a
report to indexes. Manual deletion or mutation of exact-cache files is outside
this contract.

Failed builds remove their incomplete fingerprint directory before returning;
if cleanup also fails, `WasmBuildError::FailedBuildCleanup` preserves both
failures. Every used target root receives a standards-compliant
`CACHEDIR.TAG`. Successful builds and cache hits atomically update a per-entry
last-use marker. `prune_wasm_build_cache` takes the build lock, removes entries
over the optional age limit, then removes least-recently-used entries until the
optional logical-byte limit is met. It only prunes direct fingerprint-named
children below `.ic-testkit/wasm-targets`; public artifacts and unrelated Cargo
target contents remain caller-owned. `WasmBuildSpec::with_prune_policy` performs
the same maintenance without reacquiring the lock, protects the active
fingerprint even when it exceeds the configured bound, and reports a nonfatal
`ArtifactCacheMaintenance` result plus its duration on the successful build
record. The standalone pruning function remains available when maintenance
failure should be returned directly.

High-frequency suites can use `with_prune_policy_at_most_every` to retain the
same limits while replacing repeated hit-path directory scans with a small
namespace marker check. The original `with_prune_policy` continues to run on
every successful acquisition. An explicit zero interval has the same behavior:
it skips the due-marker read and still records the maintenance attempt. Both
always-run forms return `None` from `WasmBuildSpec::prune_interval()`; inspect
`prune_policy()` to distinguish disabled maintenance. Positive intervals retain
their marker checks and nonfatal read-error evidence. Thresholds, marker bytes,
retained-entry protection and attempt-based scheduling are unchanged.

`WasmBuildTimings::input_resolution` returns the structured Cargo/rustc
identity, Cargo metadata, input discovery, content hashing, and total timing.

Relative exact target paths use the caller's working directory, matching cache
filesystem operations. Relative shared targets use the workspace root. Select
these paths through `WasmBuildSpec`; overriding `CARGO_TARGET_DIR` through
the complete environment returns `InvalidSpec` before acquisition.

The profile output directory follows `CargoSelection::profile`: default builds,
`dev` and `test` use `debug`; `release` and `bench` use `release`;
custom profiles use their own name. Unsafe profile paths fail before resolution
or cache work. Selection rendering is deterministic; equivalent old arguments in
a different spelling/order may select a new fingerprint. Existing cache namespaces,
retained artifacts and stamp formats are preserved; no old entries are deleted.
Default recipes and canonical selections such as `--release --locked` keep their
argument fingerprint bytes. Custom profiles render as `--profile=NAME` and
explicit features as one sorted `--features=FEATURES` argument. Consumers own
any local build parallelism or environment settings outside this selection.

Wasm acquisition builds package libraries with Cargo's `--lib` flag by default.
Use `with_example(name)` to select one named `cdylib` example from exactly one
package. Metadata admits that target before acquisition; its name contributes to
the fingerprint and selects `<target>/<profile>/examples/<library_name>.wasm`,
with hyphens normalized to underscores in the filename. Default-library cache
identity and output layout remain unchanged.

Raw binary, example, test, bench, and `--all-targets` arguments return
`InvalidSpec` before input resolution or cache work. Target selection belongs to
the specification; other targets can overwrite the canister library's output.
An explicit `--lib` remains accepted only for the default-library selection.

Acquired packages must declare a `cdylib` library with Cargo's default library
name: package hyphens become underscores. That admitted name selects every cached
and published Wasm path; packages whose default names collide are refused.
Acquisition validates this declaration from Cargo metadata before reuse, compilation, or scheduled
shared-target maintenance. Libraries that only produce an `rlib`, renamed
libraries, and binary-only packages return `InvalidSpec`. Input-only snapshots
from `resolve_cargo_build_inputs` can still describe arbitrary Cargo targets
for transactional artifact recipes.

Suites can override the default workspace shared Cargo target while retaining
exact immutable final Wasm entries:

```rust,no_run
# let environment = std::collections::BTreeMap::<std::ffi::OsString, std::ffi::OsString>::new();
use ic_host_process::child::OwnedChild;
use ic_build::artifacts::{
    CacheRetentionLimits, WasmBuildSpec, build_wasm_canisters_cached,
    inspect_shared_incremental_target,
};

# let workspace = std::path::PathBuf::from(".");
let exact_cache = workspace.join("target/pic-wasm");
let shared_incremental = workspace.join("target/pic-wasm-incremental");
let spec = WasmBuildSpec::new(
    &workspace,
    &exact_cache,
    &["counter_canister"], environment.clone())
.with_shared_incremental_target(&shared_incremental)
.with_shared_incremental_target_maintenance_at_most_every(
    CacheRetentionLimits::new()
        .with_max_age(std::time::Duration::from_secs(7 * 24 * 60 * 60))
        .with_max_size_bytes(4 * 1024 * 1024 * 1024),
    std::time::Duration::from_secs(24 * 60 * 60),
);
let outcome = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)?;
let shared_usage = inspect_shared_incremental_target(&spec)?
    .expect("configured acquisition creates the shared target");
let maintenance = outcome
    .record()
    .shared_incremental_maintenance()
    .expect("shared maintenance was configured");
eprintln!(
    "shared_bytes={} maintenance={} {}",
    shared_usage.logical_size_bytes(),
    maintenance,
    outcome,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Without integrated shared-target maintenance, an exact hit never acquires or
mutates the shared target. On a miss, callers using that target serialize
across processes; Cargo builds incrementally, then IC Build re-resolves
every exact input and publishes only the final Wasm files. A source race
rejects publication, while a failed build preserves the caller-owned
incremental state. Retention owns only immutable output entries, not the
shared Cargo target. `inspect_shared_incremental_target` takes the same
cross-process lock and reports its canonical path, logical size, last recorded
build use, and lock wait without removing caller-owned Cargo state.
`maintain_shared_incremental_target` is an explicit whole-target operation: if
an age or size threshold is exceeded it removes every other child while
retaining the root, cache tag, and live coordination lock. Callers must not
colocate unrelated data that needs to survive a clear. Before clearing, the
exact Cargo resolver rejects a target that overlaps source, configuration, or
additional inputs, or whose clearing could reach exact cached artifacts.
The exact and shared target may use the same container, whose `.ic-testkit`
metadata remains preserved. Independent high-frequency callers can use
`maintain_shared_incremental_target_at_most_every`; matching recent passes
coordinate through a small cross-process marker and skip both Cargo input
resolution and whole-target traversal. Missing targets remain uncreated,
policy changes are immediately due, and a zero interval always runs. When
retention always accompanies a Wasm acquisition, configure
`with_shared_incremental_target_maintenance_at_most_every` instead. It creates
and coordinates the target even on exact hits, reuses the acquisition's input
resolution for due maintenance, and reports the result through the build
record and structured progress events. Integrated maintenance is strict by
default. Consumers that prefer a usable Wasm acquisition over successful
retention can supply an explicit best-effort configuration:

```rust,no_run
# let environment = std::collections::BTreeMap::<std::ffi::OsString, std::ffi::OsString>::new();
use ic_build::artifacts::{
    SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode,
    CacheRetentionLimits, WasmBuildSpec,
};
use std::time::Duration;

# let workspace = std::path::PathBuf::from(".");
# let target = workspace.join("target/pic-wasm");
let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
    CacheRetentionLimits::new().with_max_size_bytes(4 * 1024 * 1024 * 1024),
    Duration::from_secs(24 * 60 * 60),
)
.with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
let spec = WasmBuildSpec::new(
    &workspace,
    &target,
    &["counter_canister"], environment.clone())
.with_shared_incremental_target(target.join("shared"))
.with_shared_incremental_target_maintenance(maintenance);
# let _ = spec;
```

A best-effort failure is retained as
`SharedIncrementalTargetMaintenanceOutcome::Failed` and does not record a
successful schedule marker. Spec validation, Cargo input resolution, builds,
source-race detection, and artifact publication remain strict. Configuration
getters support policy assertions without performing work.

Long input resolution, lock waits, maintenance, Cargo builds, and artifact
publication can expose synchronous structured progress without changing the
existing silent API:

```rust,no_run
# let environment = std::collections::BTreeMap::<std::ffi::OsString, std::ffi::OsString>::new();
use ic_host_process::child::{CleanupPolicy, OwnedChild};
use ic_build::artifacts::{
    WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildSpec,
    build_wasm_canisters_cached,
};
use std::time::Duration;

# let workspace = std::path::PathBuf::from(".");
let spec = WasmBuildSpec::new(
    &workspace,
    &workspace.join("target/pic-wasm"),
    &["counter_canister"], environment.clone());
let outcome = ic_build::artifacts::build_wasm_canisters_cached_with_progress(
    &spec,
        WasmBuildProgressConfig::new()
            .with_heartbeat_interval(Duration::from_secs(15))
            .with_cargo_output(false),
        &mut |event| {
            if let WasmBuildProgressEvent::Heartbeat { phase, elapsed } = event {
                eprintln!("{phase} is still active after {elapsed:?}");
            }
        },
    |command| OwnedChild::spawn(command, CleanupPolicy::KillAndWait))?;
eprintln!("{outcome}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Observers run synchronously on the build thread and should return promptly.
Raw output events preserve non-UTF-8 bytes; output is still captured in
`WasmBuildError` when forwarding is disabled. An observer panic joins active
phase work, terminates and waits for a running Cargo build, and then performs
normal lock and incomplete-entry cleanup.

Independent batches and prepared snapshots each expose one batch
entrypoint. Pass batch configuration explicitly and `None` for silent execution,
or `Some((progress_config, &mut observer))` for progress reporting. The former
`_with_config`, `_with_progress` and combined variants have no aliases.

Suites that require standalone feature resolution for several variants can
batch independent specs without changing Cargo feature semantics:

```rust,no_run
# let environment = std::collections::BTreeMap::<std::ffi::OsString, std::ffi::OsString>::new();
use ic_host_process::child::{CleanupPolicy, OwnedChild};
use ic_build::artifacts::{
    LabeledWasmBuildSpec, WasmBuildBatchConfig, WasmBuildSpec,
    build_wasm_canisters_cached_batch,
};

# let workspace = std::path::PathBuf::from(".");
# let target = workspace.join("target/pic-wasm");
let specs = [
    LabeledWasmBuildSpec::new(
        "role-a",
        WasmBuildSpec::new(&workspace, &target, &["role_a"], environment.clone()),
    ),
    LabeledWasmBuildSpec::new(
        "role-b",
        WasmBuildSpec::new(&workspace, &target, &["role_b"], environment.clone()),
    ),
];
let batch = build_wasm_canisters_cached_batch(&specs, WasmBuildBatchConfig::new(), None, |command: &mut std::process::Command| OwnedChild::spawn(command, CleanupPolicy::KillAndWait))?;
eprintln!("{batch}");
for failure in batch.failures() {
    eprintln!(
        "build {} ({}) failed in {:?} after {:?}: {}",
        failure.label(),
        failure.index(),
        failure.failure_details().unwrap().phase(),
        failure.entry_elapsed(),
        failure.error().unwrap(),
    );
    eprintln!("partial timings: {:?}", failure.failure_details().unwrap().timings());
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Starting with 0.13, `outcomes()` and `failures()` borrow canonical report entries
instead of constructing separate success/failure view types. Successful entries
have `outcome().is_some()`; failed entries retain `error()` and `failure_details()`
(or `failure()` for generic artifact batches). Access labels, indices and wall
times on the entry. Shared-target maintenance yields `(entry, outcome)` pairs.
The former view types are removed without compatibility aliases.

The batch is sequential and collect-all: every independent failure is returned
in one report, but entries are not run simultaneously. Every
`LabeledWasmBuildSpec` label
must be nonempty and unique; invalid structure returns
`WasmBuildBatchContractError` before resolution, progress, or build work. Its
`WasmBuildBatchReport` retains canonical ordered entries and provides
structured labeled `outcomes` and `failures` iterators. Labels are reporting
identity only and never affect exact cache keys. Progress and integrated
maintenance entries carry the same label. Every entry uses the ordinary
single-spec implementation and its own exact identity, locks, policy, and
Cargo command; packages are never collapsed into one invocation that could
unify shared dependency features.
When several packages are intentionally built together with Cargo's normal
shared feature resolution, keep them in one multi-package `WasmBuildSpec`;
splitting that build into per-package batch entries would repeat Cargo work and
may change feature resolution.

Resolution-compatible entries share one workspace/toolchain snapshot: Cargo
and rustc identity, Cargo metadata, input discovery, and memoized per-root
content digests are reused while each spec retains its standalone fingerprint
and Cargo invocation. A different workspace, tool environment, or
metadata-affecting feature argument starts a separate snapshot so independent
feature semantics remain intact.

`WasmBuildBatchReport::metrics` aggregates built, exact-cache reused, and failed
counts; input-resolution runs and compatible-snapshot reuses; and summed
successful acquisition timings. The structured input-resolution total makes
distinct feature-resolution costs visible while reused snapshots remain
counted explicitly, including reuse from a prepared concurrent snapshot. `entry_elapsed` retains wall time for every ordered entry,
including failures; `failures` returns structured entries that bundle the
label, index, error, primary failed phase, partial phase timings, and elapsed
time. Successful entries continue to expose detailed phase timings through
their build records.

### Prepared source immutability

Mutable `WasmBuildSession` reuse is removed in 0.17. Prepare the complete recipe
set through `WasmBuildInputSnapshot` under a genuine source write-exclusion lease.
The lease covers tools, manifests, configuration, sources and selected environment;
the guard's provenance remains the caller's responsibility. Ordinary batches
resolve current inputs each time and retain within-batch sharing.

When independent callers need the same resolution concurrently, prepare their
complete exact specification set before starting readers:

```rust,no_run
use ic_host_process::child::{CleanupPolicy, OwnedChild};
use ic_build::artifacts::{
    LabeledWasmBuildSpec, WasmBuildBatchConfig, WasmBuildInputSnapshot,
    WasmBuildSpec,
};
use std::sync::Mutex;

# let local_test: Vec<LabeledWasmBuildSpec> = Vec::new();
# let production: Vec<LabeledWasmBuildSpec> = Vec::new();
let prepared_specs = local_test
    .iter()
    .chain(&production)
    .map(|entry| entry.spec().clone())
    .collect::<Vec<WasmBuildSpec>>();
// Every source/config/tool/environment writer must coordinate on this lock.
let source_write_exclusion = Mutex::new(());
let source_guard = source_write_exclusion.lock().expect("acquire source lease");
let snapshot = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
    &source_guard,
    &prepared_specs,
)?;
let (local_report, production_report) = std::thread::scope(|scope| {
    let local = scope.spawn(|| {
        snapshot.build_batch(&local_test, WasmBuildBatchConfig::new(), None, |command: &mut std::process::Command| OwnedChild::spawn(command, CleanupPolicy::KillAndWait))
    });
    let production = scope.spawn(|| {
        snapshot.build_batch(&production, WasmBuildBatchConfig::new(), None, |command: &mut std::process::Command| OwnedChild::spawn(command, CleanupPolicy::KillAndWait))
    });
    (
        local.join().expect("join LocalTest reader"),
        production.join().expect("join Production reader"),
    )
});
local_report?;
production_report?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Preparation is closed: a reader requesting an exact spec absent from
`prepared_specs` receives `SpecificationNotPrepared` before progress or build
work; it never extends the snapshot or hashes a new path. A detected post-build
source race invalidates the snapshot for every later reader. Publication takes
a shared read boundary against invalidation, so a publication already at that
boundary completes before invalidation while every later publication is
rejected. `WasmBuildInputSnapshot::metrics` exposes preparation work,
cumulative reader reuse, and invalidation; batch metrics count
`input_resolution_prepared_reuses` separately.

Each batch remains sequential. Separate snapshot readers may overlap, but
shared incremental Cargo targets still participate in their existing locks;
use isolated targets to retain useful Cargo parallelism. There is no built-in
parallel scheduler, ambient cache, or mutation-owning workspace abstraction.
Callers without a genuine source write-exclusion guard must keep ordinary
per-call resolution.

When independent specs share incremental targets, batch-owned maintenance
removes the caller convention of modifying the first spec:

```rust,no_run
use ic_host_process::child::{CleanupPolicy, OwnedChild};
use ic_build::artifacts::{
    LabeledWasmBuildSpec,
    SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode,
    CacheRetentionLimits, WasmBuildBatchConfig,
    build_wasm_canisters_cached_batch,
};
use std::time::Duration;

# let specs: Vec<LabeledWasmBuildSpec> = Vec::new();
let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
    CacheRetentionLimits::new().with_max_size_bytes(4 * 1024 * 1024 * 1024),
    Duration::from_secs(24 * 60 * 60),
)
.with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
let batch = build_wasm_canisters_cached_batch(
    &specs,
    WasmBuildBatchConfig::new().with_shared_incremental_target_maintenance(maintenance),
    None,
    |command| OwnedChild::spawn(command, CleanupPolicy::KillAndWait),
)?;
for (entry, outcome) in batch.shared_incremental_maintenance_outcomes() {
    eprintln!("build {} ({}): {}", entry.label(), entry.index(), outcome);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Maintenance is attached to the first specification for each distinct resolved
shared-target directory. Relative paths are workspace-relative, and filesystem
aliases of the same directory share one maintenance attempt. Isolated specs
are unaffected. A specification
that also configures per-spec integrated maintenance receives a labeled,
indexed error, and later entries still run.

`ResolvedCargoBuildInputs::is_current` resolves the semantic fingerprint again,
while `is_content_current` cheaply rehashes the conservative validation paths.
Wasm target/path admission is retained as one success or refusal. Generic
snapshots for non-Wasm packages still support input and transaction validation;
requesting `artifact_paths()` or a Wasm acquisition returns the deferred refusal.
Invalid selections do not retain a partial target/path inventory.
The generic builders preserve dynamic and non-UTF-8 argument and environment
bytes. Add direct dependencies on the appropriate Host crates for raw artifact,
filesystem, process and IC helpers. IC Build does not re-export them.
For Unix tool selection, use
`ic_host_process::tool::resolve_executable(requested, current_dir, search_directories)`
with an absolute working directory and explicit search order before declaring
the selected path through `ArtifactCacheSpec::with_tool`. Resolution reads no
ambient `PATH`; filesystem errors stop search. It selects a path without admitting
it for execution. `ic_host_process::tool::AdmittedTool` adds caller-supplied digest
and exact version admission, bounded output, an explicit environment and a deadline
for external transforms. Keep the executable and its parent directories protected
from concurrent writers during admission and execution.

Read verified compiled Wasm through a retained build record and the shared
filesystem owner:

```rust
use ic_host_process::child::OwnedChild;
use ic_build::artifacts::build_wasm_canisters_cached;
use ic_host_fs::read::read_file;

let build = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)?;
let path = build.record().artifacts()[0].as_path();
let bytes = read_file(path, max_bytes)?;
drop(build); // Retain the cache entry until consumption finishes.
```

The caller chooses the byte limit; failures are typed rather than panics.
The 0.4 API removed `wasm_path` and `wasm_artifacts_ready`. Consumers
obtain verified outputs from retained records. Path construction needed only
by fixtures stays in consumer test support.
Paths must be caller-controlled; symbolic links are allowed. Reading bytes does
not validate Wasm; use `ic_host_artifacts::wasm::inspect` for structural facts,
enabling Host Artifacts' `wasm` feature in the inspecting consumer. Build does not
enable that parser feature; producing and caching Wasm does not require it.
Stream identities, archives and Wasm inspection belong to `ic_host_artifacts`;
pathname reads and durable publication belong to `ic_host_fs`; executable
resolution, execution and Git provenance belong to `ic_host_process`; Candid
service interpretation and response decoding belong to `ic_host_tools`.
Bounded line normalization belongs to `ic_host_text`. Build 0.12 owns
native extraction through Unix-only `artifacts::candid::extract`: supply an
admitted tool, absolute source path, execution context, source-byte allowance
and output limits. It returns source/tool identities and complete raw successful
process evidence. Changed or unreadable sources after invocation reject while
retaining output. Extraction does not normalize, parse, cache or publish.
Callers exclude source/tool writers, choose optional deadlines and admit
text/grammar separately; before/after digests cannot rule out transient writes.
Host 0.24 has removed its extraction module/feature; adopt the new Build owner
when upgrading that graph ([#51](https://github.com/dragginzgame/ic-build/issues/51)).
Structural Wasm inspection does not replace PocketIC's runtime validation.
Current source requires compatible published IC Host 0.31 releases where the
public engine boundary exchanges Host values. Build 0.23 metadata/tree
rendering and metadata decoding use standard commands and bytes instead.
Host FS's `capability` and `temporary-file` backends remain disabled here;
direct consumers of those APIs enable the corresponding Host features explicitly.
Host Artifacts/FS supply stream visitation and fresh-directory allocation; FS also owns shared
lock guards, logical directory sizing and cache-directory tags. Cargo.lock records
the selected graph. Host Process supplies cancellable
process-group probe capture. Build owns record lifetime, pruning policy and tag comments,
while Host owns these filesystem primitives. Existing valid tags are preserved;
new tags identify IC Build without renaming retained cache paths or formats.
Retained staging still uses checked native permission conversion on macOS. Align direct Host
dependencies when passing `OutputLimits`, `CleanupPolicy` or matching typed Host
failures across the engine boundary. Different Host minor families have distinct Rust crate identities even
when their fields match. The earlier IC Build 0.2.0 boundary also means an
`ic-build = "0.1"` requirement does not select this engine. Process execution returns completed
results even for nonzero exits: call `require_success()` before admitting or
publishing producer output, and return `io::Result<()>` from output observers.
Durable writers retain typed publication phase and cleanup evidence. Gzip
encoding accepts numeric levels 0–9. PocketIC startup remains Testkit-owned.

The `transactional_artifact_cache` example uses shared digest/version admission
and bounded execution for its transformer. Supply a reviewed raw executable
SHA-256 and the exact trimmed stdout of its `--version` command:

```bash
cargo run -p ic-build --locked --offline --example transactional_artifact_cache -- \
  /path/to/transformer "$TOOL_SHA256" 'transformer 1.0.0' \
  input.wasm output.wasm target/transform-cache
```

It runs with an empty environment, a 60-second deadline and 64 KiB per output
stream, admitting executables up to 256 MiB. A failed admission or execution
does not commit output. The operator owns interpreter/library trust and any
effects the transform has beyond its declared output.

External transforms derived from a Cargo package closure can pass both the
build spec and its resolved snapshot to
`ArtifactCacheSpec::with_cargo_build_inputs`. The semantic Cargo fingerprint
becomes transactional identity, while the conservative Cargo-aware input paths
and generated-state exclusions are revalidated during preparation, hit
materialization, and commit. This avoids duplicating Cargo discovery as broad
workspace scans while still rejecting source, workspace, or toolchain races.

External deterministic tools use the transactional artifact-set cache. The
caller declares exact inputs, tool bytes, arguments, relevant environment, and
the complete output schema; IC Build owns cross-process locking, staging,
before/after input verification, validation, publication, materialization, and
retention:

```rust,no_run
use ic_build::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, CacheRetentionLimits,
    ArtifactCacheSpec, prepare_artifact_cache,
};
use std::{io, process::Command, time::Duration};

# let workspace = std::path::PathBuf::from(".");
let input = workspace.join("target/wasm/counter.wasm");
let optimizer = workspace.join("tools/wasm-optimizer");
let destination = workspace.join("target/optimized/counter.wasm");
let spec = ArtifactCacheSpec::new(
    &workspace.join("target/external-artifact-cache"),
    "counter-optimizer",
    "counter/optimizer/v1",
)
.with_input("counter.wasm", &input)
.with_tool("optimizer", &optimizer)
.with_arguments(["--optimize-for-size"])
.with_environment([("OPTIMIZER_MODE", "deterministic")])
.with_output("optimized.wasm", &destination)
.with_prune_policy_at_most_every(
    CacheRetentionLimits::new()
        .with_max_age(Duration::from_secs(14 * 24 * 60 * 60))
        .with_max_size_bytes(2 * 1024 * 1024 * 1024),
    Duration::from_secs(60 * 60),
);

let outcome = match prepare_artifact_cache(&spec)? {
    ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
    ArtifactCachePreparation::Build(transaction) => {
        let staged = transaction.output_path("optimized.wasm")?;
        let status = Command::new(&optimizer)
            .args(["--optimize-for-size", "-o"])
            .arg(staged)
            .arg(&input)
            .env("OPTIMIZER_MODE", "deterministic")
            .status()?;
        if !status.success() {
            return Err(io::Error::other(format!("optimizer failed with {status}")).into());
        }
        transaction.commit()?
    }
};
eprintln!("artifact key: {}", outcome.record().key());
// Retain the outcome until reading or staging the exact output is complete.
let optimized_wasm = std::fs::read(outcome.record().artifacts()[0].path())?;
# let _ = optimized_wasm;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Multiple independent external recipes can use
`build_artifact_caches_batch`. Every `ArtifactCacheSpec` is wrapped in a
`LabeledArtifactCacheSpec`; labels must be nonempty and unique and are retained
in callbacks and ordered report entries. Labels are composition metadata, not
cache identity. A structural label error rejects the batch before work starts.

The spec describes command identity; it does not execute a tool or apply its
declared arguments and environment to `Command`. The caller's build callback
must run the declared recipe, as in the example above.

The callback receives only cache misses and only one live transaction at a
time, avoiding self-deadlock when specs share a coordination scope. Callback
failures synchronously abort the current staging directory, retain their label,
and do not stop later independent entries. `ArtifactCacheBatchReport::metrics`
aggregates built, reused, and failed counts plus all successful acquisition
timings. Every entry retains wall time; failed entries additionally distinguish
preparation, callback, explicit abort cleanup, and commit time. Commit timing
includes commit-owned failure cleanup. The operation is deliberately not atomic
across specs: use one `ArtifactCacheSpec` with several outputs when all
artifacts must publish as one transaction.

Preparation returns `ArtifactCacheError::InputsChangedDuringPreparation` on the
first detected input change, including changes while waiting for locks or after
warm materialization. It preserves the original before/after digests and does not
silently retry under a new input identity. Consumers own retry and source-custody
policy; existing retained output records remain valid through the refusal.

A miss transaction exposes checked staging paths for redirectable tools and an
`import_output` helper for commands that write to fixed locations. `commit`
accepts only the complete declared output set and publishes its manifest last;
dropped, failed, or panicked transactions remove staging without creating a
cache hit. Multi-output recipes declare several logical outputs and commit them
as one unit. Recipe identity is caller-owned and must change when undeclared
pipeline semantics change. Namespace and recipe identifiers are hashed before
being used on disk, environment values are never rendered in `Debug` or cache
manifests, and public destination paths do not affect content identity.
Prefer declaring pipeline implementation files, configuration, and wrappers as
ordinary exact inputs. Reserve manual `recipe_id` bumps for semantic changes
that cannot be represented by stable input bytes.

Declared input and tool roots may contain the cache directory, which is
excluded while recursively hashing that broader tree, but they must not be
located inside the cache themselves. Public output destinations must also stay
outside the cache, resolve to distinct non-nested paths, and not overlap a
declared input
or tool. These checks prevent silently unkeyed inputs, self-invalidating
recipes, and two logical outputs overwriting the same file.

The default coordination scope is the namespace. Recipes that use the same
mutable external work tree can select a shared scope even when their content
keys differ. Independent recipes can use distinct scopes. Exact content-key
locking still ensures that overlapping callers build one result only.

Artifact contents are hashed, imported, and materialized with bounded-memory
streaming. Configured or explicit retention also removes staging abandoned by
terminated processes when the corresponding content-key lock proves that no
transaction is active. `ArtifactCachePruneReport` reports those uncommitted
directories separately from committed entry age/size eviction.

`ArtifactCacheSpec::with_prune_policy_at_most_every` applies the same explicit
limits but scans no more frequently than the selected interval. A skipped pass
has no maintenance outcome; its inexpensive marker-check duration remains in
the timing record.

No retention limit is automatic. As a starting point for developer caches,
the examples use a 14-day age bound with a size appropriate to the entry type
(larger for isolated Cargo targets, smaller for final artifact sets). CI should
choose an explicit limit that fits its cache quota and expected fingerprint
count rather than inheriting a machine-wide default.

Use `InputEvidence` to capture inputs and reject changes across a caller-owned
build. Use `ArtifactCacheSpec` for transactional publication, output content
validation and retained consumption. The former watched-input stamp API is
removed in 0.3.0; existing `.ic-testkit-input` sidecars are left untouched and
grant no admission through these APIs. ICP readiness policy belongs to consumers.

The cache intentionally contains no Binaryen, `icp build`, package-name, or
PocketIC policy. This guide and [IC Build's public cache documentation](../../crates/ic-build/src/artifacts/transaction.rs)
describe the supported transactional contract. Shared-target maintenance and
prepared reuse likewise remain IC Build contracts with caller-owned policy.

## Historical extraction references

The following Testkit records explain the pre-extraction design. They are bound
to extraction source `b906b1a44f8f5f0f2f23d2413a76103d99e4fbb3`; their version
numbers refer to Testkit, and their historical contracts do not supersede this
guide or IC Build's public API documentation.

- [0.5 transactional artifact-set design](https://github.com/dragginzgame/ic-testkit/blob/b906b1a44f8f5f0f2f23d2413a76103d99e4fbb3/docs/design/0.5-artifact-transactions/0.5-design.md).
- [0.6 shared-incremental Wasm design](https://github.com/dragginzgame/ic-testkit/blob/b906b1a44f8f5f0f2f23d2413a76103d99e4fbb3/docs/design/0.6-shared-incremental-wasm/0.6-design.md).
- [0.7 artifact orchestration design](https://github.com/dragginzgame/ic-testkit/blob/b906b1a44f8f5f0f2f23d2413a76103d99e4fbb3/docs/design/0.7-artifact-orchestration/0.7-design.md).

Current Host contracts belong to [IC Host Tooling](https://github.com/dragginzgame/ic-host-tooling);
PocketIC upgrade and startup guidance belongs to IC Testkit. The pinned designs
above preserve extraction history without maintaining another Host/PocketIC
migration guide here.
