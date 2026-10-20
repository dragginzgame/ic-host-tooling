# Changelog

The 0.1.0 section preserves the original bootstrap notes. The published 0.1.0
archive also includes the extracted artifact engine and Cargo grouping API.

## [0.24.0]

- **Breaking Rust dependency contract:** adopt published IC Host 0.32.0 for
  Artifacts, FS and Process. Consumers exchanging Host values with Build's
  execution/artifact APIs must align their direct Host requirements to 0.32.
  Command/byte observation interfaces, retained formats and feature defaults
  are unchanged; the engine still needs no Platform, Candid or CDK dependency.
- Adopt Shared Tooling 0.25.8, retaining Cargo metadata failure evidence and
  accepting numbered `Unreleased` changelog headings. Run the existing Clippy
  gate before compilation, release packaging and publication; lint failure
  stops later stages under parallel Make.
- Clarify the published command/byte observation boundary and direct Host
  requirements in consumer guidance. Distinguish producer qualification from
  consumer integration and installed-artifact acceptance.

## [0.23.0] - 2026-10-20

- **Breaking:** Cargo metadata and tree specifications now render commands for
  the consumer's own process runner. Replace the combined observation functions
  with command rendering, successful/complete output admission and canonical
  metadata decoding (or standard UTF-8 tree decoding). Observation callers can
  keep their selected Host generation, budgets and typed failures.
- **Breaking:** workspace discovery follows the same command/byte boundary;
  replace combined discovery with command rendering, caller-owned capture and
  workspace-response decoding. Feature-safe grouping takes a tree observer
  instead of Host limits and a command factory, keeping execution policy and
  typed runner failures with consumers while preserving feature equivalence.

## [0.22.1] - 2026-10-20

- Adopt Shared Tooling 0.25.4, including explicit Cargo features in selected-tool
  preparation and clarified failure-evidence collection. Rerun explicit tool setup
  for the updated build policy; existing installations remain retained.
- Use Host Process 0.30.1's writer-process helper for the remaining executable
  test fixtures, preventing inherited writable descriptors from causing
  intermittent `Text file busy` failures during parallel validation.

## [0.22.0] - 2026-10-19

- **Breaking:** align exposed filesystem, artifact and process types with
  published IC Host 0.30.0. Consumers exchanging Host values must adopt the
  same 0.30 family. Leave unused capability and temporary-file backends disabled,
  removing their dependency trees while preserving build and cache behavior.

## [0.21.0] - 2026-10-19

- **Breaking:** align exposed filesystem, artifact and process types with
  published IC Host 0.29.0. Consumers exchanging Host values must adopt the
  same 0.29 family. Keep optional Host CLI, Wasm, gzip and text backends disabled;
  Cargo recipes, captured evidence and retained cache formats are unchanged.

## [0.20.0] - 2026-10-19

- Adopt Shared Tooling 0.25.0 and retire the Perl editors while preserving
  metadata rollback and independent Cargo checks. Local release commands select
  the prepared published Build 0.19.0 editor and an 8 MiB document allowance
  without manual exports. Explicit setup, standard release preflight and CI
  prepare the pinned tool; standalone validation remains offline.
- Use Clap for release-editor argument handling, with help/version output and
  positive byte-limit admission. Rendering operations, read-only input custody
  and candidate refusal behavior remain unchanged. Formatting and release
  checks preserve dependency-role sections through grouped manifest ordering.

## [0.19.0] - 2026-10-19

- **Breaking:** replace public `cargo::execution::communicate_cargo` with
  `CargoInvocation` and `execute_cargo`. Build renders typed graph choices,
  observes the explicit compiler and Cargo, drains messages and checks tool
  identities after success. Custom recipes select their environment, profile
  overrides, compiler arguments, process custody and budgets; inspect execution,
  protocol and facts before admitting artifacts. No compatibility entry point
  remains. Cached builds reuse the renderer and observer without extra probes
  or changes to retained formats, cache identities or source-mutation guards.

## [0.18.0] - 2026-10-18

- **Breaking:** replace `ArtifactStagingDirectory` with Host fresh-directory
  allocation and `OwnedDirectory`. Use published Host 0.27.5 stream visitation,
  retaining framed digests, bounded scratch and immediate size-change refusal.
  Host retries interrupted reads; callers retain cleanup policy
  ([#45](https://github.com/dragginzgame/ic-build/issues/45)).

- **Breaking:** use Host Artifacts 0.27.4 canonical batch label admission and
  `ic_host_artifacts::batch::BatchLabelError`. Remove `ic_build::batch` without
  a compatibility re-export; exact labels, duplicate indices and refusal before
  command/progress/build effects remain unchanged
  ([PR #65](https://github.com/dragginzgame/ic-build/pull/65)).

- Own explicit native Wasm optimizer qualification through retained artifact
  transactions, repaired cache reuse and the transferred execution oracle.
  Shared Tooling retains tool installation/admission; ordinary tests do not
  install or choose tools ([#61](https://github.com/dragginzgame/ic-build/issues/61),
  [PR #62](https://github.com/dragginzgame/ic-build/pull/62)).

- Render read-only Cargo release candidates with TOML-owned literal spans,
  preserving unrelated bytes and source-qualified dependencies. Expose an
  explicit, bounded CLI with live or saved workspace identity admission.
  Consumers retain preparation, release policy, source custody and rollback.
- Add independent semantic lock admission for exact local dependency-reference
  changes; the existing version-only projection contract remains unchanged.

## [0.17.6] - 2026-10-18

- Select development Rust 1.99.1, which fixes Rust 1.99.0's direct-invocation
  macOS rust-lld library lookup regression. Keep the package MSRV at 1.91.0 and
  retain native qualification requirements ([#76](https://github.com/dragginzgame/ic-build/issues/76),
  [#77](https://github.com/dragginzgame/ic-build/issues/77)).
- Add nearest explicit Cargo workspace discovery without invoking Cargo,
  preserving nested declarations, file starts, symlinks and typed refusals.
  Canic retains ICP discovery and no-match fallback; publication and consumer
  adoption remain separate ([PR #63](https://github.com/dragginzgame/ic-build/pull/63)).

## [0.17.5] - 2026-10-18

- Stop enabling Host Artifacts' unused `wasm` feature, removing wasmparser and
  leb128 from this engine's dependency graph. Consumers performing structural
  inspection select Host's `wasm` feature explicitly; Cargo/Wasm builds are unchanged.
- Consolidate Unix user identity, descriptor flags and signal constants onto
  Rustix, removing IC Build's direct libc dependency and local unsafe calls.
  Effective-owner admission and exact process-signal checks remain unchanged;
  libc remains in the upstream dependency graph.
- Delegate cancellable Cargo input probes to Host Process 0.27.3, retiring local
  spawn/deadline/capture orchestration. Preserve caller-selected cleanup, absolute
  probe deadlines, bounded evidence and Build's phase/error mapping; ordinary
  builds still have no default elapsed-time deadline.

## [0.17.4] - 2026-10-18

- Delegate retained descriptor locks, logical cache sizing and cache-directory
  tags to published Host FS 0.27.2 APIs. Preserve record lifetime, pruning policy,
  retained formats and typed failures; newly written tag comments identify Build.
- Replace stale pre-0.17 adoption guidance with the current API and dependency
  boundary. Keep historical extraction evidence separate from supported usage.

## [0.17.3] - 2026-10-18

- Bring retained Wasm evidence/export and compile-diagnostic helpers onto the
  current engine from PRs #59/#67. Evidence uses explicit child custody and binds
  both the exact build fingerprint and conservative raw input snapshot.
- Preserve caller-owned recipes, limits, source custody and report publication;
  old branch-only consumer APIs need an explicit environment and acquisition spawner.

## [0.17.2] - 2026-10-18

- Retain fingerprinted tool/recipe facts on successful and reused Wasm records.
  Provenance callers borrow the observations without extra probes; retained cache
  formats, keys, validation and Debug redaction remain unchanged.

## [0.17.1] - 2026-10-18

- Adopt the shared Candid/CDK dependency-boundary check in CI/release validation.
  Review published IC Platform 0.1.3; Build's opaque extractor orchestration needs
  no platform library dependency. Preserve extraction limits, errors and evidence.

- Adopt Shared Tooling 0.24.0 and annotate production versus test/tooling roles.
  Keep global manifest sorting and enforce it in current and committed release
  checks. Common setup prepares the two formatters; extraction tools stay explicit.
  Dependency selections and the published library contract are unchanged.

## [0.17.0] - 2026-10-18

- Keep the synthetic compiler-invalidation fixture independent of the validation
  adapter's explicit `RUSTC`, preserving compiler and optimizer cache assertions.

- Retain fingerprinted tool identities, Cargo selection and the complete selected
  environment on resolved inputs for provenance consumers, without new probes.
  Debug omits environment values and probe bytes; consumers own execution binding
  and publication redaction. Existing fingerprint bytes and admission are unchanged
  ([#71 — Retain observed build facts](https://github.com/dragginzgame/ic-build/issues/71)).

- Replace drifting README snippets with compiled Cargo examples and correct
  current ownership/upgrade guidance. Staging close delegates one-attempt cleanup
  to Host; retained artifacts, recovery policy and public contracts are unchanged.
- Exclude package target directories from publication and future Git additions,
  preserving the existing cache tag and lock files on disk.

- **Breaking:** `cargo::compatible_package_groups` requires the recipe's
  `CargoSelection` after `target`. Isolated and combined probes use its features,
  locked and offline choices; profiles remain build-only.
- Carry cache paths, public paths and verified digests together during Wasm
  publication. Preserve copy-before-stamp ordering and selective warm repairs;
  refuse mismatched Cargo output counts before copying.
- Derive configuration and lock-projection error contracts through `thiserror`,
  preserving variants, messages, redaction and original sources
  ([#74 — Verified Wasm outputs and recipe selections](https://github.com/dragginzgame/ic-build/issues/74)).

- **Breaking:** Wasm specifications require the complete environment at construction.
  Probes and builds clear inheritance; replace inherited-name/override builders
  with a caller-selected map. Environment replacement is complete, and explicit
  Cargo/Rustc selection remains available. New recipe keys preserve old retained
  entries and stamp formats without reuse under changed execution semantics.
- Prepared batches bind requested snapshots once before reuse accounting or
  progress, preserving rejection order and source-lease invalidation
  ([#72 — Make Wasm environments explicit and bind prepared inputs once](https://github.com/dragginzgame/ic-build/issues/72)).

- Retain the released silent/progress entrypoints and five-part Wasm batch
  consumption, while sharing private dispatch and failure storage.
- **Breaking:** remove mutable `WasmBuildSession` reuse and its metrics. Prepare
  the complete specification set with `WasmBuildInputSnapshot` under the existing
  source lease. Independent batches retain within-batch input-resolution sharing.
- Share Cargo child communication and canonical message observation through
  `cargo::execution`, including exact manifest-bound cdylib selection. Build uses
  this path; Canic's isolated adapter is qualified separately
  ([#75 — Shared execution and one prepared reuse lifecycle](https://github.com/dragginzgame/ic-build/issues/75)).
- Delegate Cargo newline framing to Host Process while preserving JSON/text
  interpretation, per-record limits, EOF-tail acceptance and original sink errors
  ([#47 — Consolidate boundary observations, lock decoding and native stream framing](https://github.com/dragginzgame/ic-build/issues/47)).

## [0.16.0] - 2026-10-17

- **Breaking:** align exposed Host types with the registry Host 0.27 family.
  Align consumers' direct Host dependencies when passing Host values through Build.
- Adopt Shared Tooling 0.23.0. Common setup/checks cover host and Cargo tools;
  IC executables remain an explicit product selection. Release-adapter fixtures
  no longer require an unused IC installation; retained installations stay intact.

- **Breaking:** artifact preparation refuses the first detected input mutation
  instead of retrying internally. Callers own recovery; retained outputs and
  the existing before/after error evidence remain available.
- **Breaking:** zero-interval retention always runs without reading due markers.
  Both always-run builders now report `WasmBuildSpec::prune_interval()` as `None`;
  use `prune_policy()` to distinguish disabled maintenance. Marker formats stay intact
  ([#69 — Preparation refusal, output admission and schedules](https://github.com/dragginzgame/ic-build/issues/69)).

- **Breaking:** use `CacheRetentionLimits` for exact-entry and shared-target
  maintenance. Replace both former policy types directly; thresholds and schedule
  markers stay unchanged, as do LRU pruning and whole-target clearing semantics.
- **Breaking:** match artifact batch failures as `Preparation`, `Callback` or
  `Commit`. Remove phase-enum/accessor usage; primary errors, cleanup evidence
  and partial timings remain available on the canonical failure.
- **Breaking:** decode Cargo metadata once through its typed schema and use a v2
  Wasm fingerprint domain. Existing recipes rebuild under new keys; retained
  artifacts, stamp formats and namespaces stay intact, with no migration or cleanup
  ([#68 — Retention limits, tagged failures and typed fingerprints](https://github.com/dragginzgame/ic-build/issues/68)).

- **Breaking:** select exact Cargo depfiles with `read_cargo_depfile_inputs`;
  directory-wide discovery is removed. Consumers select graph records and decide
  whether `NotFound` permits a cold inventory. Strict parsing and input custody remain.
- **Breaking:** configuration observations expose Host's `Sha256Digest` directly.
  Use `as_bytes()` for raw bytes or `Display` for hex; hash bytes stay unchanged.

- **Breaking:** Wasm builds, batches, sessions and prepared readers require an
  explicit Host child spawner. Select direct-child or group cleanup deliberately;
  progress no longer changes custody or stdin. No build deadline is introduced.
- **Breaking:** Rust-flag composition accepts explicit encoded and ordinary inherited
  values before the additions slice. Resolve the command's selected environment
  at the caller; precedence and argument boundaries remain unchanged
  ([#64 — Explicit execution and flag inputs](https://github.com/dragginzgame/ic-build/issues/64)).

- **Breaking:** native executable specifications select `NativeTargetKind::Binary`
  or `NativeTargetKind::Example` instead of string target kinds. Update callers
  directly; no compatibility selector is retained.
- Rebuild missing or invalid exact Wasm cache entries through Cargo. Public
  outputs no longer reconstruct entries; retained entries still block replacement
  and valid exact hits still repair public outputs. Retained formats stay unchanged.
- Serialize archive verification workspaces through TOML, retaining exact generated
  manifest checks and the independently admitted package graph and lock selections.

## [0.15.0] - 2026-10-17

- **Breaking:** align exposed Host types with the incoming registry Host 0.26
  family. Align direct Host dependencies before passing Host values
  ([PR #57 — Align exposed build types with Host 0.26](https://github.com/dragginzgame/ic-build/pull/57)).
- **Breaking:** standalone input file facts expose Host's `Sha256Digest` directly.
  Replace file-hash `to_hex()` calls with `to_string()`; keep `InputDigest` for
  derived inventory and cache identities. Existing digest and cache bytes remain
  unchanged; no alias or retained-data migration is added.
- Keep archive source inventories typed until evidence serialization, preserving
  their JSON fields and bytes while removing dynamic JSON from comparisons.
- Render feature, lock and network choices through one Cargo graph encoder.
  Observations use the same flag ordering as recipes; recipe fingerprint bytes
  and the public observation specifications remain unchanged
  ([#60 — Canonical hashes, typed inventories and Cargo arguments](https://github.com/dragginzgame/ic-build/issues/60)).

## [0.14.0] - 2026-10-17

- **Breaking:** move `CargoFeatureSelection` to `cargo::selection` and keep
  `all_features` inside it. Observations and builds share one feature encoder;
  typed recipe argument fingerprint bytes remain unchanged. No old-path alias.
- **Breaking:** artifact batches return `batch::BatchLabelError` directly; Wasm
  contract errors wrap it in `Labels`. Preserve early rejection and duplicate
  indices; Cargo grouping now retains the same typed label error.
- **Breaking:** progress, command errors and failure details use one
  `WasmBuildPhase`. Remove the progress/failure phase enums and their translation;
  distinguish Cargo/rustc identity failures while retaining aggregate timings.

- **Breaking:** replace raw Cargo recipe arguments with typed `CargoSelection`,
  shared by metadata and native/Wasm build commands. Use `with_cargo_selection`
  or native `selection`; package, target and output selection remain structural.
  Deterministic argument rendering can change keys for differently spelled old
  selections; retained formats, artifacts and default recipe keys remain intact.
- **Breaking:** Cargo output progress events use Host `OutputStream` directly;
  remove the duplicate `WasmBuildOutputStream` type without an alias.
- Share per-pass canonical input paths and directory facts between transactional
  and shared-target boundary validation, preserving their separate policies,
  observation order and pre-effect failures.

- **Breaking:** validate standalone input evidence from a post-work specification
  and use the returned validated inventory. Original and newly selected roots
  share one observation; dropped inputs still receive checks and digest encoding
  remains unchanged. Do not construct a separate post-work capture first
  ([Canic #530 — Converge build-input reuse](https://github.com/dragginzgame/canic/issues/530)).
- **Breaking:** use one Wasm batch entrypoint per execution mode, passing explicit
  batch configuration and optional progress reporting. Remove the configuration
  and progress wrapper variants without aliases; ordering, maintenance, source
  leases and retained successes remain unchanged
  ([#54 — Land batch report cuts](https://github.com/dragginzgame/ic-build/issues/54)).

## [0.13.0] - 2026-10-17

- **Breaking:** align exposed Host types with the selected Host 0.25 family.
  Align direct Host dependencies when passing Host values. Tool admission callers
  pass the expected version separately from the version invocation specification.
- **Breaking:** native executable builds require explicit child spawning and
  cancellation callbacks. Pass `OwnedChild::spawn_direct` and `|| false` to retain
  direct-child behavior; callers may select owned-group cleanup and in-flight
  cancellation. Host execution/cleanup errors remain intact
  ([PR #55](https://github.com/dragginzgame/ic-build/pull/55)).
- Share exclusive producer/log admission and capture-before-cleanup custody for
  caller-executed artifact transforms. Preserve partial outputs and diagnostics;
  tools, arguments, output budgets and process policy remain caller-owned
  ([PR #53](https://github.com/dragginzgame/ic-build/pull/53)).
- **Breaking:** batch success/failure iterators return canonical entry references;
  remove the separate success, failure and maintenance view types without aliases.
  Read optional results and failure details from entries; shared-target maintenance
  returns `(entry, outcome)` pairs. Labels, ordering, timing and retention remain intact
  ([#54](https://github.com/dragginzgame/ic-build/issues/54)).
- **Breaking:** make archive qualification Unix-only and remove its separate
  non-Unix source walker. The pure external-lock subset verifier remains portable.

## [0.12.1] - 2026-10-17

- Restore Shared Tooling snapshot verification by keeping the external-lock API
  guidance in the local artifact guide and restoring the shared publication guide.
- Expose bounded, typed external-lock subset admission using the archive
  qualifier's shared reader/comparison. Allow extra original packages and empty
  subsets; retain archive qualification's separate nonempty-graph policy.
  Reject malformed records, duplicate identities and changed optional checksums
  ([#25](https://github.com/dragginzgame/ic-build/issues/25)).
- Update consumer setup and ownership guidance for published Build 0.12 / Host
  0.24; record release-bound producer acceptance separately from application
  adoption and reconcile the superseded Candid extraction PR.

## [0.12.0] - 2026-10-16

- **Breaking:** align exposed Host types with registry Host 0.24.0. Consumers
  passing Host values must select the same Host minor family.
- Move native Candid extraction from Host Tools into `artifacts::candid`,
  preserving raw bounded evidence, caller-admitted tools and source-change
  refusal. Tools, normalization, grammar, custody and publication remain local
  ([#51](https://github.com/dragginzgame/ic-build/issues/51)).
- Use Host's owned-directory cleanup and fixed-buffer digest encoding directly;
  remove the local directory guard/remover and per-file evidence hex allocation
  without changing cleanup policy or retained fingerprint bytes
  ([#49](https://github.com/dragginzgame/ic-build/issues/49),
  [#50](https://github.com/dragginzgame/ic-build/issues/50)).

## [0.11.1] - 2026-10-16

- Compare retained manifests through Host's streaming matcher and one shared
  encoder, centralize batch source-lease invalidation, and format Cargo arguments
  without temporary join vectors. Preserve retained bytes, failure timing and
  empty-argument/environment precedence.
- Collect batch discovery results directly, retain binary archive digests until
  evidence formatting, and index lockfile projection selections once. Preserve
  indexed failures, evidence bytes and refusal ordering.
- Correct consumer setup guidance to use published Build 0.11 and the aligned
  Host 0.23 family, distinguishing producer CI from consumer qualification
  ([#9 — Qualify consumers against the current exposed Host boundary](https://github.com/dragginzgame/ic-build/issues/9)).
- Use canonical label ordering for transaction duplicate checks and output lookup,
  and collect effective environment values without an intermediate name set.
  Preserve validation order, diagnostics and build fingerprints.
- Share immutable artifact inventories across acquisition record clones while
  preserving slice accessors, Debug output and last-owner retention.

## [0.11.0] - 2026-10-16

- **Breaking:** align exposed Host types with the incoming Host 0.23.1 selection.
  Callers passing Host values must align their direct Host dependencies.
- Preserve the former unbounded missing-target symlink allowance, nonblocking
  lock admission and child cleanup policies through Host's current APIs.
- Move owned Cargo artifact paths into results, stream error lists directly to
  their formatter and avoid retaining generated-root lists for boundary decisions.
  Preserve ambiguity checks, exact diagnostics and complete boundary observation.
- Share fresh/batched snapshot assembly and reuse package-field JSON scratch
  space, preserving exact fingerprints, fresh verification and timing attribution.
- Format redacted cache Debug lists from borrowed iterators while preserving
  ordinary and pretty output.

- Move isolated dependency trees and borrow candidate package names during
  feature-safe grouping, preserving greedy order and exact tree comparisons
  ([#50](https://github.com/dragginzgame/ic-build/issues/50)).
- Simplify atomic snapshot reader counting while preserving matching-only
  increments, concurrency, saturation and relaxed ordering.
- Index external lock identities once per semantic projection and remove
  temporary path allocations from boundary-cache hits and relative evidence
  encoding. Preserve first-match lock selection, lazy validation and digest bytes.
- Index batch resolution contexts while preserving first-seen groups, distinct
  feature selections and independent failure results.
- Collect depfile inputs directly into the discovery set and compute Wasm stamp
  read bounds without rendering a dummy stamp. Preserve refusals and retained bytes.
- Index workspace package selection per metadata context and move native
  identities/paths out of consumed metadata. Check archive-package uniqueness
  without collecting matches, preserving selection errors and fingerprints.
- Borrow configuration include paths and tree-root names, and select each Wasm
  artifact directory once per output set. Preserve include-validation order,
  normalized dependency trees and exact artifact paths.

## [0.10.0] - 2026-10-16

- **Breaking:** align exposed Host types with the incoming registry Host 0.22.1
  selection; callers passing Host values must align their direct dependencies
  ([#48](https://github.com/dragginzgame/ic-build/issues/48)). Preserve byte
  publication's former Replace/umask-filtered 0666 policy explicitly.
- **Breaking:** reserve Cargo's JSON message format for artifact selection;
  profile arguments may no longer override `--message-format`. Cargo wrappers
  must emit exact package/target artifact records for successful builds.
- Capture Wasm from Cargo-reported artifact paths and admit custom cdylib names.
  Preserve existing recipes' fingerprints, cache paths and stamp bytes. Share
  message decoding/matching with native executable selection. A caller-adjustable
  16 MiB per-message allowance refuses publication after completion on overflow,
  without terminating Cargo or adding a build deadline
  ([#49](https://github.com/dragginzgame/ic-build/issues/49)).
- Consolidate fresh/batched resolution completion while retaining fresh
  post-build verification and progress/failure phases.

- Avoid retaining undeclared-path lists for cache-validity checks and allocating
  a placeholder manifest for its read bound. Keep complete staged diagnostics,
  directory-read errors, exact manifest bounds and existing cache bytes.

- Reuse successful Cargo input-boundary observations within each transaction
  validation call, preserving lazy exclusion checks and fresh-call admission
  ([#47 — Consolidate boundary observations, lock decoding and native framing](https://github.com/dragginzgame/ic-build/issues/47)).
- Share external Cargo.lock field decoding between fingerprinting and archive
  qualification, preserving raw fingerprint bytes and each caller's refusal policy
  ([#47 — Consolidate boundary observations, lock decoding and native framing](https://github.com/dragginzgame/ic-build/issues/47)).

- Build selected native binaries/examples from Cargo's exact package and
  executable messages, including custom profiles and target directories.
  Diagnostics stream without a default elapsed-time deadline; callers retain
  tool, source and artifact custody
  ([#30 — Select native Cargo executables](https://github.com/dragginzgame/ic-build/issues/30)).

## [0.9.0] - 2026-10-16

- **Breaking:** remove the fourth, output-directory argument from
  `WasmBuildSpec::new`. Cargo profile arguments now select both the build profile
  and its artifact directory; replacing those arguments updates the directory.
  Callers must remove the fourth argument and keep profile selection in
  `with_cargo_profile_args`.
- Preserve existing fingerprint encoding and cache namespaces for matching
  recipes. Malformed profiles and unsafe derived directories still refuse
  before tool execution or acquisition.
- Consolidate artifact identifier/label length checks and use Host's existing
  raw-byte SHA-256 API for configuration observations, preserving limits,
  diagnostics and raw digest bytes.
- **Testing:** restore batch specification-failure coverage after the constructor
  migration and isolate the refusal fixture from source-tree artifacts.

## [0.8.0] - 2026-10-16

- **Breaking:** align exposed Host types with the selected IC Host 0.21 graph.
  Consumers passing Host limits or typed errors across this boundary must align
  their direct Host dependencies and rebuild.

- Decode standalone and archive-qualification Cargo metadata directly into the
  canonical schema without an intermediate JSON tree; retain original JSON in
  cache fingerprinting.
- Adopt Shared Tooling 0.15.2 and delegate locked release-cache preparation and
  pin checks to its shared preflight helper, preserving source/recovery admission
  ([#39 — Shared release preflight](https://github.com/dragginzgame/ic-build/issues/39)).
- Remove the unused Cargo release-editor installation, pin and admission flow;
  the shared workspace-version renderer prepares versions directly.
- Use Host's existing SHA-256 formatter for input digests, preserving lowercase
  hexadecimal output, leading zeroes and cache identities.
- Observe each Cargo configuration lookup path and include tree once per call,
  retaining distinct aliases, include ordering and fresh-call re-observation.
- Simplify depfile parsing by collecting dependencies directly and retaining
  only target presence, preserving strict syntax refusal and phony rules.
- Reuse Host's SHA-256 parser for package checksums while retaining lowercase
  syntax, qualification diagnostics and exact external-selection checks.
- Run shared-target coordination and scheduled-maintenance process qualification
  against independent Build-owned Cargo/Wasm fixtures, removing skips for a
  missing Testkit package.
- Share fixture-directory setup across unit and integration tests and use Host
  marker waits with explicit timeouts and early worker-exit diagnostics.

## [0.7.0] - 2026-10-15

- **Breaking:** align exposed Host types with IC Host 0.20. Consumers passing
  Host limits or typed errors across this boundary must align their
  direct Host dependencies and rebuild. Keep trusted digest admission in the
  example and integration tests through Host's new `VersionSpec` API
  ([#9 — Exposed Host boundary](https://github.com/dragginzgame/ic-build/issues/9)).

- Verify exact workspace-version projections in Cargo lockfiles without allowing
  dependency, source, checksum or unrelated metadata changes. Version selection
  and release receipts remain caller-owned
  ([#28 — Strict workspace lock projections](https://github.com/dragginzgame/ic-build/issues/28)).
- Hash captured input evidence relative to an explicit root for relocatable
  source fingerprints, preserving absolute-path digests and caller-owned
  inventories and provenance policy
  ([#40 — Relative source evidence](https://github.com/dragginzgame/ic-build/issues/40)).
- Compose inherited Cargo Rust flags with caller-selected arguments, preserving
  encoded-variable precedence, explicitly empty values and argument boundaries
  ([#36 — Cargo Rust flags](https://github.com/dragginzgame/ic-build/issues/36)).

## [0.6.1] - 2026-10-15

- Consolidate Cargo argument interpretation, exact-cache preparation and private
  Cargo fixture scaffolding while preserving public builders, fingerprint bytes
  and retained cache formats
  ([#46 — Internal cleanup](https://github.com/dragginzgame/ic-build/issues/46)).
- Stream captured artifacts through Host's bounded copy and durable writer,
  avoiding a complete in-memory copy while retaining source admission, typed
  failures and staging lifetime
  ([#45 — Capture and Host mechanics](https://github.com/dragginzgame/ic-build/issues/45)).
- Share read-only Cargo configuration and include observations through
  `cargo::environment`; reuse that owner in the Wasm engine while preserving
  cache identities, lookup aliases and caller-owned admission
  ([#35 — Cargo configuration observations](https://github.com/dragginzgame/ic-build/issues/35)).
- Correct registry and Host dependency guidance for published 0.6.0 and record
  Canic's delivered workspace-discovery and Host-boundary adoption.

## [0.6.0] - 2026-10-15

- **Breaking:** move `artifacts::workspace_root_for` to
  `cargo::observation::workspace_root_for` and require the caller's Cargo command
  and capture limits. Offline membership discovery still avoids dependency
  resolution and toolchain installation; there is no old-path reexport.
- Let owned-directory fields perform Drop cleanup directly, preserving staging
  cleanup before transaction locks are released and explicit recovery behavior
  ([#43 — Discovery and cleanup boundaries](https://github.com/dragginzgame/ic-build/issues/43)).

## [0.5.0] - 2026-10-15

- Adopt Shared Tooling 0.15.0's committed snapshot and strict release-input
  admission; qualify registry IC Host 0.18.6 with unchanged API minimums and
  executable pins.
- **Breaking:** merge `WasmBuildSpec::with_extra_env` overrides across calls,
  with later values winning for duplicate names. Empty calls preserve prior
  values; construct a new specification when a fresh environment map is needed.
- **Breaking:** rename `WasmBuildSpec::with_rustc_program` to
  `with_rustc_identity_program`. This selects only the compiler identity probe;
  explicit `RUSTC` still selects Cargo's compiler and takes probe precedence.
- **Breaking:** remove `artifacts::test_target_dir`. Consumers select their own
  test output paths, for example `workspace.join("target").join(name)`.
- Simplify collect-all artifact batching while preserving phase timings,
  synchronous abort and retained primary/cleanup failures
  ([#42 — Batch and boundary cleanup](https://github.com/dragginzgame/ic-build/issues/42)).

- Correct the registry dependency and Shared Tooling guidance for published 0.4.0,
  and reconcile delivered MSRV and retained-capture acceptance.

## [0.4.0] - 2026-10-15

- **Breaking:** require Rust 1.91.0 or newer and select `cargo-platform` 0.3.3.
  Consumers adopting this release must prepare that compiler or newer; published
  0.3.3's Rust 1.88 declaration is unchanged
  ([#34 — Dependency graph MSRV](https://github.com/dragginzgame/ic-build/issues/34)).
- Use equivalent Rust 1.91 duration constructors in tests and the cache example
  to satisfy strict Clippy after the MSRV increase.
- **Breaking:** remove the public `wasm_path` and `wasm_artifacts_ready` helpers.
  Obtain verified output paths from retained artifact records; fixture-only path
  construction belongs in consumer tests.
- Decode Cargo metadata through the canonical schema while preserving Wasm
  cache fingerprint projections, probe cancellation and retained diagnostics.
- Delegate package-source traversal on Unix to Host's descriptor-based walker,
  preserving source budgets and exact byte/mode evidence.

- Adopt Shared Tooling 0.14.0's committed governance snapshot and concise
  change-reporting guidance, preserving executable pins and release behavior.

## [0.3.3] - 2026-10-14

- Observe Cargo catalogs, resolved graphs and selected dependency trees through
  caller-bounded execution and Cargo's canonical Rust schema; share tree capture
  with feature-safe grouping
  ([#33 — Cargo observations](https://github.com/dragginzgame/ic-build/issues/33)).
- Finalize captured artifacts at caller-selected destinations with explicit
  retained-file and staging custody on rename or cleanup failure
  ([#26 — Capture finalization](https://github.com/dragginzgame/ic-build/issues/26)).
- Reconcile the release and PR handoff with published 0.3.2, preserving separate
  native, consumer-adoption and Canic peer-reproduction acceptance
  ([#1 — Extraction acceptance](https://github.com/dragginzgame/ic-build/issues/1)).

## [0.3.2] - 2026-10-14

- Adopt published Host fixture allocation in both test helpers, preserving
  caller-owned cleanup and retained evidence
  ([#27 — Adopt Host fixture allocation](https://github.com/dragginzgame/ic-build/issues/27)).
- Consolidate owned-directory cleanup across Wasm builds, artifact transactions
  and captures, preserving failure retries, explicit recovery and lock lifetime
  ([#32 — Consolidate cleanup guards](https://github.com/dragginzgame/ic-build/issues/32)).
- Correct the Host dependency guidance for published 0.3.1. Callers exchanging
  Host types with IC Build must align direct dependencies to Host 0.18
  ([#9 — Align the exposed Host boundary](https://github.com/dragginzgame/ic-build/issues/9)).

## [0.3.1] - 2026-10-14

- Adopt Shared Tooling 0.12.7 and delegate release metadata preparation and
  exact-commit validation to its canonical transaction helper, preserving local
  source admission, selected-editor custody and interrupted-release recovery.
- Stream Cargo archive qualification hashes through Host's bounded no-follow
  reader instead of retaining whole files solely for digesting.

- Qualify coordinated Cargo archives in a fresh retained workspace; admit exact
  extracted manifests and frozen external selections, then check source/archive
  integrity after tests. Consumers retain tools, package/version and release policy.
  ([#25](https://github.com/dragginzgame/ic-build/issues/25)).

- Add opt-in caller deadlines, cancellation and cleanup for Cargo input probes
  and revalidation. Preserve default execution, complete-output bounds and
  input identity ([#22 — Caller-bounded Cargo input resolution](https://github.com/dragginzgame/ic-build/issues/22)).

## [0.3.0] - 2026-10-14

- Adopt Shared Tooling 0.12.4 and published IC Host 0.17.11, retaining the
  reviewed tool pins and explicit dependency features.
- Delegate remaining descriptor locks and final-owner unlock to Host and remove
  the direct `fs2` dependency. Preserve cloned-record retention, inherited
  descriptor release and lock timing
  ([#21 — Adopt Host descriptor locks](https://github.com/dragginzgame/ic-build/issues/21)).

- **Breaking:** remove the watched-input stamp API (`WatchedInputSnapshot` and
  `icp_artifact_ready_*`). Use `InputEvidence` for input guards and
  `ArtifactCacheSpec` for verified retained outputs; existing sidecars are
  preserved ([#19 — Retire the watched-input stamp API](https://github.com/dragginzgame/ic-build/issues/19)).
- Share the verified destination-copy decision between Wasm and generic artifact
  caches, preserving digest domains, cold publication and stamp ordering
  ([#20 — Unify verified destination materialization](https://github.com/dragginzgame/ic-build/issues/20)).

- Use IC Host 0.17.9's opt-in Unix test fixture writer and remove both local
  executable-script implementations. Keep fixture content and assertions with
  IC Build ([#18 — Adopt Host executable fixtures](https://github.com/dragginzgame/ic-build/issues/18)).

## [0.2.7] - 2026-10-14

- Require IC Host 0.17.8's checked native permission conversion, preventing
  selection of the earlier retained-staging macOS compilation defect
  ([Host #84](https://github.com/dragginzgame/ic-host-tooling/issues/84)).

## [0.2.6] - 2026-10-14

- Add exact Cargo depfile reads through the existing discovery owner, allowing
  consumers to select their build graph without inspecting unrelated records
  in shared targets. Preserve bounded reads, exclusions and late-input rejection
  ([#17](https://github.com/dragginzgame/ic-build/issues/17)).

## [0.2.5] - 2026-10-13

- Replace the duplicated optional-duration helper with standard-library reduction
  in Wasm and artifact timing aggregation. Preserve absent measurements, measured
  zero and saturation; keep regression coverage at the aggregate owners
  ([#7](https://github.com/dragginzgame/ic-build/issues/7)).

## [0.2.4] - 2026-10-13

- Adopt Shared Tooling 0.11.2 and its unified CI binary installer, retaining
  existing tool pins and incorporating release/evidence fixes.
- Require compatible IC Host 0.17.4 releases for artifact, filesystem and process
  mechanics; retain explicit features and qualify the locked graph on Rust 1.88.

- Admit Cargo's default underscored cdylib filenames for hyphenated packages in
  every cache mode; refuse colliding outputs and genuinely renamed libraries
  ([#16](https://github.com/dragginzgame/ic-build/issues/16)).
- Add explicit captured-artifact retention for caller-owned recovery evidence
  ([#14](https://github.com/dragginzgame/ic-build/issues/14)).
- Bind named cdylib examples to shared Cargo input identity and output admission.
  Expose the existing command recipe for caller-owned execution without requiring
  a cache or changing deadlines, cleanup or consumer receipt encoding
  ([#13](https://github.com/dragginzgame/ic-build/issues/13)).

## [0.2.3] - 2026-10-13

- Make the native-path evidence fixture use filesystem-admissible filenames on
  macOS, retaining native capture, digest and descendant-error coverage. Linux
  still exercises invalid-byte filenames
  ([#15](https://github.com/dragginzgame/ic-build/issues/15)).
- Keep the remaining generic source-inventory and generated-root discovery
  regressions at IC Build as Canic adopts standalone evidence and removes its
  duplicate input engine ([#12](https://github.com/dragginzgame/ic-build/issues/12)).

## [0.2.2] - 2026-10-13

- Add standalone input evidence and strict Cargo depfile discovery for caller-owned
  build inventories. Recheck original inputs when inventories shrink or change,
  refuse late input authority, and preserve consumer evidence and existing cache
  encodings through the shared traversal. Recipes and release admission remain
  with consumers ([#12](https://github.com/dragginzgame/ic-build/issues/12)).
- Correct the artifact guide's IC Build ownership and Host 0.17 requirements.
  Keep source-lease and retained-cache contracts here, and bind historical
  Testkit references to their extraction revision
  ([#2](https://github.com/dragginzgame/ic-build/issues/2)).

## [0.2.1] - 2026-10-13

- Adopt Shared Tooling 0.10.0's snapshot-owned common catalogs. Keep existing
  executable versions and digests; select host, Rust and IC exceptions
  independently. Library APIs and cache formats are unchanged
  ([#10](https://github.com/dragginzgame/ic-build/issues/10)).

## [0.2.0] - 2026-10-13

- Document full LTO as a candidate shared deployable-Wasm release default,
  using the existing profile/environment API. Record IC Timers' linked-canister
  measurements and require complete-application qualification through final
  transforms before adoption; engine defaults and public APIs are unchanged.

- **Breaking:** align the public Cargo grouping/capture boundary with IC Host
  0.17. Consumers supplying `OutputLimits` or matching typed Host failures must
  select Host 0.17. Artifact, filesystem and process dependencies now share that
  family; retained artifact and cache formats are unchanged
  ([#9](https://github.com/dragginzgame/ic-build/issues/9)).
- Keep release-adapter tests independent of the live draft by supplying their
  own disposable patch notes, so a pending minor release can be validated.
- Prepare and admit a pinned Cargo editor through shared tooling, so fresh CI
  and release preparation do not depend on a global `cargo set-version`.
  Failed adapter fixtures show bounded inner diagnostics and retain full logs
  ([#11](https://github.com/dragginzgame/ic-build/issues/11)).

## [0.1.4] - 2026-10-13

- Qualify the extracted library, tests and examples with warning-denied Rust
  Clippy, and include that check in `make check` / `make ci`. Document public
  error contracts and preserve source leases through their final consumers;
  prepare Clippy through the declared development toolchain
  ([#8](https://github.com/dragginzgame/ic-build/issues/8)).
- Declare the development Wasm target in `rust-toolchain.toml`, so Rustup
  prepares it on fresh runners when installing the active toolchain. Preserve
  the prepared compiler selection and real positive-heartbeat build
  ([#6](https://github.com/dragginzgame/ic-build/issues/6)).

## [0.1.3] - 2026-10-13

- Add temporary artifact staging and bounded capture of completed producer outputs.
  Keep captured bytes alive across later compilations; callers retain recipes,
  qualification and release publication. This prepares Canic's staging removal
  ([Canic #529](https://github.com/dragginzgame/canic/issues/529)); consumer adoption
  requires this API to be published.
- Carry the repository-prepared compiler through the test gate into independent
  real Wasm fixtures, so a different ambient default does not lose the installed
  Wasm target. Keep heartbeat rejection and positive-build qualification
  ([#6](https://github.com/dragginzgame/ic-build/issues/6)).

## [0.1.2] - 2026-10-12

- Prepare the locked workspace dependency cache on every CI host before offline
  validation, so fresh runners can compile the package and run its gates
  ([#5](https://github.com/dragginzgame/ic-build/issues/5)).

## [0.1.1] - 2026-10-12

- Preserve typed Cargo grouping failures, including original status and bounded
  stdout/stderr, through Host's existing execution error
  ([#3](https://github.com/dragginzgame/ic-build/issues/3)).
- Reject duplicate and empty package inputs before invoking the Cargo command
  factory, reporting the offending input and zero-based indices. Empty package
  lists remain empty plans; valid grouping semantics are unchanged
  ([#4](https://github.com/dragginzgame/ic-build/issues/4)).
- Align current documentation and repository description with published 0.1.0,
  add a direct dependency/import example, and distinguish pending consumer/native
  acceptance from registry availability
  ([#2](https://github.com/dragginzgame/ic-build/issues/2)).
- Enable the shared release workflow with workspace/lock/changelog preparation
  and recovery, and separate clean tagged-source publication to crates.io.
  Release and publication retain artifacts; cleanup remains explicit.
- Record qualification of Testkit's extracted generic artifact/Wasm engine and
  behavioral/process tests, and Canic's feature-equivalent Cargo grouping. Retain
  the existing cache formats and namespaces; consumers select recipes and retain
  their release/runtime policy
  ([#1](https://github.com/dragginzgame/ic-build/issues/1)).
- Run the extracted behavioral, cross-process and real Cargo/Wasm tests in the
  package gate and include the preserved MIT license notice in package contents.
- Keep SPDX MIT metadata without the redundant license-file field; package the
  original notice as a regular crate-local file, compatible with the shared
  formatting hook and without Cargo warnings.
- Adopt Shared Tooling 0.9.5 with bounded tool-download retries, complete Git
  identity checks and hardened evidence/path verification
  ([Shared #138](https://github.com/dragginzgame/shared-tooling/issues/138),
  [Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).

## [0.1.0]

- Record the build ownership and extraction design for the planned common engine.
- Reserve initial Cargo workspace metadata; no build-engine package is implemented
  or released yet.
- Adopt Shared Tooling 0.9.1 and prepare the complete common developer toolset
  through its shared executable store.
- Add the common contributor Make interface, manifest formatting hook, explicit
  development toolchain and bootstrap CI for Linux and both macOS 15 architectures,
  with pinned validation tools and retained failure evidence.
- Add an unpublished `ic-build` library scaffold in the standard `crates/`
  layout, inheriting workspace metadata and lints, with a root lockfile, source
  formatting and compilation/documentation/MSRV gates. The engine remains planned.
