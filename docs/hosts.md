# Host qualification

Required hosts are Linux x86-64 and macOS 15 on Intel and Apple Silicon.
The imported filesystem/process APIs retain their platform gates; unsupported
platforms receive their existing explicit refusal rather than simulated behavior.

Select one changed crate with `PACKAGE=<crate>` for `make check`, `make clippy`,
`make docs-check` or `make test`. Artifact checks use its actual enabled gzip,
archive and Wasm features; production filesystem dependencies select none of them.
Check target-directory ownership before compilation. Preserve failure logs and artifacts.

`make test-artifacts-minimal` checks the artifact library without optional
features. `make test-tools-response` checks the response/text library,
`make test-tools-interface` checks Wasm export interpretation,
`make test-tools-service` checks parser-backed Candid service inspection,
`make test-tools-extraction` checks native Candid extraction,
`make test-tools-limits` checks IC resource comparison.
`make test-tools-certificate` checks strict owned certificate/hash-tree decoding.
`make test PACKAGE=ic-host-text` checks escaping and Unicode table alignment.
`make tools-features-check` verifies its production dependency exclusions and
the explicitly selected extractor's required owners. Native CI includes these profiles.
The `candid-service` profile introduces the optional Candid parser;
`ic-interface` excludes it. Both must qualify on the same Rust 1.88 and native
Linux/macOS matrix as the existing libraries.

Native CI prepares pinned Rust and the complete common toolsets, then verifies
the shared snapshot, declarations, formatter prerequisites, actual hook behavior,
release adapter,
common tool/LOC command wiring,
minimal artifact tests and each selected package with all features in sequence.
The release gate uses the same complete host, IC and Cargo toolsets, even though
the libraries do not invoke every tool. Explicit setup runs `make install-tools`;
`make tools-check` verifies the set offline. Selected release preflight prepares
it; standalone verification never installs tools. Existing bundles and failed
evidence are retained. Rust toolchain bootstrap remains explicit.
Native CI uses the shared Cargo-tool build identity plus Host's release-tool
inputs to restore builds. Cache hits still run complete setup and admission;
cache transport failure falls through to setup. Main saves only after admission.
This cache does not qualify native macOS behavior or guarantee a setup duration.
`make formatting-hook-check` runs the canonical shared hook regressions and the
shared adoption checker with Host's real Cargo manifest, Rust input and Make
formatters. Failed Git observations, partial staging, formatting failures and
unrelated edits are covered in disposable checkouts; the real index, working
files and hook configuration are preserved. The shared frontend helper is a
companion of the canonical regression fixture, not a Host frontend build step.
Both native macOS jobs run this same target through `make ci`; local Linux and
Bash 3.2 passes remain separate evidence under
[Host #55](https://github.com/dragginzgame/ic-host-tooling/issues/55).
Shared 0.3.1 setup checks IC platform/catalog admission and Rust/Cargo
availability before downloading common tools. These preflight checks do not
install a toolchain or replace the offline verification of installed tools.
Host extends the aggregate with its [release executable](publishing.md#git-releases),
using the shared installer and receipt checks. Native CI prepares and checks
this selection too; Linux installation and substitute installer fixtures do not
qualify its native macOS build or execution.
CI also owns native macOS execution; Linux qualification is not macOS evidence.
The reviewed Shared snapshot owns `ci/tool-versions.env` and `ci/ic-tools.tsv`.
Host uses these common defaults without catalog exceptions; its release executable
pin and Rust compiler policy remain local. Host, IC and Rust exceptions, if
needed later, require independent explicit selection and native qualification.
The common catalog selects Binaryen 133 for all three supported hosts.
Explicit IC setup,
offline admission and optimization/execution smoke pass on Linux. Both native
macOS setup/check jobs passed on Host 0.12.9
([Host #53](https://github.com/dragginzgame/ic-host-tooling/issues/53)); later
changes retain their own qualification obligation.
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
