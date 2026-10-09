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
`ci/ic-tools.tsv` is a consumer-owned pin selection, retained byte-for-byte from
the previous snapshot at `db039347d2372b877c1c46dcdd2b5c3aa9412009`.
It is excluded from subsequent shared snapshot refreshes so tooling maintenance
does not silently change the selected PocketIC 16.0.0 server. Host library gates
do not use that server; changing these pins and qualifying protocol consumers
remain separate work. The shared installer still reads this one local matrix.
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
