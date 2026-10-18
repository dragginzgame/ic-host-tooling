# Changelog

## [0.27.7]

- Run the non-Unix compile check with the MSRV toolchain that owns its prepared
  Wasm target, fixing the CI compiler/target mismatch without changing the
  development compiler or filesystem support contract.
  [Details](docs/changelog/0.27.md).

- Share positive whole-second duration parsing, rounded single-unit display and
  compact two-unit display for Query and Canic. Keep their distinct text policies
  explicit; deadlines, freshness and persisted units remain caller-owned
  ([PR #162 — Duration text](https://github.com/dragginzgame/ic-host-tooling/pull/162)).
  [Details](docs/changelog/0.27.md).

- Share opt-in canonical unsigned `u64` LEB128 admission without allocation or
  new dependencies. Callers retain field diagnostics, semantic limits and trust.
  [Details](docs/changelog/0.27.md).

- Share explicit Unix signal-action installation and restoration with rollback,
  leaving signal choice, registration custody and shutdown policy with callers.
  [Details](docs/changelog/0.27.md).

- Project exact environment/network facts from bounded effective ICP YAML using
  an optional real parser, leaving configuration resolution and funding to callers.
  [Details](docs/changelog/0.27.md).

- Share opt-in bounded ICRC-3 block admission and contiguous-chain verification,
  using Platform's upstream types and leaving tip authentication with callers.
  [Details](docs/changelog/0.27.md).

- Share opt-in native response-bounded Agent construction and exact endpoint
  binding through Platform, leaving endpoint admission and network policy local.
  [Details](docs/changelog/0.27.md).

## [0.27.6] - 2026-10-18

- Share pure ICP call/status command builders and typed installation-state
  projection for Canic and IcyDB, preserving native arguments and caller-owned
  context, execution, budgets and lifecycle
  ([#97 — ICP command facts](https://github.com/dragginzgame/ic-host-tooling/issues/97)).
  [Details](docs/changelog/0.27.md).

## [0.27.5] - 2026-10-18

- Adopt Shared's latest committed Platform dependency-boundary rules and extend
  Host's direct-dependency guard to the known external IC package families.
  Preserve the canonical snapshot's version label and record its exact revision.
  [Details](docs/changelog/0.27.md).
- Adopt Shared Tooling 0.24.2 and replace both local source-evidence adapters with
  canonical helpers, preserving release custody and matching grouped formatting
  across hooks and release checks
  ([#193 — Canonical source-evidence adoption](https://github.com/dragginzgame/ic-host-tooling/issues/193)).
  [Details](docs/changelog/0.27.md).
- Remove unused internal dependency aliases and separate current ownership and
  qualification from historical migration receipts. Preserve public APIs,
  dependency graphs and release/publication admission.
  [Details](docs/changelog/0.27.md).
- Adopt IC Platform 0.1.11 for the existing Candid/parser and optional certification
  boundary, preserving the selected features and upstream dependency versions.
  [Details](docs/changelog/0.27.md).
- Share bounded stream visitation with caller-owned scratch and fresh directory
  allocation with explicit permissions and collision budgets
  ([PR #135 — Build traversal and directory allocation](https://github.com/dragginzgame/ic-host-tooling/pull/135)).
  [Details](docs/changelog/0.27.md).
- Align lock-file admission with its Unix-only dependency, allowing non-Unix
  library compilation with explicit unsupported-operation errors
  ([#194 — Lock platform guards](https://github.com/dragginzgame/ic-host-tooling/issues/194)).
  [Details](docs/changelog/0.27.md).

## [0.27.4] - 2026-10-18

- Share ordered empty/duplicate batch-label admission with exact comparisons and
  typed occurrence positions; callers retain batch execution and failure policy
  ([PR #184 — Batch label admission](https://github.com/dragginzgame/ic-host-tooling/pull/184)).
  [Details](docs/changelog/0.27.md).
- Share exact fixed/compact two-place decimal rendering with explicit scaling
  and half-up rounding, without floating point or new dependencies
  ([#138 — Exact decimal text](https://github.com/dragginzgame/ic-host-tooling/issues/138)).
  [Details](docs/changelog/0.27.md).
- Share canonical UTC second parsing and formatting, preserving expanded years
  and strict calendar admission without clocks or new dependencies
  ([#140 — Canonical UTC text](https://github.com/dragginzgame/ic-host-tooling/issues/140)).
  [Details](docs/changelog/0.27.md).
- Share read-only physical-directory validation with private-parent creation,
  keeping symlink, non-directory and parent-traversal checks in one place
  ([PR #147 — Share read-only physical directory validation](https://github.com/dragginzgame/ic-host-tooling/pull/147)).
  [Details](docs/changelog/0.27.md).

## [0.27.3] - 2026-10-18

- Protect caller-selected in-place exports from overwriting protected paths or
  inodes, preserving existing permissions, unrelated symlinks and typed failures
  ([#153 — Own non-aliasing in-place export writes](https://github.com/dragginzgame/ic-host-tooling/issues/153)).
  [Details](docs/changelog/0.27.md).
- Publish admitted symlink selections with explicit before/after-publication
  errors, preserving immutable bundles when final synchronization fails
  ([#121 — Extract retained tool-bundle publication](https://github.com/dragginzgame/ic-host-tooling/issues/121)).
  [Details](docs/changelog/0.27.md).
- Share cancellable process-group capture and configured-IO execution through the
  existing process engine. Callers retain cleanup, optional deadlines and inherited
  build output; no finite deadline is added
  ([#95 — Expose cancellable group capture](https://github.com/dragginzgame/ic-host-tooling/issues/95)).
  [Details](docs/changelog/0.27.md).
- Keep publication source retrieval independent of Git's default hash format and
  reject ambiguous saved-source member observations before release setup
  ([#191 — Keep source-evidence adapters hash-format independent](https://github.com/dragginzgame/ic-host-tooling/issues/191)).
  [Details](docs/changelog/0.27.md).
- Delegate release metadata backup, installation, caught-failure restoration and
  receipt archiving to pinned Shared helpers. Retain Host's rendering, source-bound
  validation and receipt format. [Details](docs/changelog/0.27.md).
- Route Candid parsing, reply decoding and certification types through published
  IC Platform 0.1.4. Preserve service/certificate admission and aggregate reply
  accounting; decoder diagnostics now use Platform's compact format while
  retaining typed causes. Enforce the direct-dependency boundary in CI and release
  graph checks. [Details](docs/changelog/0.27.md).

## [0.27.2] - 2026-10-18

### Changed

- Adopt Shared Tooling 0.24.0 with production-role labels, matching release
  formatting checks and target-specific normal/build dependency exclusions.
  Common Cargo setup now selects only the two formatters; source-provenance
  protections remain intact. [Details](docs/changelog/0.27.md).
- Use Shared's pinned native-host and pipe-jobserver observers in CI and Make
  integration checks, retaining Host's platform selection and command routing.
- Share checksum collection between read-only and ordered-sync traversal,
  preserving file-before-hash and post-order directory synchronization.
- Render bordered tables without allocating an all-left alignment vector;
  escaping, widths, padding and plain-table alignment remain unchanged.
- Share opened-file metadata admission across regular and no-follow reads and
  hashes, preserving opening policy and refusal ordering. Released tree-descriptor
  and stream-copy interfaces remain supported. [Details](docs/changelog/0.27.md).

### Added

- Preserve standard cache-directory tags and durably replace invalid entries,
  with caller-selected trailing bytes and permissions
  ([PR #183 — Own cache-directory tag recognition and durable replacement](https://github.com/dragginzgame/ic-host-tooling/pull/183)).
- Hold a shared descriptor lock with explicit unlock on guard drop, so duplicated
  descriptors cannot prolong it; independently opened holders remain separate
  ([PR #174 — Extract scoped shared descriptor lock](https://github.com/dragginzgame/ic-host-tooling/pull/174)).
- Decode Candid replies with caller-selected byte/work limits and aggregate byte
  accounting, retaining domain interpretation with consumers
  ([#188 — Centralize bounded Candid reply decoding](https://github.com/dragginzgame/ic-host-tooling/issues/188)).
- Observe metadata-only logical directory sizes through Host FS, preserving
  symlink and sparse-file lengths while consumers retain traversal and cache
  policy ([PR #173 — Extract logical directory-size observation](https://github.com/dragginzgame/ic-host-tooling/pull/173)).

## [0.27.1] - 2026-10-17

### Added

- Include canonical directory checksums, no-follow tree hashing and ordered
  tree synchronization from the 0.26 release line.

## [0.27.0] - 2026-10-17

### Breaking

- Use the same Unicode scalar-column accounting for tables, clipping and
  progress erasure. Table padding for emoji sequences and ligatures can change.
- Move native file identity to `ic_host_fs::identity::FileIdentity`. Import it
  from FS when using Process custody; no compatibility reexport remains.
  [Migration notes](docs/changelog/0.27.md).

### Changed

- Prepare Make fixtures through Shared's declared companion closure, retiring
  Host's selection of the separate Make-fixture copier.
- Include every minor-line migration document in link validation automatically.
- Share native device/inode observations across FS publication and Process
  custody, retaining file-kind admission and namespace policy with their callers.
- Derive directory-publication error traits while preserving generic formatting,
  original causes and failures before or after publication.
- Prepare release fixtures from declared script dependencies using Shared's
  canonical fixture copier.
- Use the hashed executable descriptor's metadata for permission admission,
  preserving size/digest refusal before permission policy. Private-file errors
  retain their messages and original causes through derived error traits.
- Test fixtures borrow their cleanup guard's path instead of storing another
  copy; successful cleanup and panic retention remain unchanged.
- Adopt Shared Tooling 0.23.0 with common host observation, validated Rust-tool
  records and shared transient retries for CI binary installation. Retain both
  Cargo source-evidence helpers through their refreshed Shared contribution.
- Share saved Cargo member-source context with Shared Tooling while retaining
  local package selection and release recovery.
- Consolidate durable writer dispatch while preserving admission ordering and
  publication errors.
- Build progress frames without temporary padding strings, preserving escaping,
  width accounting and rendered output.

### Added

- Render dynamic ASCII-bordered tables through `ic_host_text::table::render_bordered`,
  sharing escaping and width accounting with plain tables. Ragged rows receive
  empty cells or unnamed columns; values, budgets and output stay caller-owned
  ([PR #129 — Dynamic bordered tables](https://github.com/dragginzgame/ic-host-tooling/pull/129)).
- Retain descriptor metadata alongside bounded file hashes through following
  and final-component no-follow reads. Metadata is an observation rather than
  immutable content, pathname custody or atomic execution authority.
- Share bounded newline framing for buffered readers and incremental process
  output, preserving queued messages, refused prefixes and original I/O causes.
  Schemas, deadlines, EOF-tail acceptance and recovery remain caller-owned
  ([#126 — Share bounded newline framing](https://github.com/dragginzgame/ic-host-tooling/issues/126)).
- Copy caller-selected streams through durable publication with optional exact
  output permissions. Failed reads preserve prior output and cleanup evidence;
  source admission, budgets and reuse remain caller-owned
  ([#92 — Share durable stream copying](https://github.com/dragginzgame/ic-host-tooling/issues/92)).
## [0.26.2] - 2026-10-17

### Added

- Synchronize and hash an existing caller-held directory with ordered pre-sync
  hooks through FS `durable::tree::sync_checksum_directory_at`. Files sync before
  hashing; directories sync after children. Callers retain total-tree budgets,
  expected-checksum admission, publication and recovery policy.

## [0.26.1] - 2026-10-17

### Added

- Compose exact directory identities from canonical relative paths and SHA-256
  digests in Artifacts, and hash no-follow directory trees in FS with explicit
  per-file byte limits. Preserve Backup's component ordering and path/NUL/hex/newline
  framing; callers retain tree budgets, custody, synchronization and publication.

## [0.26.0] - 2026-10-17

### Breaking

- Consuming `OwnedDirectory::cleanup()` attempts removal once, including on
  failure. Capture the path before consuming and own any retry or recovery;
  borrowed `try_cleanup()` and ordinary armed Drop retain their contracts.
  [Migration notes](docs/changelog/0.26.md).

### Changed

- Adopt Shared Tooling 0.21.0 with direct validated host-tool records and the
  ICP CLI 1.7.0 catalog. Preserve the provenance helper through its refreshed
  Shared contribution; publication policy and explicit IC-tool setup stay local.
- Share prepared package provenance and exact remote source retrieval with
  Shared Tooling. Host retains eligibility, licenses, packaging and recovery.
- Derive filesystem error formatting and source chains with `thiserror`,
  preserving public variants, sanitized messages and native I/O causes.
  Retain the [PocketIC-compatible dependency floor](docs/changelog/0.26.md#dependency-compatibility).

### Added

- Copy a bounded regular artifact through durable publication with an expected
  digest checked before replacement. Preserve prior output and typed cleanup
  evidence on refusal; paths, reuse and recovery remain caller-owned
  ([#102 — Own digest-verified durable artifact copying](https://github.com/dragginzgame/ic-host-tooling/issues/102)).
- Stream one bounded gzip member into a caller-owned sink while identifying its
  decoded bytes. Integrity and framing checks precede success; errors can retain
  partial output, so staging and publication remain caller-owned
  ([#170 — Stream bounded gzip payloads into caller-owned sinks](https://github.com/dragginzgame/ic-host-tooling/issues/170)).
- Create private invocation files through `ic_host_fs::scratch::create_private_bytes`,
  with explicit collision attempts, close-before-return and separately retained
  write/cleanup failures. Callers own byte budgets, successful cleanup and recovery
  ([#122 — Share private non-durable invocation scratch-file creation](https://github.com/dragginzgame/ic-host-tooling/issues/122)).

## [0.25.1] - 2026-10-17

### Added

- Inspect `HashingWriter`'s sink and snapshot the identity of accepted bytes
  without finalizing or flushing it. Producer success, aliased writes and
  durability remain caller-owned
  ([PR #175 — Inspect hashing-writer sinks and identities](https://github.com/dragginzgame/ic-host-tooling/pull/175)).

### Fixed

- Accept the staged HTTP fixture connection before selecting nonblocking reads,
  avoiding the macOS Intel `WouldBlock` setup race while preserving the header gate
  ([PR #176 — Accept staged HTTP connections before nonblocking reads](https://github.com/dragginzgame/ic-host-tooling/pull/176)).

## [0.25.0] - 2026-10-17

### Breaking

- Retire `write_at_with` and `write_existing_parent_with`. Use their validated
  counterparts with explicit result admission, preserving publication modes,
  permissions and recovery. [Migration notes](docs/changelog/0.25.md).
- Include Candid service inspection in Host Tools unconditionally. Retire the
  `candid-service` feature; remove it from consumer feature lists. Disabling
  defaults no longer excludes `candid_parser` and its `candid` dependency.
- Share one `VersionSpec` across tool observation and verification. Verifying
  entrypoints require the exact version as a separate argument; digest pins
  remain mandatory for pinned admission. Remove `VersionObservationSpec`.
- Remove Host Tools' replica-limit reports, `ic-limits` feature and resource-report
  example. Consumers own replica revision selection and enforcement, using Host
  Artifacts for generic Wasm facts. Canic must adopt its local policy before
  upgrading. [Migration notes](docs/changelog/0.25.md).

### Changed

- Adopt Shared Tooling 0.20.0. Common setup and CI prepare host/Cargo tools;
  IC executables require explicit setup. Preserve tool pins, literal catalogs,
  shared version admission and per-path Rust LOC counts. One delivery-policy
  file now supplies Host's direct-only release admission to Make and its adapter.
- Share test-directory cleanup through an opt-in fixture guard that retains
  failed-test evidence during panic; production directory custody is unchanged.
- Derive response and certificate error formatting/source chains through
  `thiserror`, preserving public error variants and sanitized diagnostics.

## [0.24.0] - 2026-10-16

### Breaking

- Move native Candid extraction to IC Build; remove Host Tools' extraction API,
  feature and example. Consumers use `ic_build::artifacts::candid` after the
  Build release, preserving their tool, normalization and publication policies
  ([Build #51 — Own native Candid extraction](https://github.com/dragginzgame/ic-build/issues/51)).
  [Detailed notes](docs/changelog/0.24.md).

- Remove `artifact::hash_gzip_or_raw` without an alias. Callers select raw
  `hash_reader` or strict `hash_gzip` explicitly and retain their input/payload
  allowances. Gzip encoding, decoding and exact comparison are unchanged.

### Added

- Observe installed tool version and byte identity with one provenance probe;
  retain exact-version and digest-pinned admission as separate contracts
  ([#166 — Observe installed tool version with one probe](https://github.com/dragginzgame/ic-host-tooling/issues/166)).
- Share invocation-owned directory cleanup with explicit preservation and error
  custody; Build retains cache policy and artifact lifecycle
  ([#159 — Own invocation directory custody for Build](https://github.com/dragginzgame/ic-host-tooling/issues/159)).

### Changed

- Add fixed-buffer SHA-256 hexadecimal output and share its encoder with Display,
  preserving lowercase bytes, leading zeroes and formatting behavior
  ([#161 — Fixed-buffer SHA-256 output](https://github.com/dragginzgame/ic-host-tooling/issues/161)).
  [Detailed notes](docs/changelog/0.24.md).
- Add bounded regular-file prefix reads with the descriptor-observed length;
  larger files are permitted without loading them completely
  ([PR #155 — Bounded file-prefix reads](https://github.com/dragginzgame/ic-host-tooling/pull/155)).
- Adopt Shared Tooling 0.16.1, reading snapshot companion headers directly with
  Bash while preserving the selected payloads, compiler policy and tool pins.
- Share fallible bounded buffer growth between stream reads, chunk identities
  and process capture; preserve caller-selected byte limits and overflow policy.
- Keep publication status combinations at their Shared owner; retain Host's
  delegation, preparation-lock cleanup and package/recovery qualification.

## [0.23.1] - 2026-10-16

- Add caller-selected inherited command custody and exact existing-sidecar
  quiescence in Host Process; journal schemas and effect recovery stay local.
  [Detailed notes](docs/changelog/0.23.md).
- Admit existing regular lock sidecars and publish caller-verified directory
  trees without replacing an occupied name. [Detailed notes](docs/changelog/0.23.md).

## [0.23.0] - 2026-10-16

### Breaking

- Move bounded line normalization to Host Text as `normalize::normalize_lines`;
  remove `ic_host_tools::candid::text`. Callers update imports and direct
  dependencies. [Detailed notes](docs/changelog/0.23.md).
- Require an explicit missing-target symlink depth in
  `canonicalize_allow_missing(path, base, depth)` and remove the alternate
  resolver name. Select `usize::MAX` explicitly to preserve unbounded expansion.
- Render process cleanup failures once in explicit failure diagnostics;
  consumers asserting the duplicated diagnostic must update their expected text.
- Remove the combined pathname open-and-lock helpers. Prepare the lock file with
  `open_regular_lock_file_with_parents`, then acquire on its descriptor with
  `lock_exclusive` or the new immediate `try_lock_exclusive` operation.
- Require cleanup policy in `OwnedChild::spawn(command, cleanup)` and remove
  `spawn_with_cleanup`. Select `KillAndWait` to preserve the former default;
  existing bounded policies retain their grace and reap allowances.
- Remove `artifact::verify_reader`; use `hash_reader(reader, limit)?.verify(expected)`
  to hash once and admit the returned identity against the selected digest.

### Added

- Report encoded Wasm indirect-call counts and data payload bytes without a
  subprocess, preserving the existing structural inspector contract
  ([#145 — Bounded encoded Wasm statistics](https://github.com/dragginzgame/ic-host-tooling/issues/145)).
  [Detailed notes](docs/changelog/0.23.md).
- Add pure terminal-safe progress redraw frames with Unicode column padding;
  callers retain terminal detection, output and cleanup policy
  ([#157 — Pure progress-line redraw frames](https://github.com/dragginzgame/ic-host-tooling/issues/157)).

### Changed

- Share no-follow filesystem entry operations across root-relative reads and
  tree walking, preserving traversal and error contracts.
- Admit and convert durable-write permissions once at entry, preserving early
  refusal and the existing staging/publication checks.
- Admit synthetic HTTP capture bounds once and reuse the parsed header length
  while reading body fragments, preserving deadlines and partial-byte evidence.
- Adopt Shared Tooling 0.16.0 with explicit CI installation destinations and
  narrower consumer probes. Fleet LOC commands now remain at Shared's owner;
  Host keeps local LOC and its actual target checks. [Detailed notes](docs/changelog/0.23.md).

## [0.22.1] - 2026-10-16

- Adopt Shared Tooling 0.15.7, strengthening fixture failure-retention checks
  while preserving the selected tooling, package versions and library APIs.
  [Detailed notes](docs/changelog/0.22.md).

## [0.22.0] - 2026-10-16

### Breaking

- Require explicit `WriteOptions` for `durable::write_bytes`; retire the two
  create-only byte wrappers. Callers select publication mode and permissions.
- Return raw Candid extraction evidence without a duplicate normalized string.
  Callers needing normalized text invoke `candid::text::normalize` explicitly.
- Retire the Artifacts `archive` feature and tar-member extraction API, its tar
  dependency and filesystem example. Gzip support remains available separately.
- Remove labeled-hex response decoding. Select JSON or plain hex explicitly;
  CLI labels and preambles remain the caller's presentation policy. The saved
  response example now accepts only `json` or `hex`.
- Add `ArtifactError::LengthMismatch` for an exact stream that ends early.
  Consumers exhaustively matching artifact errors must handle the new variant.

### Changed

- Add durable publication beneath an existing parent, retaining descriptor
  custody and refusing missing or special-file parents before production
  ([#148 — Existing-parent pathname publication](https://github.com/dragginzgame/ic-host-tooling/issues/148)).
- Share explicitly requested process failure diagnostics, including captured
  streams and cleanup evidence, while ordinary error display keeps omitting them
  ([#149 — Explicit process failure-evidence rendering](https://github.com/dragginzgame/ic-host-tooling/issues/149)).
- Route descriptor publication through one adapter, preserving producer results,
  validation, publication errors and cleanup behavior.
- Adopt Shared Tooling 0.15.6 at `5910c9f`, sharing CI tool preparation and Make
  fixture setup while preserving common tool pins, package versions and MIT license.
- Clip terminal text while escaping, avoiding a complete escaped intermediate
  and preserving existing Unicode, control and ellipsis output.
- Keep generic formatter/hook, Make-consumer and tool-command regression suites
  in Shared Tooling while retaining Host's actual integration, preservation and
  prepared-tool checks.
  [Detailed notes](docs/changelog/0.22.md).

### Added

- Add exact-length streaming hash and bounded read operations to Host Artifacts.
  They reject premature EOF and probe one excess byte; callers still select
  byte limits and own stream custody.
- Open regular files or directories through a retained Unix root descriptor,
  refusing traversal, symlinks and special entries without pathname reopening.
  Callers retain root and byte custody
  ([#80 — Capability-rooted filesystem confinement](https://github.com/dragginzgame/ic-host-tooling/issues/80)).
  [Detailed notes](docs/changelog/0.22.md).
- Share bounded synthetic HTTP request capture through Process's opt-in
  `test-support` module, retaining partial bytes and typed failures. Consumers
  own responses and evidence publication
  ([#123 — Own bounded synthetic HTTP request fixtures](https://github.com/dragginzgame/ic-host-tooling/issues/123)).
  [Detailed notes](docs/changelog/0.22.md).

## [0.21.0] - 2026-10-15

### Breaking

- Candid service inspection no longer enables Wasm export inspection implicitly.
  Select both `candid-service` and `ic-interface` when both APIs are needed;
  Candid-only consumers no longer select Host Artifacts or the Wasm parser.

### Changed

- Simplify durable publication's internal admission modes while preserving
  closed-writer validation, cleanup and publication error states.

### Fixed

- Derive Git's dirty observation from retained status bytes, so editing report
  metadata cannot change it.
- Admit optional-file descriptor metadata once in the consuming reader, retaining
  no-follow opening, private-file checks and rejection before reading.
  [Detailed notes](docs/changelog/0.21.md).

## [0.20.1] - 2026-10-15

- Add terminal-line clipping with explicit column limits, control escaping and
  an ellipsis for host status panels. Callers retain terminal and redraw policy
  ([#144](https://github.com/dragginzgame/ic-host-tooling/pull/144)).
  [Detailed notes](docs/changelog/0.20.md).

## [0.20.0] - 2026-10-15

### Breaking

- Remove `ToolSpec`: pinned executable admission now takes `VersionSpec` plus
  a mandatory digest argument. Update both `admit` and `admit_no_follow` callers;
  digest verification still precedes version execution.
- Borrow Git revision/tree IDs from retained captures through accessors, replacing
  duplicated public strings and mutable capture fields.
- Retain gzip decoding failures as `ArchiveError::Gzip(GzipError)` instead of
  parallel archive variants; update matching and error-chain handling.
- Require explicit `WriteOptions` for named publication so callers select
  replacement/create-only behavior and staging permissions.
- Return borrowed-or-owned terminal text with `Cow<str>`, avoiding copies of safe
  cells. Callers requiring `String` must explicitly take ownership.

### Changed

- Simplify durable-write failure handling and table rendering, and consolidate
  ordered Git failure evidence.
- Add optional digest verification and detailed Wasm records to the inspector,
  replacing the overlapping filesystem example while preserving default TSV.
- Derive publication paths from admitted Cargo metadata and share Unix read-only
  opening mechanics across following and no-follow reads.
- Adopt Shared Tooling 0.15.2 and its Cargo ordering-fixture generator, with
  explicit fixture variable selection and unchanged common tool pins
  ([#134 — Shared Cargo ordering fixture](https://github.com/dragginzgame/ic-host-tooling/issues/134)).
- Keep generic installer/LOC regression matrices in Shared Tooling while retaining
  Host's actual wiring, failure propagation, tool admission and native checks.
- Share identical invalid-count artifact test doubles privately, retaining each
  API's rejection assertions and distinct partial-I/O cases.
- Reuse filesystem fixture allocation in CLI/example tests and retire unused
  Shared regression exports while keeping active checkers and integration proofs.
- Consolidate crash-barrier polling and inspector output errors, preserving
  marker/child/timeout precedence and admission before output.
- Share descriptor-lock dispatch and failing-reader fixtures. Use Shared's
  saved-source renderer instead of Host's duplicate dependency projector
  ([#131](https://github.com/dragginzgame/ic-host-tooling/issues/131)).
  [Detailed notes](docs/changelog/0.20.md).

## [0.19.0] - 2026-10-15

### Breaking

- Require explicit native-command capture limits for RSS sampling. macOS capture
  now retains bounded diagnostics and rejects incomplete tables; callers must
  pass `OutputLimits` and handle retained command failures. Optional deadlines
  remain supported without a default time limit
  ([#130 — Native RSS command capture](https://github.com/dragginzgame/ic-host-tooling/issues/130)).
- Reject oversized Linux stat records before parsing, preventing a cutoff RSS
  field from becoming a plausible smaller observation
  ([#130 — Native RSS command capture](https://github.com/dragginzgame/ic-host-tooling/issues/130)).
- Reject ancestor symlinks and `..` in private-parent preparation before creating
  directories. Callers intentionally selecting directory aliases must supply
  canonical physical paths
  ([PR #90 — Reject symlink ancestors](https://github.com/dragginzgame/ic-host-tooling/pull/90)).

### Changed

- Consolidate real FIFO fixture creation so filesystem tests share one portable
  Linux/macOS setup path while retaining their individual modes and assertions.
  [Detailed notes](docs/changelog/0.19.md).

## [0.18.6] - 2026-10-15

- Record Shared Tooling 0.15.0 as the contributor baseline; selected helpers and
  tool pins are unchanged, with no tool reinstall required.
  [Detailed notes](docs/changelog/0.18.md).

## [0.18.5] - 2026-10-15

- Adopt integrated Shared release-input validation and fixture-retention probes,
  preserving tool pins and using the formatting helper now delivered on Shared main
  ([#124 — Shared release and fixture adoption](https://github.com/dragginzgame/ic-host-tooling/issues/124)).
  [Detailed notes](docs/changelog/0.18.md).

## [0.18.4] - 2026-10-15

- Adopt Shared Tooling 0.14.0 reporting and changelog guidance while retaining
  current-source formatting discovery; tool pins and executable behavior stay unchanged
  ([#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)).
- Fix the tree-walk fixture on filesystems that reject non-UTF-8 filenames,
  preserving symlink refusal and native filename-admission coverage
  ([#116 — Release and fixture cleanup](https://github.com/dragginzgame/ic-host-tooling/issues/116)).
  [Detailed notes](docs/changelog/0.18.md).

## [0.18.3] - 2026-10-14

- Prepare release metadata with literal rendering and byte/mode rollback,
  retiring the release-only cargo-edit installation and commands
  ([#116 — Release and fixture cleanup](https://github.com/dragginzgame/ic-host-tooling/issues/116)).
- Use fork-safe executable fixture writers and remove global test serialization;
  finalize the new tree walker's directory-only admission inside the library
  ([#116 — Release and fixture cleanup](https://github.com/dragginzgame/ic-host-tooling/issues/116)).
- Discover current Rust sources and crate manifests for formatting fixtures,
  replacing the manual module roster through Shared's canonical helper
  ([#113 — Fixture and graph simplification](https://github.com/dragginzgame/ic-host-tooling/issues/113)).
- Reuse Shared fixture allocation and consolidate feature-graph checks,
  preserving failed evidence and every dependency-boundary assertion
  ([#113 — Fixture and graph simplification](https://github.com/dragginzgame/ic-host-tooling/issues/113)).
- Add a no-follow descriptor tree walker in `ic-host-fs` for caller-owned
  checksums, synchronization and entry policy; it does not claim tree custody.
- Add opt-in acknowledged direct-child crash barriers in `ic-host-process`
  testing, with explicit timeouts and direct-child reaping on handshake failure.
  [Detailed notes](docs/changelog/0.18.md).

## [0.18.2] - 2026-10-14

- Add optional strict owned IC certificate/hash-tree decoding, with explicit byte
  and nesting limits; authentication and witness policy remain caller-owned
  ([#109 — Owned certificate decoding](https://github.com/dragginzgame/ic-host-tooling/issues/109)).
- Add `ic-host-text` for pure terminal-safe escaping and Unicode-aware tables,
  with explicit formatting choices and no terminal/output policy
  ([#110 — Host text ownership](https://github.com/dragginzgame/ic-host-tooling/issues/110)).
- Adopt Shared Tooling 0.13.1, including its reusable fixture allocation helper
  and publication-guide export correction; contributor tool pins stay unchanged
  ([#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)).
  [Detailed notes](docs/changelog/0.18.md).

## [0.18.1] - 2026-10-14

- Add opt-in filesystem fixture allocation and reuse it across filesystem,
  process and tools tests, preserving caller-owned cleanup and retention
  ([#98 — Fixture directory allocation](https://github.com/dragginzgame/ic-host-tooling/issues/98)).
- Use canonical bounded file admission in the IC resource-report example and
  upstream integer encoding in its test builder
  ([#106 — Resource-report example cleanup](https://github.com/dragginzgame/ic-host-tooling/issues/106)).
- Hash admitted raw payload slices directly while preserving both byte allowances
  and typed failures. Consolidate publication fixtures and failure-retention probes
  around Cargo metadata and the existing Shared helper
  ([#105 — Raw hashing and fixture simplification](https://github.com/dragginzgame/ic-host-tooling/issues/105)).
  [Detailed notes](docs/changelog/0.18.md).

## [0.18.0] - 2026-10-14

### Breaking

- Following filesystem reads/hashes now open first, then admit descriptor type
  and size. Callers must handle native open failures before metadata rejection
  ([#96 — Archive, read and lock projection](https://github.com/dragginzgame/ic-host-tooling/issues/96)).

### Changed

- Hash admitted immutable archive slices directly, retaining digest-before-decode
  ordering and mismatch evidence ([#96 — Archive, read and lock projection](https://github.com/dragginzgame/ic-host-tooling/issues/96)).
- Delegate saved-source release lockfile projection to Shared Tooling while
  retaining exact payload and historical recovery checks ([#96 — Archive, read and lock projection](https://github.com/dragginzgame/ic-host-tooling/issues/96)).

- Adopt Shared Tooling 0.12.7 with clearer release-note layout and reviewed
  CI-selection/toolchain guidance
  ([#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)).

- Add retained private-directory creation and pathname admission, preserving creation
  phase, owner-only modes and caller-owned namespace custody.
  ([#94 — Directory and buffered-output custody](https://github.com/dragginzgame/ic-host-tooling/issues/94)).
- Complete retained buffered child stdout through the existing bounded capture
  engine, preserving prefetched bytes, cached exits and cleanup ownership.
  ([#94 — Directory and buffered-output custody](https://github.com/dragginzgame/ic-host-tooling/issues/94)).

- Add bounded generic Wasm custom-section encoding so consumers can share framing
  while retaining metadata and module-assembly policy
  ([#77 — Bounded Wasm custom-section encoder](https://github.com/dragginzgame/ic-host-tooling/issues/77)).
  [Detailed notes](docs/changelog/0.18.md).

## [0.17.13] - 2026-10-14

- Adopt Shared Tooling 0.12.5 and align contributor guidance with focused
  handoff checks and explicitly requested full local validation
  ([#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.12] - 2026-10-14

- Adopt Shared Tooling 0.12.4, sharing host/IC tool-selection admission and
  extending installation, reuse and failed-evidence fixture coverage
  ([#73 — Shared Tooling adoption](https://github.com/dragginzgame/ic-host-tooling/issues/73)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.11] - 2026-10-14

- Expose durable owner-only parent preparation so callers can retain private
  directory permissions while using descriptor-based file publication.
- Own blocking shared/exclusive regular-descriptor locks and explicit unlock;
  callers retain descriptor, namespace and final-owner custody
  ([#89](https://github.com/dragginzgame/ic-host-tooling/issues/89)).

## [0.17.10] - 2026-10-14

- Restore macOS test compilation with portable FIFO creation while retaining
  regular-file and tool-admission refusal checks
  ([#85](https://github.com/dragginzgame/ic-host-tooling/issues/85),
  [#88](https://github.com/dragginzgame/ic-host-tooling/issues/88)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.9] - 2026-10-14

- Adopt Shared Tooling 0.12.2 for shared failure-evidence collection and stricter
  historical release-note admission
  ([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73)).
- Expose a following regular-file opener that retains the selected descriptor
  and refuses Unix FIFOs without waiting for a writer; reuse it for bounded reads
  ([#88](https://github.com/dragginzgame/ic-host-tooling/issues/88)).

- Add opt-in Unix executable-script fixtures that use an owned writer process
  to avoid inherited writable descriptors, allowing consumers to share the
  mechanism while keeping fixture policy local
  ([#87](https://github.com/dragginzgame/ic-host-tooling/issues/87)).

- Delegate publication attempts to Shared Tooling while retaining Host admission,
  original Cargo/logger statuses and interrupted-attempt evidence
  ([#75](https://github.com/dragginzgame/ic-host-tooling/issues/75)).
- Share child-owner initialization and remove unnecessary CLI test buffering;
  public APIs and process ownership remain unchanged.
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.8] - 2026-10-14

- Let custody-aware callers transfer an already spawned direct child into Host's
  bounded IO and cleanup owner, preserving one-spawn authority.

- Adopt Shared Tooling 0.12.0 and delegate strict release-note admission to its
  canonical finalizer, removing duplicate local guards
  ([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73),
  [#75](https://github.com/dragginzgame/ic-host-tooling/issues/75)).
- Fix retained staging compilation on macOS by reusing checked native permission
  conversion; retain invalid-mode refusal and failed staging evidence
  ([#84](https://github.com/dragginzgame/ic-host-tooling/issues/84)).
- Simplify trusted test-tool admission and consolidate version coverage while
  retaining explicit digest-pinned admission checks
  ([#85](https://github.com/dragginzgame/ic-host-tooling/issues/85)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.7] - 2026-10-13

- Add observed-identity digest verification and reuse shared fixture cleanup and
  Cargo-derived test workspace setup
  ([#82](https://github.com/dragginzgame/ic-host-tooling/issues/82)).

- Share Git observation failure handling while preserving query order and captured
  evidence; cover newer Cargo targets in Make execution probes and document the
  complete publication set ([#83](https://github.com/dragginzgame/ic-host-tooling/issues/83)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.6] - 2026-10-13

- Adopt Shared Tooling 0.11.3's publication-admission guidance; selected executable
  tooling and Host's publication contract are unchanged
  ([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.5] - 2026-10-13

- Adopt Shared Tooling 0.11.2: release recovery shares annotated-tag validation,
  and shared fixtures preserve cleanup failures and physical temporary roots
  ([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73),
  [Shared #166](https://github.com/dragginzgame/shared-tooling/issues/166)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.4] - 2026-10-13

- Cover every workspace package in release projection and CI, including the Wasm
  CLI, and record the complete selected publication set
  ([#75](https://github.com/dragginzgame/ic-host-tooling/issues/75)).
- Adopt Shared Tooling 0.11.0, retiring the old CI installer forwarding command
  without changing tool pins ([#73](https://github.com/dragginzgame/ic-host-tooling/issues/73)).
- Share equivalent feature-boundary checks and hash artifact-example bytes directly
  while retaining expected-digest verification.

- Stream regular-file example hashing and avoid copying decoded responses for
  hashing; simplify the tools fixture's fixed admission flow
  ([#78](https://github.com/dragginzgame/ic-host-tooling/issues/78)).

- Add no-follow executable admission, native process-tree RSS observations and
  post-sync validation beneath an admitted publication directory, allowing
  Testkit to remove duplicate admission, sampling and publication mechanics
  ([#79](https://github.com/dragginzgame/ic-host-tooling/issues/79)).

- Add caller-named retained staging and no-replace publication beneath an admitted
  directory descriptor, preserving failed inputs through the durable engine.

- Share release-payload comparisons between prepared and committed checks,
  preserving source-bound historical recovery
  ([#75](https://github.com/dragginzgame/ic-host-tooling/issues/75)).
  [Detailed notes](docs/changelog/0.17.md).

- Add the standalone `ic-host-wasm-inspect` CLI package,
  using canonical bounded file reads and Wasm facts. It retains explicit budgets,
  exact input hashes, TSV columns, typed failures and no report on input rejection.
  Response-only defaults and parser/process feature boundaries stay unchanged.

## [0.17.3] - 2026-10-13

- Adopt Shared Tooling 0.10.2, retaining the 0.10.1 fix so unrelated catalog
  changes no longer invalidate Cargo CI caches; tool selection and build changes
  still invalidate them
  ([Shared #154](https://github.com/dragginzgame/shared-tooling/issues/154)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.2] - 2026-10-13

- Simplify subprocess test fixtures to their crate-specific modes and remove an
  unused tools test dependency ([#74](https://github.com/dragginzgame/ic-host-tooling/issues/74)).
  Preserve failed IC-limits example inputs and skip occupied temporary names
  ([#72](https://github.com/dragginzgame/ic-host-tooling/issues/72)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.1] - 2026-10-13

- Retain failed process/tools test fixtures, skip occupied fixture paths without
  deleting evidence, and test extraction/IC-limits features independently in CI
  ([#72](https://github.com/dragginzgame/ic-host-tooling/issues/72)).
- Adopt Shared Tooling 0.10.0: the reviewed snapshot owns the default common
  tool catalogs, with Host's existing versions and digests unchanged
  ([Shared #153](https://github.com/dragginzgame/shared-tooling/issues/153)).
  Retain maintenance and LOC rejection of directory
  paths containing line breaks; release lock failures preserve native errors
  and statuses ([Shared #150](https://github.com/dragginzgame/shared-tooling/issues/150),
  [#151](https://github.com/dragginzgame/shared-tooling/issues/151),
  [#152](https://github.com/dragginzgame/shared-tooling/issues/152)).
  [Detailed notes](docs/changelog/0.17.md).

## [0.17.0] - 2026-10-13

- **Breaking:** Wasm-only `ic-interface` inspection no longer enables the Candid
  parser. Select `candid-service` when using `candid::service`.
- Share copy hashing with the existing hashing writer and consolidate durable
  filesystem test fixtures, retaining failed fixtures for diagnosis.
  [#72](https://github.com/dragginzgame/ic-host-tooling/issues/72).
  [Detailed notes and migration](docs/changelog/0.17.md).

## [0.16.0] - 2026-10-13

- **Breaking:** `ic-host-tools` now has no default features. Enable
  `candid-extraction` explicitly when using the native extraction API; response
  decoding and Candid text normalization keep their minimal dependency graph.
- Reuse Shared Tooling's Cargo-tool cache in native CI while retaining complete
  tool admission and Linux/macOS coverage.
  [Detailed notes and migration](docs/changelog/0.16.md).
- Adopt Shared Tooling 0.9.13: routine development and validation may fetch
  already-locked Cargo sources without another approval
  ([Shared #147](https://github.com/dragginzgame/shared-tooling/issues/147)).
  Retain tool-lock and source-acquisition failure statuses and diagnostics
  ([Shared #145](https://github.com/dragginzgame/shared-tooling/issues/145),
  [Shared #146](https://github.com/dragginzgame/shared-tooling/issues/146)).

## [0.15.2] - 2026-10-13

- Adopt Shared Tooling 0.9.10, retaining the fix for Rust-tool qualification from
  consumer roots and removing its synthetic Git commits
  ([Shared #143](https://github.com/dragginzgame/shared-tooling/issues/143),
  [Shared #144](https://github.com/dragginzgame/shared-tooling/issues/144)).
  Includes Cargo-tool cache identity/restore coverage
  ([Shared #142](https://github.com/dragginzgame/shared-tooling/issues/142)).
  Retain 0.9.7's rejection of non-boolean Cargo offline configuration and its
  explicit Cargo-tool setup requirement; previous builds are retained
  ([Shared #140](https://github.com/dragginzgame/shared-tooling/issues/140)).
  [Detailed notes](docs/changelog/0.15.md).

## [0.15.1] - 2026-10-12

- Adopt Shared Tooling 0.9.6: require complete Git identities in snapshots and
  release recovery, retaining malformed evidence for diagnosis
  ([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).
  [Detailed notes](docs/changelog/0.15.md).

## [0.15.0] - 2026-10-12

- **Breaking:** split Candid normalization and extraction into `candid::text` and
  `candid::extraction`; update imports directly. Pure normalization no longer
  requires Unix or process dependencies.
- Add optional parser-backed Candid service inventories and IC Wasm method facts,
  retaining method modes and caller-owned interface policy. Consumers can share
  interpretation without expanding endpoint type trees
  ([#71](https://github.com/dragginzgame/ic-host-tooling/issues/71)).
  [Detailed notes and migration](docs/changelog/0.15.md).
- Adopt Shared Tooling 0.9.3: retry transient Host/IC tool downloads within bounded
  attempts and time limits, retaining failed candidates and previous selections
  ([Shared #138](https://github.com/dragginzgame/shared-tooling/issues/138)).
  [Detailed notes](docs/changelog/0.15.md).

## [0.14.10] - 2026-10-12

- Adopt Shared Tooling 0.9.2: reject NUL bytes in evidence checksum manifests
  and invalid consumer paths before installer setup. Host and IC installers share
  cleanup handling, releasing the shared lock even when checkout cleanup fails
  while preserving the original installation error and failed candidate.
  [Detailed notes](docs/changelog/0.14.md).
- Isolate the private-lock release test from unrelated subprocess creation,
  avoiding intermittent contention without weakening its immediate-release check
  ([#70](https://github.com/dragginzgame/ic-host-tooling/issues/70)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.9] - 2026-10-12

- Adopt Shared Tooling's 0.9.0 manifest-reader changes, sharing syntax validation
  across refresh and verification and rejecting embedded NUL bytes
  ([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).
  Retains 0.8.4's JSON workspace line counts and clarified snapshot guidance.
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.8] - 2026-10-12

- Adopt Shared Tooling 0.8.3: protect installer diagnostic paths
  ([Shared #136](https://github.com/dragginzgame/shared-tooling/issues/136)),
  honor the selected Rust formatter pins
  ([Shared #135](https://github.com/dragginzgame/shared-tooling/issues/135)),
  and make snapshot payload replacement atomic
  ([Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)).
  Includes the centrally owned Cargo-tool freshness task.
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.7] - 2026-10-12

- Derive internal shared helpers from their owning entrypoints, reducing explicit
  snapshot selections from 98 to 86 while retaining all existing files and checks
  ([Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.6] - 2026-10-12

- Adopt Shared Tooling 0.8.1's explicit snapshot entrypoints, automatic companion
  selection and automatic retirement of unchanged obsolete files, retaining
  Host's existing shared files
  ([Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)).
- Reject malformed snapshot fields consistently and retain both build-identity
  observations when Cargo tool installation detects a changed environment
  ([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133),
  [Shared #134](https://github.com/dragginzgame/shared-tooling/issues/134)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.5] - 2026-10-12

- Accidental version-only release; code and tooling are unchanged from 0.14.4.
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.4] - 2026-10-12

- Adopt Shared Tooling 0.6.6 and cargo-sort-derives 0.14.1, preserving literal
  `$crate` paths when sorting macro derives. Refresh contributor tools with
  `make install-tools`, then `make tools-check`
  ([Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.3] - 2026-10-12

- Adopt Shared Tooling 0.6.5 so staged formatting can check prepared Rust-tool
  receipts against staged pins and compiler selections without installing tools
  ([#69](https://github.com/dragginzgame/ic-host-tooling/issues/69),
  [Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.2] - 2026-10-11

- Adopt Shared Tooling 0.6.4's common Make entrypoint for setup, formatting,
  hooks, snapshot checks and releases, preserving Host's existing commands and
  release-tool extension ([Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130)).
- Consolidate developer setup and release guidance at their existing documentation
  owners, keeping a short README quick start ([#68](https://github.com/dragginzgame/ic-host-tooling/issues/68)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.1] - 2026-10-11

- Adopt Shared Tooling 0.6.3 to restart interrupted direct preparation from
  committed metadata, preserving prior Host validation receipts, notes and logs
  before revalidation ([#67](https://github.com/dragginzgame/ic-host-tooling/issues/67),
  [Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.14.0] - 2026-10-11

- **Breaking:** Return completed process results with a required exit status;
  select `require_success()` for zero-exit admission. Output observers now return
  `io::Result<()>`, preserving sink failures and cleanup evidence
  ([#66](https://github.com/dragginzgame/ic-host-tooling/issues/66)).
- Adopt Shared Tooling 0.6.2 for actionable interrupted-release source conflicts
  and clarified Audit/Machete invocation guidance
  ([Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126),
  [Shared #123](https://github.com/dragginzgame/shared-tooling/issues/123)).
  [Detailed notes](docs/changelog/0.14.md).

## [0.13.5] - 2026-10-11

- Adopt Shared Tooling 0.6.1: fix macOS Cargo-tool fixture counts and stop setup
  on lock, validation or activation failures. Run `make install-tools`, then
  `make tools-check` to select the updated build policy
  ([#65](https://github.com/dragginzgame/ic-host-tooling/issues/65),
  [Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120),
  [Shared #124](https://github.com/dragginzgame/shared-tooling/issues/124)).
  [Detailed notes](docs/changelog/0.13.md).

## [0.13.4] - 2026-10-11

- Add `durable::open_private_regular_lock_file` for owner-only sidecar creation
  in an existing parent, preserving existing files and caller-owned locking
  ([#64](https://github.com/dragginzgame/ic-host-tooling/issues/64)).
  [Detailed notes](docs/changelog/0.13.md).

## [0.13.3] - 2026-10-11

- Adopt Shared Tooling 0.6.0 to reuse verified Cargo tools across checkouts,
  retain local failure evidence when shared storage cannot be admitted, and
  identify installation-lock conflicts. Run `make install-tools`, then
  `make tools-check`; command paths and library APIs are unchanged
  ([#65](https://github.com/dragginzgame/ic-host-tooling/issues/65),
  [Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120),
  [Shared #121](https://github.com/dragginzgame/shared-tooling/issues/121)).
  [Detailed notes](docs/changelog/0.13.md).

## [0.13.2] - 2026-10-11

- Adopt Shared Tooling 0.5.0: host and IC tools share verified per-user bundles,
  while Cargo tools retain local slots. Run `make install-tools`, then
  `make tools-check`; use `.tools/rust/common/bin` for Rust tools on your PATH.
  Existing installations are retained;
  library APIs are unchanged
  ([#63](https://github.com/dragginzgame/ic-host-tooling/issues/63)).
  [Detailed notes](docs/changelog/0.13.md).

## [0.13.1] - 2026-10-11

- Keep Host's Make, release and publication test fixtures working with relative
  temporary directories, including retained failure evidence
  ([#62](https://github.com/dragginzgame/ic-host-tooling/issues/62)).
- Adopt Shared Tooling 0.3.12 so formatting-hook qualification accepts relative
  and symlinked temporary directories while preserving failure diagnostics
  ([Shared #116](https://github.com/dragginzgame/shared-tooling/issues/116),
  [#61](https://github.com/dragginzgame/ic-host-tooling/issues/61)).
  [Detailed notes](docs/changelog/0.13.md).

## [0.13.0] - 2026-10-11

- **Breaking:** Remove `durable::lock_file_with_progress`. Compose
  `open_regular_lock_file_with_parents` with `lock_exclusive_with_wait` and select
  polling/reporting intervals in the caller
  ([#60](https://github.com/dragginzgame/ic-host-tooling/issues/60)).

- Adopt Shared 0.3.11's temporary-path fixes so host-tool fixtures accept symlinked roots
  and release-command checks handle relative temporary directories
  ([Shared #111](https://github.com/dragginzgame/shared-tooling/issues/111),
  [Shared #112](https://github.com/dragginzgame/shared-tooling/issues/112)).
  Run the host-tool regression in native CI
  ([#59](https://github.com/dragginzgame/ic-host-tooling/issues/59)).
  [Detailed notes](docs/changelog/0.13.md).

## [0.12.10] - 2026-10-11

- Adopt Shared Tooling 0.3.9 so Rust tool preflight, installation and offline
  checks consistently use the selected consumer's toolchain and Cargo configuration
  ([#58](https://github.com/dragginzgame/ic-host-tooling/issues/58),
  [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101)).
  [Detailed notes](docs/changelog/0.12.md).

## [0.12.9] - 2026-10-10

- Fix macOS test assumptions about temporary paths and padded line counts, and
  include failed native fixtures in CI evidence uploads
  ([#57](https://github.com/dragginzgame/ic-host-tooling/issues/57)).
  [Detailed notes](docs/changelog/0.12.md).

## [0.12.8] - 2026-10-10

- Exercise the actual pre-commit hook and Host formatting in native CI, covering
  failed Git observations and preservation of staging and working edits
  ([#55](https://github.com/dragginzgame/ic-host-tooling/issues/55)).
  [Detailed notes](docs/changelog/0.12.md).

## [0.12.7] - 2026-10-10

- Adopt Shared Tooling 0.3.7 and make mandatory CI, installer and fixture checks
  reject failed assertions on Bash 3.2, preserving failed-test evidence
  ([#57](https://github.com/dragginzgame/ic-host-tooling/issues/57),
  [Shared #107](https://github.com/dragginzgame/shared-tooling/issues/107)).
  [Detailed notes](docs/changelog/0.12.md).

## [0.12.6] - 2026-10-10

- Keep only the newest CI run per workflow and branch or PR, cancelling older
  queued and running checks while retaining the existing host matrix and gates
  ([Shared #108](https://github.com/dragginzgame/shared-tooling/issues/108)).

See [the 0.12 notes](docs/changelog/0.12.md).

## [0.12.5] - 2026-10-10

- Reject NUL-containing publication paths before creating parent directories
  ([#54](https://github.com/dragginzgame/ic-host-tooling/issues/54)).
- Stop release and publication commands when source or version observations fail,
  even if their output appears valid
  ([#56](https://github.com/dragginzgame/ic-host-tooling/issues/56)).
- Adopt Shared Tooling 0.3.6: stop formatting on failed Git observations,
  retain incomplete-check evidence, and support lockfile-selected Cargo tools
  ([#55](https://github.com/dragginzgame/ic-host-tooling/issues/55),
  [Shared #106](https://github.com/dragginzgame/shared-tooling/issues/106),
  [Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103),
  [Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96)).

See [the 0.12 notes](docs/changelog/0.12.md).

## [0.12.4] - 2026-10-10

- Select Binaryen 133 for Linux and both macOS architectures, using the reviewed
  Shared Tooling 0.3.4 pins. Refresh local tools with `make install-ic-tools`;
  previous bundles are preserved
  ([#53](https://github.com/dragginzgame/ic-host-tooling/issues/53)).

See [the 0.12 notes](docs/changelog/0.12.md).

## [0.12.3] - 2026-10-10

- Adopt Shared Tooling 0.3.3, including shared Cargo jobserver forwarding and
  reliable Bash 3.2 fixture failure retention
  ([#52](https://github.com/dragginzgame/ic-host-tooling/issues/52),
  [Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99),
  [Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103)).
- Preserve jobserver access through Host's metadata and command-check wrappers.

See [the 0.12 notes](docs/changelog/0.12.md).

## [0.12.2] - 2026-10-10

- Adopt Shared Tooling 0.3.1: check platform and Rust/Cargo prerequisites before
  setup downloads, and identify the exact tool and repair command on failure
  ([#51](https://github.com/dragginzgame/ic-host-tooling/issues/51),
  [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101)).

See [the 0.12 notes](docs/changelog/0.12.md).

## [0.12.1] - 2026-10-10

- Provision and verify the pinned release executable through normal setup,
  removing the undeclared global `cargo-set-version` prerequisite
  ([#48](https://github.com/dragginzgame/ic-host-tooling/issues/48)).
- Preserve Make jobserver access for Host-owned Cargo, release and publication
  recipes ([#50](https://github.com/dragginzgame/ic-host-tooling/issues/50));
  shared formatting and installer recipes remain tracked separately under
  [Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99).
- Reject premature fixture exits on Bash 3.2 and preserve their failure evidence
  ([#49](https://github.com/dragginzgame/ic-host-tooling/issues/49)).

See [the 0.12 notes](docs/changelog/0.12.md).

## [0.12.0] - 2026-10-10

- **Breaking:** adopt Shared Tooling 0.3.0. Setup, CI and release preflight now
  require the complete host, IC and Cargo toolsets; run `make install-tools`
  followed by `make tools-check`. Retire the optional ripgrep/cloc installer flags
  ([#47](https://github.com/dragginzgame/ic-host-tooling/issues/47)).

See [the 0.12 setup migration notes](docs/changelog/0.12.md).

## [0.11.0] - 2026-10-10

- **Breaking:** consolidate process limits and cleanup evidence; update callers
  to `OutputLimits` with per-stream retention policies and optional deadlines,
  and read secondary failures from `ExecutionError::cleanup`.
- Forward long-running output with bounded retained diagnostics, expose cleanup
  failures in error messages, move generic examples to their owners, and retire
  unnumbered release drafts
  ([#46](https://github.com/dragginzgame/ic-host-tooling/issues/46)).
- Adopt Shared Tooling 0.2.13: reject line-break directory paths during snapshot
  refresh/verification and identify missing or invalid selected Cargo tools
  ([#45](https://github.com/dragginzgame/ic-host-tooling/issues/45)).

See [the 0.11 notes and migration guidance](docs/changelog/0.11.md).

## [0.10.2] - 2026-10-10

- Reject directory-suffixed publication targets instead of replacing the stripped
  filename ([#44](https://github.com/dragginzgame/ic-host-tooling/issues/44)).
- Adopt Shared Tooling 0.2.11 to reject hidden unsafe Make modes and report concise
  formatting results with retained failure logs
  ([#42](https://github.com/dragginzgame/ic-host-tooling/issues/42),
  [#41](https://github.com/dragginzgame/ic-host-tooling/issues/41)).

See [the 0.10 notes](docs/changelog/0.10.md).

## [0.10.1] - 2026-10-09

- Finish syncing newly created parent directories even when another writer wins
  the creation race, so durable writes do not depend on that writer completing
  its sync ([#43](https://github.com/dragginzgame/ic-host-tooling/issues/43)).

See [the 0.10 notes](docs/changelog/0.10.md).

## [0.10.0] - 2026-10-09

### Breaking

- Consolidate pathname publication on `write_with(path, options, producer)`;
  remove `write_typed_with`. Byte writers now retain publication state and
  cleanup errors, and lock-file creation exposes `RegularFileLockError::Publication`.
  Consumers must update writer calls and error handling before adopting 0.10.

See [the 0.10 notes](docs/changelog/0.10.md).

## [0.9.7] - 2026-10-09

- Accept recursive Make commands with additional arguments while preserving
  execution-mode checks, through Shared Tooling 0.2.8
  ([#42](https://github.com/dragginzgame/ic-host-tooling/issues/42),
  [shared #30](https://github.com/dragginzgame/shared-tooling/issues/30)).

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.9.6] - 2026-10-09

- Simplify response hex validation by checking empty text in the existing digit
  pass, preserving accepted formats, byte limits and typed errors.

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.9.5] - 2026-10-09

- Reject Make modes that can hide failed release or formatting commands, and
  keep Host commands and release-command checks bound to their own checkout's
  snapshot, adopting Shared Tooling 0.2.7
  ([#42](https://github.com/dragginzgame/ic-host-tooling/issues/42)).
- Support newline-ending checkout paths in formatting-hook qualification
  ([shared #90](https://github.com/dragginzgame/shared-tooling/issues/90)).

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.9.4] - 2026-10-09

- Share release and Rust formatting command definitions through Shared Tooling
  0.2.6, preserving Host's release policy and execution guards while removing
  duplicate recipes ([#41](https://github.com/dragginzgame/ic-host-tooling/issues/41)).

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.9.3] - 2026-10-09

- Preserve literal hook-path selections and support formatting hooks in checkouts
  whose directory names end in newlines through Shared Tooling 0.2.5
  ([#40](https://github.com/dragginzgame/ic-host-tooling/issues/40)).

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.9.2] - 2026-10-09

- Preserve CI qualification for each pushed commit while allowing superseded PR
  checks to be cancelled
  ([#39](https://github.com/dragginzgame/ic-host-tooling/issues/39)).
- Adopt Shared Tooling 0.2.2 installer fixes for pin files without a final newline
  and destinations changed during tool installation
  ([shared #87](https://github.com/dragginzgame/shared-tooling/issues/87),
  [shared #88](https://github.com/dragginzgame/shared-tooling/issues/88)).

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.9.1] - 2026-10-09

- Simplify durable file publication around one shared options representation,
  preserving existing permissions and publication behavior.
- Release captured output pipes at EOF and remove redundant completion flags.

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.9.0] - 2026-10-09

### Breaking

- Adopt Shared Tooling 0.2.0's five-tool IC bundle, removing PocketIC setup and
  pins in favour of Testkit ownership. Existing bundles require explicit
  `make install-ic-tools` before offline checks; retained installations are
  preserved ([#38](https://github.com/dragginzgame/ic-host-tooling/issues/38),
  [shared #76](https://github.com/dragginzgame/shared-tooling/issues/76)).
  Rust library APIs are unchanged.

See [the 0.9 notes](docs/changelog/0.9.md).

## [0.8.10] - 2026-10-09

- Reject multi-document dependency-pinning exceptions through Shared Tooling
  0.1.38, and preserve checkout-local tool lookup in isolated formatting hooks
  ([#37](https://github.com/dragginzgame/ic-host-tooling/issues/37)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.9] - 2026-10-09

- Prepare missing locked Cargo dependencies during release preflight before
  offline validation, while honouring explicitly offline Cargo settings
  ([#36](https://github.com/dragginzgame/ic-host-tooling/issues/36)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.8] - 2026-10-09

- Expose direct-child ownership for communication that preserves inherited or
  caller-selected process groups, enabling foreground consumers to reuse Host
  without changing group membership
  ([#34](https://github.com/dragginzgame/ic-host-tooling/issues/34)).
- Isolate the executable-admission fixture from parallel process spawns that
  can cause intermittent Linux `Text file busy` failures
  ([#35](https://github.com/dragginzgame/ic-host-tooling/issues/35)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.7] - 2026-10-09

- Add bounded streaming file hashing that rejects final symlinks and special
  files, allowing consumers to share file admission without buffering contents
  ([#33](https://github.com/dragginzgame/ic-host-tooling/issues/33)).
- Keep fleet tooling reports in Shared Tooling, removing Host's unused copy
  while retaining local LOC and setup/check commands
  ([#32](https://github.com/dragginzgame/ic-host-tooling/issues/32)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.6] - 2026-10-09

- Allow caller-owned child communication without an elapsed-time deadline for
  long builds and foreground services, retaining output bounds, cancellation
  and cleanup ([#31](https://github.com/dragginzgame/ic-host-tooling/issues/31)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.5] - 2026-10-09

- Reuse verified IC tools when pin-file comments or record order change, avoiding
  unnecessary downloads while retaining checksum and version admission
  ([shared #79](https://github.com/dragginzgame/shared-tooling/issues/79)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.4] - 2026-10-08

- Open existing lock files without temporary publication or sync work, allowing
  writable locks under read-only parent directories
  ([#29](https://github.com/dragginzgame/ic-host-tooling/issues/29)).
- Refresh shared installer validation and simulation-only release fixtures;
  report all refused release-source paths through the common checker
  ([#30](https://github.com/dragginzgame/ic-host-tooling/issues/30)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.3] - 2026-10-08

- Add live output observation to owned-child communication, retaining bounded
  capture and cleanup so consumers can remove private pipe-reader loops
  ([#28](https://github.com/dragginzgame/ic-host-tooling/issues/28)).
- Expose unlocked regular lock-file admission so consumers can share safe opening
  while choosing their own locking and wait policy
  ([#27](https://github.com/dragginzgame/ic-host-tooling/issues/27)).
- Keep lock-admission test fixtures in writable temporary storage, avoiding
  failures caused by `/dev` permissions.

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.2] - 2026-10-08

- Fix the cleanup-error test fixture on macOS, where `/bin/true` is absent,
  so native CI can exercise the intended cancellation and cleanup assertions
  ([#5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.1] - 2026-10-08

- Add immediate path-lock contention refusal through the existing regular-file
  admission API ([#24](https://github.com/dragginzgame/ic-host-tooling/issues/24)).
- Reduce allocation churn for fragmented reads, captured output, chunk digests
  and Candid normalization while retaining caller bounds
  ([#26](https://github.com/dragginzgame/ic-host-tooling/issues/26)).
- Add a compiled IC resource-report example with consistent text/JSON enforcement
  ([#25](https://github.com/dragginzgame/ic-host-tooling/issues/25)).
- Fix relative installer paths under inherited `CDPATH` and reject snapshots
  missing the installer fixtures' evidence helpers, by adopting committed
  Shared Tooling fixes
  ([shared #67](https://github.com/dragginzgame/shared-tooling/issues/67),
  [#73](https://github.com/dragginzgame/shared-tooling/issues/73)).

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.8.0] - 2026-10-08

- Allow caller-selected TERM grace and bounded reaping for owned groups,
  shared by explicit termination, communication failures and Drop. This supplies
  Query's required cleanup timing without another process runner
  ([#5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).
- **Breaking:** `CleanupError` and `ExecutionError` add `term_error`. Update struct
  literals and exhaustive destructuring. Existing spawn/capture defaults remain
  immediate KILL with synchronous reaping.

See [the 0.8 notes](docs/changelog/0.8.md).

## [0.7.2] - 2026-10-08

- Preserve checkout paths in release-adapter, publication and feature checks with an
  inherited `CDPATH` or trailing path newlines
  ([#23](https://github.com/dragginzgame/ic-host-tooling/issues/23),
  [shared #67](https://github.com/dragginzgame/shared-tooling/issues/67)).
- Refresh Shared Tooling's installer evidence fixtures and dotted snapshot-name
  recognition in tooling counts
  ([shared #66](https://github.com/dragginzgame/shared-tooling/issues/66),
  [#68](https://github.com/dragginzgame/shared-tooling/issues/68),
  [#69](https://github.com/dragginzgame/shared-tooling/issues/69)).

See [the 0.7 notes](docs/changelog/0.7.md).

## [0.7.1] - 2026-10-08

- Refresh matching local tracking refs after confirmed releases, avoiding stale
  ahead-of-origin status while preserving concurrent or unrelated refs
  ([shared #62](https://github.com/dragginzgame/shared-tooling/issues/62)).
- Adopt guarded consecutive snapshot refreshes and explicit companion-checked
  additions, plus tooling counts for `bin/` and repositories without commits
  ([shared #60](https://github.com/dragginzgame/shared-tooling/issues/60),
  [#61](https://github.com/dragginzgame/shared-tooling/issues/61),
  [#64](https://github.com/dragginzgame/shared-tooling/issues/64)).

See [the 0.7 notes](docs/changelog/0.7.md).

## [0.7.0] - 2026-10-08

- Communicate with caller-spawned children through the shared bounded IO engine,
  supporting piped input, inherited output, cancellation and explicit retention
  of successful background commands for admission before handoff
  ([#5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).
- **Breaking:** process failure categories now include cancellation and stdin IO;
  obsolete missing-output-pipe categories are removed. Update exhaustive error
  matches. Existing capture entrypoints keep their cleanup behavior.

See [the 0.7 notes](docs/changelog/0.7.md).

## [0.6.0] - 2026-10-08

- Capture wrapper commands with explicit process-group cleanup through the shared
  bounded execution engine, stopping same-group children on exit or failure
  ([#5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).
- **Breaking:** `ExecutionError` adds `group_error` so signalling failures remain
  separate from the original failure and direct-child cleanup errors. Update
  struct literals and exhaustive destructuring; ordinary capture keeps its
  existing direct-child behavior.

See [the 0.6 notes](docs/changelog/0.6.md).

## [0.5.2] - 2026-10-08

- Observe child exit while retaining cleanup ownership, then explicitly hand off
  a successful background command to caller-owned lifecycle management. Ordinary
  waiting and cancellation retain their existing group cleanup behavior
  ([#5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).

See [the 0.5 notes](docs/changelog/0.5.md).

## [0.5.1] - 2026-10-08

- Refuse release delivery when the final check changes the payload or tag, and
  verify local/remote release identity before reporting a completed resume
  ([#22](https://github.com/dragginzgame/ic-host-tooling/issues/22)).

See [the 0.5 notes](docs/changelog/0.5.md).

## [0.5.0] - 2026-10-08

- Own explicitly spawned process groups with polling, termination and cleanup
  during unwinding, while preserving caller-selected command IO and keeping
  original operation failures separate from cleanup diagnostics
  ([#5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).

- Add closed-writer staged executable admission before durable publication
  ([#20](https://github.com/dragginzgame/ic-host-tooling/issues/20)).
- **Breaking:** gzip encoding now accepts numeric levels 0–9; Wasm facts and
  typed inspection failures expand. Update compression arguments, fact literals
  and exhaustive error matches. Add opt-in revision-bound IC resource reports
  with exact code-body, defined-function and global counts
  ([#10](https://github.com/dragginzgame/ic-host-tooling/issues/10),
  [#14](https://github.com/dragginzgame/ic-host-tooling/issues/14)).

See [the 0.5 notes](docs/changelog/0.5.md).

## [0.4.6] - 2026-10-07

- Fix macOS publication-test expectations for non-UTF-8 filenames by checking
  native filesystem admission, while retaining exact-name success and cleanup
  coverage on supported filesystems
  ([#19](https://github.com/dragginzgame/ic-host-tooling/issues/19)).

See [the 0.4 notes](docs/changelog/0.4.md).

## [0.4.5] - 2026-10-07

- Fix durable-writer compilation on Intel and Apple Silicon macOS by converting
  admitted permissions to Darwin's narrower native type without truncation
  ([#18](https://github.com/dragginzgame/ic-host-tooling/issues/18)).

See [the 0.4 notes](docs/changelog/0.4.md).

## [0.4.4] - 2026-10-07

- Hash gzip or raw artifacts and compare decoded gzip with expected bytes
  without allocating a full decoded copy. Retain independent input/payload
  limits and strict single-member integrity checks
  ([#12](https://github.com/dragginzgame/ic-host-tooling/issues/12)).

See [the 0.4 notes](docs/changelog/0.4.md).

## [0.4.3] - 2026-10-07

- Stream durable output with typed producer errors, explicit replace/create-only
  policy and caller-selected permissions, including publication beneath an
  already-admitted directory descriptor
  ([#15](https://github.com/dragginzgame/ic-host-tooling/issues/15),
  [#16](https://github.com/dragginzgame/ic-host-tooling/issues/16)).
- Report staging cleanup failures after hard-link publication as already
  published, preserving the information callers need before retrying
  ([#15](https://github.com/dragginzgame/ic-host-tooling/issues/15)).
- Adopt Shared Tooling 0.1.19 to repair temporary-path handling in macOS tooling
  fixtures, preserve changelog history bytes and provide guarded optional Rust
  tool installation
  ([#6](https://github.com/dragginzgame/ic-host-tooling/issues/6)).

See [the 0.4 notes](docs/changelog/0.4.md).

## [0.4.2] - 2026-10-07

- Admit caller-trusted installed tools by exact version while retaining their
  observed digest for execution drift checks, removing caller-side hash/admit
  composition without changing pinned-byte admission
  ([#11](https://github.com/dragginzgame/ic-host-tooling/issues/11)).
- Hash ordered upload chunks and their complete input in one bounded pass,
  with explicit byte and digest-count limits
  ([#12](https://github.com/dragginzgame/ic-host-tooling/issues/12)).
- Preserve native input/output errors when converting bounded-copy failures
  at I/O publication boundaries
  ([#14](https://github.com/dragginzgame/ic-host-tooling/issues/14)).

See [the 0.4 notes](docs/changelog/0.4.md).

## [0.4.1] - 2026-10-07

- Add owned named staging for external-tool output, sharing durable publication
  while retaining producer errors, cleanup evidence and publication state
  ([#8](https://github.com/dragginzgame/ic-host-tooling/issues/8)).
- Share setup/check recipes and pinned host tools between Make and CI; add
  workspace and sibling-tooling LOC reports
  ([#6](https://github.com/dragginzgame/ic-host-tooling/issues/6)).
- Accept trailing whitespace on release headings while preserving notes and
  refusing duplicate or misplaced candidates
  ([#6](https://github.com/dragginzgame/ic-host-tooling/issues/6)).
- Preserve short internal dependency requirements such as `0.4` during release
  validation and preparation, matching Cargo's version updates.

See [the 0.4 notes](docs/changelog/0.4.md).

## [0.4.0] - 2026-10-07

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
