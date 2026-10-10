# Host qualification

Required hosts are Linux x86-64 and macOS 15 on Intel and Apple Silicon.
The imported filesystem/process APIs retain their platform gates; unsupported
platforms receive their existing explicit refusal rather than simulated behavior.

Select one changed crate with PACKAGE=<crate> for make check, make clippy,
make docs-check or make test. Artifact checks use its actual enabled gzip,
archive and Wasm features; production filesystem dependencies select none of them.
Check target-directory ownership before compilation. Preserve failure logs and artifacts.

Native CI prepares pinned Rust and the complete common toolsets, then verifies
the shared snapshot, declarations, formatter prerequisites, release adapter,
common tool/LOC command wiring,
minimal artifact tests and each selected package with all features in sequence.
The release gate uses the same complete host, IC and Cargo toolsets, even though
the libraries do not invoke every tool. Explicit setup runs `make install-tools`;
`make tools-check` verifies the set offline. Selected release preflight prepares
it; standalone verification never installs tools. Existing bundles and failed
evidence are retained. Rust toolchain bootstrap remains explicit.
Shared 0.3.1 setup checks IC platform/catalog admission and Rust/Cargo
availability before downloading common tools. These preflight checks do not
install a toolchain or replace the offline verification of installed tools.
Host extends the aggregate with its pinned `cargo-set-version` executable,
using the shared installer and receipt checks. Native CI prepares and checks
this selection too; Linux installation and substitute installer fixtures do not
qualify its native macOS build or execution.
CI also owns native macOS execution; Linux qualification is not macOS evidence.
`ci/ic-tools.tsv` is a consumer-owned pin selection, excluded from automatic
snapshot refreshes. Host selects Binaryen 133 for all three supported hosts,
using the reviewed Shared 0.3.7 version and archive digests. Explicit IC setup,
offline admission and optimization/execution smoke pass on Linux; native macOS
setup/check remains tracked in [Host #53](https://github.com/dragginzgame/ic-host-tooling/issues/53).
Host has no production optimizer invocation or canister build;
archive fixtures containing `binaryen-version_132` exercise member lookup only.
The upstream Node-based optimizer smoke is not added to this Rust library's
validation surface. Native producer optimizer qualification remains owned by
[Shared #102](https://github.com/dragginzgame/shared-tooling/issues/102).
Pin changes use explicit IC setup, preserving old bundles, and need native
acceptance independently of snapshot integrity.
Shared 0.2.0 adoption removed its PocketIC rows while retaining the other tools.
Host library gates have no PocketIC callers, so no simulator dependency or setup
is added.
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
