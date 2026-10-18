# Build ownership and extraction

IC Build owns the extracted reusable Cargo/Wasm and artifact-cache engine used
directly by Canic, IC Testkit and IcyDB. Canic remains responsible for Canic release
orchestration; Testkit remains responsible for testing and PocketIC lifecycle.
Neither should independently implement the common build and artifact-cache engine.

This design preserves the ownership audit discussed on 2026-10-12. Repository
bootstrap and saving the design were initially authorized. The maintainer has
since authorized the IC Testkit and Canic source extraction and scoped consumer
adoption recorded in the handoff. These past authorizations do not grant blanket
sibling mutation, installation changes or artifact cleanup.
The design is the coordination reference; GitHub issues own assignments,
acceptance and delivery status.

The current implementation and qualification evidence is recorded in the
[handoff](../status/current.md); [IC Build #1](https://github.com/dragginzgame/ic-build/issues/1)
owns extraction acceptance. The generic engine retains Testkit's `v1` stamp
format and `.ic-testkit` cache/lock namespaces. Release 0.16 changes Wasm recipe
keys to a typed v2 fingerprint domain. Release 0.17 selects a separate explicit-
environment recipe domain and clears child inheritance, causing cold builds while preserving
retained entries and conservative raw-input mutation checks. No migration reader or
automatic cleanup is introduced. Canic's feature-equivalent Cargo grouping moved
to IC Build with caller-supplied environment and output policy. Release 0.17
also requires the recipe's `CargoSelection` for isolated and combined grouping
probes, leaving feature, lock and network choices with the consumer. Complete-release
reuse/finalization remains Canic-owned. Shared execution does not admit Canic
roles, release identities or manifests.

## Implemented ownership

The current manifest depends directly on Host Artifacts, FS and Process, plus
Cargo's canonical metadata schema. Host Tools and Text are consumer dependencies
where interpretation or normalization is needed; Build does not depend on them.
Build's `cargo::execution` composes Host communication with Cargo decoding and
exact artifact selection. `cargo::native` keeps immediate protocol-refusal cleanup;
the Wasm path instead drains diagnostics after protocol refusal. These distinct
contracts share decoding, not one imposed failure policy. The 0.17.3 retained-evidence
and diagnostic adapters compose these existing owners for Wire evaluation; they
do not own measurement scenarios, report schemas or acceptance/publication.

Host FS 0.27.2 owns shared descriptor-lock release, metadata-only logical directory
sizes and standard cache-directory tag recognition/publication. Build consumes
those APIs directly, keeping its record-clone lifetime, cache thresholds,
maintenance ordering, tag suffix and typed cache-error context. Generic filesystem
tests belong to Host; Build retains record-retention and cache-policy integration
checks. The historical `.ic-testkit` names are live retained identities, not an
alternate implementation or an invitation to remove existing artifacts.

IC Platform is the Candid/CDK library dependency boundary. Published 0.1.3 is the
reviewed adoption baseline; Build currently declares no such dependency or uses
any platform API. Its native extraction adapter executes an admitted tool and
retains opaque bounded bytes through Host, so replacing it with a Candid decoder
would change its contract. Keep its non-UTF-8 output, source-change refusal and
typed execution/cleanup evidence. The shared direct-dependency guard prevents
listed Candid/CDK packages from being introduced directly, including by aliases.
Resolved transitive graphs and downstream deployed artifacts need separate checks.

Build 0.12's Unix `artifacts::candid` moves native extraction
orchestration out of Host Tools, over Host hashing and admitted execution.
Host 0.24 removes the former extraction owner without an alias. Tools retains
Candid/IC interpretation; Text retains normalization. Consumers retain source
leases, tools, budgets, environment, text/grammar policy and publication.
Producer and consumer acceptance remain separate under
[#51](https://github.com/dragginzgame/ic-build/issues/51).

| Consumer | Shared owner and retained consumer responsibility |
| --- | --- |
| Canic | IC Build supplies grouping, declaration-cache transactions, captured outputs, staging, standalone input evidence and strict depfile discovery. Canic retains recipes, source custody, transforms and complete-release admission. |
| IcyDB | IC Build supplies Cargo/Wasm and post-link artifact caching. IcyDB retains optimizer choices, feature policy and interface validation. |
| IC Testkit | IC Build supplies native fixture/probe builds. Testkit retains PocketIC infrastructure, testing contracts and benchmark policy. |

IC Build 0.2.4 supplies explicit capture retention for Jobs recovery evidence and
structural named-example input/output admission with the existing Cargo command
factory for Query. These producer capabilities do not establish consumer adoption.
Jobs owns evidence publication and lifecycle; Query owns its executor, receipt
framing, metadata and smoke/deployment lifecycle. The producer/adapter evidence
and remaining publication/native boundaries are recorded in the handoff above.

The larger Canic input-engine convergence is tracked in
[#530](https://github.com/dragginzgame/canic/issues/530); its local replacement and
remaining delivery/native acceptance are recorded in the handoff. Generic compilation or a cache hit does not
establish complete-release validity.

## Historical build paths at the 2026-10-12 audit

The following inventory and extraction proposals preserve the inspected source
at that audit. They describe the original duplication, rather than the current
dependency route; the implementation handoff above owns subsequent evidence.

| Entry point | Execution path |
| --- | --- |
| Canic production builds, including Toko Miner | Canic CLI → Canic Host → Cargo → Candid extraction → Wasm transforms → Canic release publication |
| Canic integration tests | Testkit build/cache APIs for fixtures; some tests wrap the complete Canic build in Testkit's generic artifact cache |
| IcyDB application and fixture builds | IcyDB adapter → Testkit Cargo/cache engine → IcyDB optimizer and artifact validation |
| Backup and Blob optimizer qualification | Repository scripts invoke prepared wasm-opt and run product-specific tests |
| Smaller sibling compilation checks and probes | Direct Cargo commands with repository-owned package/profile selections |
| Shared Tooling | Tool installation, executable storage, CI helpers and optimizer qualification |

The strongest existing reusable starting point is
[Testkit's artifact engine](https://github.com/dragginzgame/ic-testkit/blob/235579e3342d/crates/ic-testkit/src/artifacts/mod.rs).
It already handles resolved inputs, cached builds, transactions, retained artifact
ownership and batch results. IcyDB and Canic's tests use it. Canic production builds
have a separate implementation with additional release-specific guarantees.

## Intended owners

| Responsibility | Sole implementation or policy owner |
| --- | --- |
| Compilation and Cargo's own incremental freshness | Cargo/rustc |
| Cargo input resolution, build execution, fingerprints, artifact caches, retention and batch results | IC Build |
| Transform execution and cached intermediate artifacts | IC Build |
| Hashing, bounded streams, gzip and raw Wasm structural facts | ic-host-artifacts |
| Filesystem admission, durable writes and lock primitives | ic-host-fs |
| Process execution, output capture, cancellation and cleanup | ic-host-process |
| Native Candid extraction orchestration | IC Build; consumers retain tool and source admission |
| Common IC artifact interpretation | ic-host-tools |
| Common executable installation, storage and portable repository tooling | Shared Tooling |
| Role selection, declaration/runtime build choices, generated infrastructure, release IDs and release manifests | Canic |
| PocketIC, test lifecycle and fixture-specific requirements | IC Testkit |
| IcyDB profiles, optimizer recipe and expected generated services | IcyDB |
| Toko application configuration | Toko Miner, delegating deployable builds to Canic |
| Product qualification and published report schemas | Each product, consuming shared facts and build results |

The dependency direction is application/test adapters → IC Build → Host Artifacts,
FS and Process. IC Build must not depend on Canic, IcyDB, PocketIC or IC Testkit. Avoid
introducing a generic scheduler, plugin system or installation store to perform
this extraction. Existing proven mechanisms are the starting point.

Build provenance follows the same direction: the engine produces a receipt;
Canic projects it into its release schema. Consumers should not reconstruct
compiler arguments, tool identities or transform outcomes after execution.
Independent verification at a release or trust boundary remains necessary.

## Historical structural findings

### Generic build mechanics are coupled to the testing package

Severity: MEDIUM. Testkit's artifact APIs are useful outside testing. IcyDB already
uses them for its application build route, while native ic-testkit dependencies
include PocketIC.

Extract the generic artifact/build implementation into `crates/ic-build/` in this
repository, moving its behavioral tests with it. Testkit, IcyDB and Canic should
consume the new owner directly. Do not write another engine beside the original
or turn ic-host-fs or ic-host-process into a build system.

The extraction candidates include Testkit's `artifacts/wasm_cache.rs`,
`wasm_batch.rs`, `transaction.rs`, `transaction_batch.rs`, `cache_fs.rs`,
`digest.rs`, `workspace.rs` and generic artifact discovery helpers. Review the
actual callers of each helper before moving it; PocketIC-specific behavior stays
in Testkit. These paths describe candidates, not an approved blind directory move.

### Build identity and reuse have independent implementations

Severity: MEDIUM. Compare
[Canic complete-build reuse](https://github.com/dragginzgame/canic/blob/25b73cd403b2/crates/canic-host/src/canister_build/reuse/mod.rs)
and [Testkit Wasm caching](https://github.com/dragginzgame/ic-testkit/blob/235579e3342d/crates/ic-testkit/src/artifacts/wasm_cache.rs).
Both inspect sources, Cargo configuration, tools, environment and outputs, but
what their evidence establishes is different.

| Concern | Canic | Testkit |
| --- | --- | --- |
| Cached result | Complete Canic release with role/manifest bindings | Compiled Wasm or an arbitrary artifact set |
| Additional inputs | Includes observed Cargo dependency files and generated exports | Supports explicitly declared additional inputs |
| Environment | Broad inherited-environment identity with explicit exclusions | Selected common variables plus caller declarations |
| Artifact lifetime | Canic build/release coordination | Returned records retain cache entries against pruning |
| Batching | Groups packages only when dependency-feature trees remain equivalent | Sequential batches with retained successes and collected failures |

IC Build should own Cargo input resolution, fingerprints, artifact caches and
artifact lifetime. Canic supplies its generated and application inputs and keeps
the decision that a release's roles, identities and manifests are valid.

Do not replace Canic's release reuse with a generic cache-hit test. Preserve its
observed dependency inputs and before/after-source guarantees, including the rule
that newly discovered inputs cannot acquire retrospective cache authority. A
successful compile, artifact-cache hit and valid complete release are distinct
facts, even when they share mechanical infrastructure.

Canic's declaration extraction cache is another consumer of generic cache
mechanics: its recipe is exact declaration Wasm, extractor identity, environment
and normalized output. Its optional-cache fallback and bounded-worker semantics
must survive any replacement.

### Transform execution is repeated but recipes differ

Severity: MEDIUM.
[Canic finalization](https://github.com/dragginzgame/canic/blob/25b73cd403b2/crates/canic-host/src/artifact_io/mod.rs)
shrinks Wasm, optionally embeds Candid, optimizes release builds, checks installation
limits and produces gzip. It compares pre/post export inventories, public Candid
metadata and required Wasm features around optimization.

[IcyDB's optimizer](https://github.com/dragginzgame/icydb/blob/13b0958aeb70/testing/integration/src/wasm_optimizer.rs)
uses its own explicit flag sequence, including an inlining restriction. Backup
and Blob have separate qualification recipes; those are not automatically product
build policies.

IC Build should own execution of an explicit transform recipe, result capture,
staging, transform caching and validation of declared invariants. Consumers own
flags, ordering, metadata choices, limits and acceptance requirements. Do not
flatten these into a universal optimization profile or silently weaken a
consumer's validation to match another implementation.

A common deployable-Wasm adapter may select a qualified shared LTO policy through
IC Build's existing explicit environment. This centralizes the recipe without
making the generic engine reinterpret consumer profiles. The
[LTO guidance](../artifacts.md#deployable-wasm-lto) records initial whole-canister
evidence and the complete-application comparison needed for adoption.

Retain the distinction between publishing a cache entry and publishing a product
release. Canic currently qualifies a staged Wasm/Candid/gzip set and replaces its
files under its build coordination boundary. A cached directory transaction alone
does not establish Canic release visibility or manifest validity.

### Candid and Wasm interpretation are independently implemented

Severity: MEDIUM. Canic uses a parser-backed Candid service model. IcyDB contains a
separate text scanner. Canic's artifact check compares method names; IcyDB carries
query/update/composite-query modes. Reserved CDK exports are filtered differently.

Relevant source:
[Canic endpoint parsing](https://github.com/dragginzgame/canic/blob/25b73cd403b2/crates/canic-host/src/candid_endpoints/mod.rs)
and [IcyDB artifact inspection](https://github.com/dragginzgame/icydb/blob/13b0958aeb70/testing/integration/src/canister_artifact.rs).

Propose one generic Candid-service and IC-export interpretation layer in
ic-host-tools, using the existing Candid parser. This explicitly expands its
current contract, which leaves grammar interpretation to consumers. Expected
methods, permitted internal exports, payload annotations and application-specific
validation remain consumer inputs. Raw Wasm parsing remains in ic-host-artifacts.

Agree the intended name/mode and reserved-export semantics before replacing
callers. Parsing and projection can be shared without requiring Canic and IcyDB
to accept the same application contract.

### Common build tools have competing installation implementations

Severity: MEDIUM. Canic owns Binaryen/ic-wasm authority tables and installation
mechanics. Shared Tooling owns common IC installation and verified storage.
Canic's inspected Binaryen installer selects `~/.local/bin/wasm-opt`; shared setup
uses verified bundles and checkout selections.

Shared Tooling should own common installation/storage mechanics and the common
catalog. Canic keeps its explicit `canic toolchain install` entry point and its
required tool selection. IC Build receives admitted tool selections and never
installs tools implicitly.

A published Canic CLI needs a supported, versioned installation interface. It
cannot require a mutable sibling checkout. Resolve that distribution requirement
before retiring Canic's installer, preserving platform runtime-library layout,
archive and executable verification, retries and existing retained installations.
Consumer-selected qualified versions need not become identical by accident.

## Original extraction sequence

1. Consolidate Candid/export interpretation and qualify the behavior differences.
   This can proceed independently of build-engine extraction.
2. Extract Testkit's generic build/cache engine into IC Build. Move the behavioral
   tests and preserve typed results, failures, artifact retention and maintenance
   semantics. Avoid unrelated redesign during the move.
3. Move IcyDB and Canic's tests to the extracted owner without changing their
   recipes. Testkit retains only testing adapters and PocketIC behavior.
4. Converge Canic production builds after preserving dependency discovery,
   feature-safe batching, cancellation, declaration/runtime separation and
   release-input guarantees. Delete superseded generic implementations only when
   those consumers use the new owner.
5. Unify installation behind a distributable Shared interface, then retire
   Canic's duplicate installation mechanics. This can be coordinated separately
   from the build-library extraction.

Every slice needs an owning GitHub issue with producer and consumer acceptance.
This sequence records dependencies between changes; it is not a second issue queue.
Existing sibling work must be coordinated before modifying their active files or
build directories.

## Required behavior and retained boundaries

- Builds have no finite default elapsed-time deadline. Preserve builds lasting
  hours, progress reporting and explicit caller cancellation.
- Preserve original process failures, diagnostic bounds, observer failures and
  cleanup evidence. Canic's current command-output capture and Testkit's observed
  process execution are not automatically interchangeable.
- A prepared source snapshot is valid only under its documented source-custody
  contract. A build lock does not automatically exclude arbitrary source writers.
- Reject source/environment/tool changes before publishing reusable evidence.
  Preserve external inputs, generated inputs and Cargo configuration selection.
- Batching must not silently unify dependency features differently from isolated
  builds. Preserve returned successes and collected failures where callers rely
  on that behavior.
- Retain exact artifact ownership through downstream consumption. Pruning must
  not remove artifacts retained by a live record. Cleanup requires explicit scope;
  extraction does not authorize deleting consumer targets or retained evidence.
- Keep application profile choices, byte limits, source leases, tool selections,
  credentials, release identity and lifecycle with the consumer.
- Preserve native Linux x86-64 and macOS 15 Intel/Apple Silicon qualification.
  Substitute fixtures and Linux tests alone do not prove native macOS behavior.
- Keep simple direct `cargo check --target ...` recipes. They already delegate
  compilation to Cargo and do not justify another abstraction.
- Product qualification reports keep their own subjects, expected behavior and
  schemas. Reuse build receipts and raw metrics rather than copying build logic.

## API and retained state transition

The intended pre-1.0 cut has one current API and one production owner. Do not
leave deprecated aliases, compatibility reexports, dual cache engines or copied
implementations indefinitely active. Removing Testkit's exported artifact types
is a consumer API change and requires the affected package's appropriate release
boundary; consumers must be updated and qualified explicitly.

Freeze the retained cache formats and lock namespaces before moving their owner.
Determine whether exact semantics permit retaining the format, or select a new
namespace with an explicit disposition for old artifacts. Do not reuse a format
identity for incompatible contents, add an implicit dual reader, or erase old
artifacts merely to simplify the transition. Release evidence and application
release identities remain with their existing owners.

## Audit evidence and limits

The audit used the flow-convergence method from Shared Tooling revision
`019ce8236cc7cee14ea5540d468b939a2053ef30`, under the IC Host Tooling overlay.
This repository boots from the corrected Shared 0.9.0 revision
`aec7a149f581a42b9531fc747b46024fc0314fcf`; that follow-up changed VERSION only.

Primary inspected source identities were Canic `25b73cd403b2`, IC Testkit
`235579e3342d`, IcyDB `13b0958aeb70` and IC Host Tooling `bea0f3e76491`.
Canic and IcyDB had active working-tree changes. In the selected Canic build
families, dirty changes included `canister_build/candid.rs` and
`canister_build/reuse/lock/mod.rs`; the inspected IcyDB build/cache/optimizer files
had no local diff. Source links identify committed anchors, not every dirty byte.
Recheck active source and authorization before implementation.

The caller inventory also covered Toko Miner and the smaller Backup, Blob,
Auth, Query, Memory, Metrics and Timers build/qualification entrypoints. This was
source inspection, not a line-by-line audit of every test, compiled integration
proof, performance measurement or deployment qualification. The finding severity
reflects ownership and maintenance burden, not a demonstrated cache-corruption
or runtime-safety failure. No code was moved and no builds or issue mutations were
performed for that audit.
