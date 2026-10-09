# Host qualification

Required hosts are Linux x86-64 and macOS 15 on Intel and Apple Silicon.
The imported filesystem/process APIs retain their platform gates; unsupported
platforms receive their existing explicit refusal rather than simulated behavior.

Select one changed crate with PACKAGE=<crate> for make check, make clippy,
make docs-check or make test. Artifact checks use its actual enabled gzip,
archive and Wasm features; production filesystem dependencies select none of them.
Check target-directory ownership before compilation. Preserve failure logs and artifacts.

Native CI prepares pinned Rust/cargo-sort and local host tools, then verifies
the shared snapshot, declarations, formatter prerequisites, release adapter,
common tool/LOC command wiring,
minimal artifact tests and each selected package with all features in sequence.
The release gate uses the same host tools; installing the IC executable bundle
is separate explicit setup and is not required for library qualification.
CI also owns native macOS execution; Linux qualification is not macOS evidence.
`ci/ic-tools.tsv` is a consumer-owned pin selection, excluded from automatic
snapshot refreshes. Shared 0.2.0 adoption removes its PocketIC rows; versions and
digests for Quill, ICP CLI, didc, ic-wasm and wasm-opt are retained. Host library
gates have no PocketIC callers, so no simulator dependency or setup is added.
Consumers that need a simulator use Testkit's qualified setup/check/run contract.
The shared installer reads this one local five-tool matrix. Existing six-tool
bundles fail the new offline check; explicit `make install-ic-tools` prepares
the new selection while preserving previous bundles, receipts and failed evidence.
Tool reuse compares validated records, so comments and record order do not
require downloads or replace installation provenance. Changed tool selections
and malformed records still fail admission before executable checks.
The workflow runs on pull requests and pushes to `main`; it cannot qualify
uncommitted local bytes. A scoped contribution PR exercises both macOS 15
architectures and Linux against the candidate source before release. Bind
results to that PR's tested commit and keep consumer rehearsals separate.
Earlier Shared Tooling host-fixture failures on macOS remain historical evidence
in the original repository. The refreshed fixture has local Linux execution with
substituted host identities here; native macOS qualification remains separate.
