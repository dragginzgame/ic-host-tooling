> Historical handoff imported from Build 0.24.0. The live handoff is
> [IC Tooling current status](../../status/current.md).

# Current handoff

## Published source and boundary

Build 0.23.0 is published, non-yanked, from
`2fd5539b835065cba943df43a5ef5e5a366df064`. Its archive SHA-256 is
`225d849b7d759341caffe3e4e1b5f02bac50773d28b687a7ca7bcfa7f6d770f6`.
That published graph used Host 0.31.2. The pending root graph now selects published
Host Artifacts/FS/Process 0.32.0, retaining the incoming manifest/lockfile edits.
The notes move to 0.24.0 because consumers exchanging public Host values must
align their direct Host dependencies to 0.32. Build's own package version remains
0.23.0; index and refs are unchanged. Command/byte observations, engine
implementations, retained formats and feature defaults are unchanged.
Shared Tooling is reviewed 0.25.8, `0a2d2ee3b5e0ae77755c2915ac28836d4d5ba359`.
The canonical refresh preserves the selected roots and updates six payloads.
The release gate now runs its existing Clippy scope before expensive stages;
package and tagged publication entrypoints also refuse lint failures first.

The exact-source [0.23 producer workflow](https://github.com/dragginzgame/ic-build/actions/runs/39414505849)
passes Linux, native macOS Intel/Apple Silicon, optimizer controls and lint/security.
These producer receipts do not establish consumer installation or lifecycle acceptance.

Build owns generic Cargo execution, feature-safe grouping, input evidence and
retained artifact/cache mechanics. Host owns process/filesystem primitives;
Cargo owns workspace membership and its metadata schema. Consumers own recipes,
source custody, budgets, tools, credentials, release identities and lifecycle.
Metadata/tree/workspace observation uses standard commands and bytes; grouping
accepts the consumer's tree observer. Admit success then completeness before
decoding. The execution/artifact engine still exchanges Host values and requires
matching direct Host identities. Preserve retained formats, lock namespaces,
fingerprint domains and leases, including live `.ic-testkit` paths.
See [ownership](../design/build-ownership.md) and [usage](../artifacts.md).

## Registry consumer delivery

[Memory PR #126](https://github.com/dragginzgame/ic-memory/pull/126), head `764b127`,
selects published Build 0.23 and moves native repo-tool capture onto Memory's Host.
Its locked/offline catalog, 4 MiB/64 KiB limits, no deadline and typed diagnostic
reporting are preserved. Only Build's version/checksum changes in its lock;
independent runtime qualification is unchanged. All 55 repo-tool tests, strict
helper Clippy, actual Rust 1.91 checks, pinned formatting and inherited dependency
checks pass using registry dependencies. At the earlier `0330277`, all 55 repo-tool
cases pass on macOS and all three runtime rows pass. Its subsequent adapter fixture
exposed Bash 3.2's empty-array/nounset refusal during editor preparation. The pushed
repair keeps the full selection array nonempty; the existing adapter fixture,
ShellCheck and syntax checks pass locally. All three new-head MSRV and runtime
rows pass; remaining hosted tests and native Bash qualification stay separate.

[Testkit PR #136](https://github.com/dragginzgame/ic-testkit/pull/136), head `cbb68d4`,
completes current main `31e09dd`'s already-selected Build 0.23 adoption in the
private benchmark driver and installed-test caller. Selected Cargo/Rustc, cwd,
64 KiB capture limits and typed I/O/Host mapping remain local. The committed
package-changelog conflict markers are resolved without rewriting dated history.
The selected graph and own version are unchanged. All 13 focused driver cases,
strict private-caller Clippy and actual Rust 1.91 caller checks pass. Formatting,
direct/inherited policy and native/Wasm graph exclusions pass; Build/Host remain
excluded from the declared deployed probe graph. All matching hosted checks,
MSRV, portable Host, browser-session and PocketIC-concurrency rows pass on Linux
and native macOS Intel/Apple Silicon. These receipts do not close the broader
installed benchmark/application acceptance.

Build PRs [#79](https://github.com/dragginzgame/ic-build/pull/79) and
[#80](https://github.com/dragginzgame/ic-build/pull/80) are closed as superseded by
the published source; no unique implementation was discarded.
[Build #9](https://github.com/dragginzgame/ic-build/issues/9) remains the consumer
acceptance tracker. Review and integration remain with each consumer owner.

## Canic follow-through

[Canic PR #596](https://github.com/dragginzgame/canic/pull/596), head `c378d48e1`,
retains its published Build 0.19/Host 0.27.6 graph and observed runtime-facts work.
The pushed CI repair binds the pinned compiler-cache installer's exact executable into each cache-enabled
job and aligns the repository development pin with the existing Rust 1.99.1
workflow/tools. Shared wrappers and strict admission stay unchanged. Optimizer
fixtures now check canonical physical identity; an explicit directory-alias
regression qualifies this on Linux. Eight Binaryen cases, compiler-cache controls,
actual workflow binding blocks with substituted installer/compiler, Actionlint,
strict Host library/test Clippy and grouped formatting pass. The subsequent
Core/Contracts/CLI lint failures are repaired without suppressions. All-target,
all-feature Core/Contracts/CLI and Host test Clippy pass. Eighteen session,
28 memory, 11 sharding, one pending-intent and one Candid round-trip case pass;
Core/Contracts also pass actual Rust 1.91 native and Wasm library checks. Session
admission reads its subject count with the other indexes, preserving all refusal
and pre-write checks. Application Wasm build admission is not bypassed. New-head
hosted qualification remains separate.

[Draft Canic PR #612](https://github.com/dragginzgame/canic/pull/612), head `6fa94614d`,
is stacked on #596 and selects published Build 0.23, Host 0.31.2 and Backup 0.35.1.
Backup supplies the matching checksum/publication error identities;
its artifact checksum implementation is unchanged from the previous selection.
Host Tools explicitly enables the Candid parser already required by Canic.
Host/CLI/Internal all-target checks and actual Rust 1.91 library checks pass after
porting the actual metadata, tree/grouping, workspace and fixture-inventory callers.
Strict affected Clippy, nine build/grouping, 13 provenance, nine backup artifact,
22 persistence and four embedded-inventory cases pass; one explicit real-Wasm
reproduction stays ignored. Direct/inherited policy, all three workspace boundaries
and Build/Backup/Host exclusions from declared Wasm roots pass. The root application Platform requirement
remains unchanged; newer Platform and older Host selections required by upstream
packages are separate graph identities, not evidence of runtime migration.
The parent repairs are normally merged into the adoption branch without rewriting
shared history. Both graphs now pass the passive-contract closure and internal
package-admission regressions: the check uses the package-selected Cargo tree,
and the managed test role selects its configured delegated-token verifier. The
test canister passes actual Rust 1.91 native checking. Merged strict all-target,
all-feature Core/Contracts/CLI/Host Clippy and Host/CLI/Internal MSRV checks pass.
The role group improves from 34/41 to 36/41. Five fresh-fixture Memory 0.48/0.49
conflicts remain; keep the guards and coordinate the existing consumer-owned
Memory/Storage/Blob work. The draft is delivered, not integration or installed acceptance.
Normal sibling checkouts and their pending work remain untouched.

## Evidence and limits

Host 0.32.0 adoption evidence and incoming before-images:
`/tmp/ic-build-host-adoption.fcpwYQ/`. The official registry index confirms all
three packages are the latest non-yanked stable releases; their cached archive
checksums match the index and source `c3f6f605a47f248e18bc3ad337e431a990a7a619`.
Their source implementations match 0.31.2 byte-for-byte, so no engine adapters
or copied replacements are needed. The graph contains one version per package,
no Platform/Candid/CDK, and no optional capability/tempfile/Wasm/text backends.
Strict all-target/all-feature Clippy, actual Rust 1.91 all-target checks,
seven execution, three Host/cache and four artifact-handoff tests pass; the
subprocess-only handoff worker remains marked ignored in the parent run.
Snapshot, pins/inheritance, boundaries, formatting and links pass. Both macOS
target graphs resolve with unchanged exclusions; this is dependency inspection,
not native execution. Full release, native macOS, optimizer and installed-IC
qualification were not rerun. Incoming manifest/lockfile edits are preserved
exactly, with no further dependency or package-version changes.

Shared Tooling 0.25.8 adoption evidence and incoming before-images:
`/tmp/ic-build-shared-0258.rea087/`. Snapshot integrity, dependency pins/inheritance,
direct boundaries, formatting, local links, release routing, shell/workflow lint,
strict Clippy and the isolated release-adapter checks pass on Linux. Fixtures
verify lint failure stops CI, packaging and publication under parallel Make,
and finalize a numbered `Unreleased` heading through the real metadata adapter.
No full release gate, live release/publication or new native macOS qualification
was run for this adoption.

Current patch evidence/before-images: `/tmp/ic-build-patch1-fixes.LDF1Oa/`.
Earlier delivery/qualification evidence: `/tmp/ic-build-adoption-delivery.O20V1y/`.
Earlier API-slice before-images: `/tmp/ic-build-hardcut-more.RkD1D3/` and
`/tmp/ic-build-observation-cut.v31SQM/`. Historical extraction evidence is preserved
in the [0.22.1 handoff](https://github.com/dragginzgame/ic-build/blob/1d21935cd6779229a07db435691a1e86598691f2/docs/status/current.md),
the design and numbered changelogs. No normal sibling checkout mutation, root Git
operation, integration-branch merge, package version change, release, publication
or artifact cleanup is included.
