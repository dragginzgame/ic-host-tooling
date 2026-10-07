# Changelog

## [0.4.0]

### Breaking

- Consolidate filesystem reads under `ic-host-fs::read`, remove unbounded and
  duplicate durable readers, and retain typed read, private-file and lock
  failures. Consumers must supply read budgets, update imports and distinguish
  missing private files from rejected or unreadable files. Replace `lock_file`
  with the existing typed lock API; progress locks now return that same error
  ([#14](https://github.com/dragginzgame/ic-host-tooling/issues/14)).
  See [the 0.4 migration notes](docs/changelog/0.4.md).

### Fixed

- Refuse package uploads unless each prepared archive records the clean selected
  source and that exact commit is retrievable from the declared repository;
  preserve preparation and retrieval evidence before dispatch
  ([#7](https://github.com/dragginzgame/ic-host-tooling/issues/7)).

## [0.3.3] - 2026-10-07

### Added

- Capture caller-configured commands through the same bounded process engine as
  admitted tools, preserving caller-owned executable policy and command setup
  ([#5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).
- Borrow retained process and version evidence directly from `ToolError`, including
  original execution and cleanup failures, without copying captured output
  ([#9](https://github.com/dragginzgame/ic-host-tooling/issues/9)).

### Fixed

- Qualify path traversal against each host's native canonicalization results,
  fixing a Linux-specific test expectation that failed on both macOS architectures
  ([#1](https://github.com/dragginzgame/ic-host-tooling/issues/1)).

## [0.3.2] - 2026-10-07

### Added

- Add a response-only tools profile by disabling default features. Keep Candid
  extraction enabled by default with its required dependency owners
  ([#3](https://github.com/dragginzgame/ic-host-tooling/issues/3)).
- Share bounded `HashingWriter` for push serializers, with sink recovery and raw
  identities for accepted bytes; completion and publication remain caller-owned
  ([#4](https://github.com/dragginzgame/ic-host-tooling/issues/4)).

### Fixed

- Resolve dangling symlink targets and retain native directory-traversal errors
  when normalizing missing paths. Add a caller-selected limit for nested missing
  symlink targets so consumers can retain their existing resolution budgets
  ([#1](https://github.com/dragginzgame/ic-host-tooling/issues/1)).
- Adopt Shared Tooling 0.1.14: independently verify snapshots, keep release pushes
  bound to the captured destination and reject inherited Make modes that hide
  failures or skip formatting/release execution. Retain existing tool pins and
  virtual workspace package locations, with the shared `apps/` allowance
  ([#6](https://github.com/dragginzgame/ic-host-tooling/issues/6)).

## [0.3.1] - 2026-10-07

### Added

- Share missing-suffix path resolution with an explicit relative-path base
  ([#1](https://github.com/dragginzgame/ic-host-tooling/issues/1)).
- Share descriptor lock acquisition with caller-selected wait observations and
  polling, retrying interruptions without changing the existing path helper's
  one-second reporting policy
  ([#2](https://github.com/dragginzgame/ic-host-tooling/issues/2)).

- Add separate `make publish-check` and `make publish` commands for verified
  workspace publication to crates.io, retaining logs for partial-upload recovery.
  Enable all four packages for that registry and include their repository link
  and existing MIT license text.
- Share streamed durable replacement, exact byte-stream comparison and bounded
  zero-timestamp gzip encoding so consumers can remove their local mechanics
  while retaining cache, serialization and publication policy.

### Fixed

- Read release versions through the shared Cargo/TOML helper and preserve valid
  inline version comments during preparation and recovery. Prepare the existing
  pinned jq/yq set before version queries or release entry points.
- Compare large changelog version components exactly, keeping future drafts
  distinct from undated release history
  ([Shared Tooling #23](https://github.com/dragginzgame/shared-tooling/issues/23)).
- Keep publication staging names short enough for maximum-length destination
  names, retry collisions without changing unowned files, and exclude the
  selected destination from staging allocation
  ([IC Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13)).

- Keep standard MIT license metadata without Cargo's redundant license-file warning,
  while retaining the canonical license notice in every crate archive. Use regular
  license copies compatible with the formatting hook, and check their contents
  against the root notice before packaging or publication.

### Testing

- Isolate descriptor close/reacquisition checks from parallel subprocess tests,
  preserving immediate release assertions without transient inherited-descriptor
  contention ([#2](https://github.com/dragginzgame/ic-host-tooling/issues/2)).

## [0.3.0] - 2026-10-06

### Breaking

- Split generic artifact, filesystem and executable tooling into ic-host-artifacts,
  ic-host-fs and ic-host-process. Consumers must import moved APIs from their owning
  crates; ic-host-tools retains Candid extraction and ICP response decoding.

### Added

- Start the four-crate extraction workspace with optional artifact parser features
  and Canic's durable local-file publication and locking implementation.
  Packages remain unpublished while extraction and consumer qualification proceed.

### Fixed

- Allow the standard Git release workflow while Cargo packages remain
  non-publishable; registry publication stays separate and disabled.
- Reject failed formatter prerequisite probes and competing pending changelog
  candidates using the reviewed shared helpers.

### Changed

- Require only host parsers for library release qualification; the IC executable
  bundle remains explicit setup.

### Testing

- Exercise the artifact library without optional features in native CI alongside
  the all-feature package checks.

## [0.2.0] - 2026-10-06

- **Breaking:** Replace the exposed wasmparser error with host-owned
  `wasm::ParseError`, keeping parser diagnostics private. Use its `offset()`
  (`u64`) and `message()` accessors; the public conversion from
  `wasmparser::BinaryReaderError` is removed. Update the internal parser to 0.261.0.
  Wasm facts and this library's own error offsets retain their `usize` types.
- **Breaking:** `ExecutionFailure::Io.operation` now uses `ExecutionOperation`.
  Match typed process-operation variants instead of string labels; diagnostics,
  retained output and cleanup evidence are preserved.
- Add JSON consumer contracts for exact pretty-encoded counting/hashing and
  independently bounded staging after serialization preflight
  ([#5](https://github.com/dragginzgame/ic-host-tools/issues/5)).
- Refresh Shared Tooling and delegate formatting-hook qualification to its
  common checker, retaining the consumer's unselected Rust edit case
  ([#6](https://github.com/dragginzgame/ic-host-tools/issues/6)).
  Refreshed installers also reject failed tool-version checks explicitly.

## [0.1.14] - 2026-10-06

- Add a tool-bundle consumer contract requiring both executable and runtime
  library digests before publication, with failed staging retained for inspection.
  Document real Linux Binaryen 132 archive qualification using the existing
  shared extractor; native macOS distribution qualification remains separate.

## [0.1.13] - 2026-10-06

- Reject impossible reader/writer byte counts with typed IO failures instead
  of panicking during artifact reads, hashes or staged copies. Reuse the shared
  writer guard and preserve partial output
  ([#5](https://github.com/dragginzgame/ic-host-tools/issues/5)).
- Refresh Shared Tooling and reuse its common release-command checks in local
  adapter fixtures. Share portable file-digest generation with IC installation
  receipts and stop installation on receipt traversal failure.
- Exercise refreshed release helpers in shallow adapter fixtures without
  copying tracked source or weakening dirty-source admission.
- Add offline `make check-doc-links` using the shared local Markdown checker.

## [0.1.12] - 2026-10-06

- Add `artifact::BoundedWriter` for shared serialization/output budgets and
  byte counting, and `artifact::copy_reader` for bounded copying with SHA-256
  identity. Preserve partial output and distinguish source from sink failures;
  callers retain encoding, filesystem and publication policy
  ([#5](https://github.com/dragginzgame/ic-host-tools/issues/5)).
- Provision ripgrep on every native CI host before shared installer fixtures,
  fixing the missing prerequisite that stopped all three 0.1.11 native gates
  ([#1](https://github.com/dragginzgame/ic-host-tools/issues/1),
  [#4](https://github.com/dragginzgame/ic-host-tools/issues/4)).

## [0.1.11] - 2026-10-06

- Fix release adapter fixtures in shallow checkouts by supplying an explicit
  simulated release identity instead of requiring a parent commit. Exercise
  depth-one clones locally as well as in native CI ([#1](https://github.com/dragginzgame/ic-host-tools/issues/1),
  [#2](https://github.com/dragginzgame/ic-host-tools/issues/2)).
- Require tar 0.4.46 or a compatible newer release so consumers cannot retain
  an affected older tar selection; keep the tested lockfile unchanged
  ([#3](https://github.com/dragginzgame/ic-host-tools/issues/3)).
- Adopt shared repository-local parser and IC setup with `make install-tools`
  and offline `make tools-check`. Native CI uses the same reviewed pins and
  installers. Maintainer release preflight prepares missing or changed sets
  automatically before offline validation; ordinary checks never install tools.
  Local setup replaces `make install-yq`
  ([#4](https://github.com/dragginzgame/ic-host-tools/issues/4)).

## [0.1.10] - 2026-10-06

- Add offline dependency declaration checks to CI and release validation through
  refreshed Shared Tooling. Prepare a checksum-verified YAML/TOML parser in
  native CI and expose the same setup as `make install-yq` locally. Report
  missing jq/yq with setup instructions and check declarations in release
  preflight before cache preparation or the full gate.
- Recover interrupted releases from their exact committed Cargo metadata and
  validated notes even when newer fixes exist. Reject staged changes concealed
  by restored working files, and stop the formatting hook on formatter failure.

## [0.1.9] - 2026-10-06

- Allow compatible registry dependency updates in consuming repositories,
  removing the remaining exact flate2, sha2 and wasmparser constraints alongside
  rustix and tar. Retain existing minimums and features; refresh locked rustix,
  tar, Serde and serde_json dependencies.

## [0.1.8] - 2026-10-06

- Allow compatible Serde and serde_json updates in consuming workspaces instead
  of requiring exact patch versions. Keep the tested minimums and existing
  lockfile selection.
- Fix macOS test compilation by creating the FIFO fixture with the native
  `mkfifo` utility instead of a Rust API unavailable on Apple hosts. Keep the
  real FIFO rejection test enabled on every required host.
- Add bounded single-member gzip artifact decoding with payload integrity
  checks and rejection of concatenated members or trailing bytes. Reuse it in
  archive extraction and include a read-only compressed-artifact example.

## [0.1.7] - 2026-10-06

- Add bounded reads from already-open regular files and Unix path reads that
  reject final symlinks and opened special files without waiting for a FIFO
  writer. Caller confinement and missing-file policy remain explicit. Include
  a read-only regular-file inspection example.

## [0.1.6] - 2026-10-06

- Add read-only executable resolution with explicit working directory and search
  order, literal relative paths, canonical symlink resolution and typed failures.
  Tool digest/version admission remains required. Include a local resolver example.
- Fix the macOS Candid fixture to preserve the actual source lookup error instead
  of assuming it matches the error from rejected filename creation. Verify that
  rejection occurs before tool invocation; extraction behavior is unchanged.

## [0.1.5] - 2026-10-05

- Add bounded decoding of ICP CLI JSON, plain hex and labeled hex responses,
  with explicit formats, separate input/output limits and typed failures that
  omit response contents. Candid decoding and request retry policy stay with
  callers. Include a read-only response inspection example.
- Fix native macOS Candid fixtures that assumed every filesystem accepts
  non-UTF-8 filenames. Check exact path arguments for admitted files and typed
  filesystem rejection before tool invocation; extraction behavior is unchanged.

## [0.1.4] - 2026-10-05

- Prepare the locked dependency cache before CI's offline release fixtures and
  library checks, fixing fresh-runner failures on Linux and macOS. Validation
  remains offline after explicit preparation, with no dependency upgrades.
- Add pinned Cargo manifest sorting to formatting, CI and release checks, plus
  `make install-hooks` for staged-file formatting that preserves unrelated edits
  and rejects partial staging. Fix installation through logical repository paths,
  including macOS aliases ([Shared Tooling #1](https://github.com/dragginzgame/shared-tooling/issues/1)).
- Restart failed release preflight or validation through the normal release
  target against current source, retaining earlier evidence. Rerunning the same
  target also reconciles later interruptions at the saved exact version after
  identity checks, without incrementing prepared metadata again.
- Include the local tool pins in isolated publication fixtures, restoring
  `make publish-tools-check` in CI and release validation after formatter adoption.

## [0.1.3] - 2026-10-05

- Enable crates.io publication and add `make package`, `publish-dry-run`,
  `publish-check` and `publish`. Publication requires a clean checkout at the
  matching annotated release tag and preserves Cargo failures without retries.

## [0.1.2] - 2026-10-05

- Add bounded, digest-verified selection of one regular member from a gzip tar
  archive, with exact names and typed rejection of duplicates, corrupt data,
  unsupported records and resource overflow. The API performs no installation.
- Add bounded owned stream reads and a read-only archive inspection example.
- Add bounded Git revision, tree and dirty-status observations through an
  admitted executable, with explicit status scope, retained raw evidence and
  typed failures. Include a runnable inspection example; queries never retry.

## [0.1.1] - 2026-10-05

- Add shared `make release-patch`, `release-minor`, `release-major` and
  `release-resume` commands with offline validation, exact release staging,
  annotated tags, atomic branch/tag pushes and retained recovery evidence.
- Add exact digest/version admission and Unix host-tool execution with explicit
  environment, working directory, output limits and deadline; preserve bounded
  failure evidence without automatic retries or credential/output logging.
- Add bounded Candid extraction and Canic-compatible whitespace normalization,
  retaining tool/source identities and rejecting observed source changes.
  Include a runnable extractor example and deterministic process fixtures.
- Add bounded file reads, streaming raw SHA-256 identities and typed digest
  verification failures, with caller-owned limits and digest authority.
- Add borrowed core Wasm structural facts using `wasmparser`: section payload
  sizes, defined functions, data segments, exports, and custom metadata.
  Consumers retain full validation, optimization, and artifact acceptance policy.
- Add a read-only artifact inspection example and malformed-input/limit fixtures;
  record a repository-wide host ownership audit without changing consumers.

## [0.1.0]

- Establish the initial Rust workspace, crate boundary, extraction instructions,
  and pinned Shared Tooling governance snapshot.
- Configure focused development commands and native Linux/macOS CI; product
  implementations and downstream adoption remain unimplemented.
