# Current handoff

## Source and ownership

Starting commit: `1d21935cd6779229a07db435691a1e86598691f2` (Build 0.22.1).
Own package versions remain 0.22.1; the complete pending API batch selects 0.23.0
notes. Preserve the incoming Cargo.lock selection of Host 0.31.2. No staging,
root Git refs, version changes, release, publication or artifact cleanup is included.
Shared Tooling stays at reviewed 0.25.4, `db555ab6cf4a93af9a183f18d921d3b0ee97c725`.

Build owns generic Cargo execution, feature-safe grouping, input evidence and
retained artifact/cache mechanics. Host owns process/filesystem primitives;
Cargo owns workspace membership and its metadata schema. Consumers own recipes,
source custody, budgets, tools, credentials, release identities and lifecycle.
Keep current cache formats, fingerprint domains, lock namespaces and retained
leases unchanged. The historical `.ic-testkit` names are live retained identities.
See [ownership](../design/build-ownership.md) and [usage](../artifacts.md).

## Pending 0.23 boundary

Metadata/tree specifications render standard commands, and metadata decoding is
public. Workspace discovery now renders offline `locate-project` separately from
workspace-response decoding. It still canonicalizes the crate directory, disables
toolchain installation and refuses malformed/nonabsolute manifest responses.
Consumers admit success, then completeness, before decoding with their runner.

Feature-safe grouping accepts a tree observer instead of Host limits and a
command factory. Build retains label validation, isolated/combined recipes and
feature-equivalence checks. Original boxed observer failures pass through unchanged.
The Cargo observation module has no production Host Process dependency or combined
executor. No compatibility aliases, runner mirrors or new shared policy types
are introduced. Execution and retained-artifact APIs still exchange Host values;
matching those types still requires the corresponding direct Host generation.

The first slice's metadata/tree consumer ports remain prepared in
`/tmp/ic-build-observation-cut.v31SQM/`. Memory's 55 repo-tool cases, strict Clippy
and actual Rust 1.91 helper checks pass with a qualification-only Build path.
An external Host 0.30.1 consumer probe passes against the prior Host 0.31.1 graph.
These are local qualification receipts, not published consumer adoptions.

## Consumer preparation and delivery

Canic and Testkit workspace discovery ports preserve their selected commands,
output budgets and typed I/O error sources. Canic's grouping adapter now executes
through its own Host, preserving build-context environment and unbounded existing
capture allowances. A narrow Rust 1.91 harness imports the actual Canic adapters
with Host 0.27.6 against local Build/Host 0.31.1; context policy is substituted
only for the grouping harness. It proves API/execution wiring, not full Canic
compilation or application admission. Testkit's actual private library and
integration-test callers compile against the local Build source.

The adoption ports stay isolated until Build 0.23 is published and real consumer
locks can be resolved. Never deliver qualification paths or fabricated registry
checksums. Other Host-exchanging Canic engine calls still need coherent alignment.
Normal sibling checkouts and their pending work remain untouched.

Canic [PR #596](https://github.com/dragginzgame/canic/pull/596), head `168f3f407`,
carries the independent published-graph follow-through. Its predecessor `5bcdc7677`
passes hosted preflight and security; subsequent checks exposed retired Memory
re-export imports and a stale hook installer call. The pushed repairs use existing
published governed IC Storage adapters and the canonical explicit-consumer
installer. No storage-format, lease or lifecycle policy change is intended.
Full exact-head hosted qualification remains separate.

## Current validation and limits

Focused Linux evidence: 67 Cargo unit cases and three real Cargo integration
cases pass, covering launcher/environment preservation, typed failure ordering,
complete-output admission, feature-safe grouping and workspace membership,
exclusions and independent roots. Strict all-target Clippy, warning-denied Rustdoc
and actual Rust 1.91 all-target checking pass on the incoming Host 0.31.2 graph.
Testkit's 13 focused driver cases pass, including workspace runner-error mapping;
its strict private-caller Clippy and actual Rust 1.91 caller checks pass.
The native executable fixture deliberately compiles one invalid producer and
reports its diagnostic while its negative test passes.

Canic's real hook fixture, ShellCheck, grouped formatting, inherited dependency
checks, strict caller Clippy and focused native Rust 1.91 caller checks pass.
Direct Cargo Wasm checking is refused by its existing authoritative-build
guard; do not bypass it or describe this as Wasm/installed qualification.
Full release gates, native macOS, native tool reuse and installed/managed IC
acceptance remain unproven for this pending batch. Shared's earlier tool-reuse
run `39387644688` failed on all hosts; local checks do not replace that evidence.

Current evidence and before-images: `/tmp/ic-build-hardcut-more.RkD1D3/`.
Earlier qualification/extraction narratives are preserved at the immutable
[0.22.1 handoff](https://github.com/dragginzgame/ic-build/blob/1d21935cd6779229a07db435691a1e86598691f2/docs/status/current.md)
and the numbered changelog/design. The startup handoff is now a current summary.
