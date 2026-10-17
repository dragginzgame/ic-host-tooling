# Host qualification

Required hosts are Linux x86-64 and macOS 15 on Intel and Apple Silicon.
The imported filesystem/process APIs retain their platform gates; unsupported
platforms receive their existing explicit refusal rather than simulated behavior.

Select one changed crate with `PACKAGE=<crate>` for `make check`, `make clippy`,
`make docs-check` or `make test`. Artifact checks use its actual enabled gzip,
and Wasm features; production filesystem dependencies select none of them.
Check target-directory ownership before compilation. Preserve failure logs and artifacts.

`make test-artifacts-minimal` checks the artifact library without optional
features. `make test-tools-default` checks response decoding and Candid service
inspection with the default graph. `make test-tools-interface` adds Wasm export
interpretation; `make test-tools-certificate` adds strict owned certificate/hash-tree
decoding. `make test PACKAGE=ic-host-text` checks bounded line normalization,
escaping, progress frames and Unicode table alignment.
`make tools-features-check` verifies the normal parser dependency, including with
default features disabled, and the remaining optional owner boundaries. Native
CI includes these profiles. Tools includes the Candid parser unconditionally;
`ic-interface` adds Host Artifacts and Wasm parsing. The standalone Wasm inspector
retains its parser-free graph. These profiles must qualify on the same Rust 1.88
and native Linux/macOS matrix as the existing libraries.

Native CI prepares pinned Rust and the complete common toolsets, then verifies
the shared snapshot, declarations, formatter prerequisites, actual hook behavior,
release adapter,
common tool/LOC command wiring,
minimal artifact tests and each selected package with all features in sequence.
Shared owns the synthetic Make-consumer, host/IC/Rust installer and LOC parser
matrices in `shared-tooling/scripts/ci/test-portable-tools.sh`. Host retains its actual
Make routing, aggregate failure ordering, LOC arguments/status and pipe-jobserver
checks in `scripts/ci/test-make-snapshot.sh`, plus prepared-tool admission and
native qualification. Common fixture copying and per-target unsafe-mode probes
use Shared helpers; exhaustive flag aliases and inherited/replaced combinations
remain producer checks. Native CI's shared common-tool action retains setup/check
logs and publishes PATH only after both succeed. Snapshot checks retain all
selected bytes and modes;
the CI cache-key helper is an explicit snapshot root for its workflow caller.
The release gate uses the same complete host, IC and Cargo toolsets, even though
the libraries do not invoke every tool. Explicit setup runs `make install-tools`;
`make tools-check` verifies the set offline. Selected release preflight prepares
it; standalone verification never installs tools. Existing bundles and failed
evidence are retained. Rust toolchain bootstrap remains explicit.
Native CI uses the shared Cargo-tool build identity to restore builds.
Cache hits still run complete setup and admission;
cache transport failure falls through to setup. Main saves only after admission.
This cache does not qualify native macOS behavior or guarantee a setup duration.
`make formatting-hook-check` runs the shared adoption checker with Host's real
Cargo manifest, Rust input and Make formatters. Generic formatter prerequisite
and hook implementation matrices remain in Shared Tooling's own CI. Host keeps
prepared-tool admission and its actual Make failure-propagation checks.
Failed Git observations, partial staging, formatting failures and
unrelated edits are covered in disposable checkouts; the real index, working
files and hook configuration are preserved. The frontend formatter companion
retires with its unused generic fixture; Host has no frontend build step.
Both native macOS jobs run this same target through `make ci`; local Linux and
Bash 3.2 passes remain separate evidence under
[Host #55](https://github.com/dragginzgame/ic-host-tooling/issues/55).
Shared 0.3.1 setup checks IC platform/catalog admission and Rust/Cargo
availability before downloading common tools. These preflight checks do not
install a toolchain or replace the offline verification of installed tools.
Release preparation uses the shared literal metadata renderers with source-bound
member identities, with rollback qualification in disposable release fixtures.
No release-only executable extends the aggregate common toolset.
CI also owns native macOS execution; Linux qualification is not macOS evidence.
The reviewed Shared snapshot owns `ci/tool-versions.env` and `ci/ic-tools.tsv`.
Host uses these common defaults without catalog exceptions; Rust compiler policy
remains local. Host, IC and Rust exceptions, if
needed later, require independent explicit selection and native qualification.
The common catalog selects Binaryen 133 for all three supported hosts.
Explicit IC setup,
offline admission and optimization/execution smoke pass on Linux. Both native
macOS setup/check jobs passed on Host 0.12.9
([Host #53](https://github.com/dragginzgame/ic-host-tooling/issues/53)); later
changes retain their own qualification obligation.
Host has no production optimizer invocation or canister build.
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
