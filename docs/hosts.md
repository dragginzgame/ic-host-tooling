# Host qualification

Required hosts are Linux x86-64 and macOS 15 on Intel and Apple Silicon.
The FS package also owns the [native file CLI](file-cli.md). Its actual-executable
fixture uses the focused command in that guide; native CI includes it through
the existing package gate.

The imported filesystem/process APIs retain their platform gates; unsupported
platforms receive their existing explicit refusal rather than simulated behavior.

`ic-host-fs` can compile as a library for `wasm32-unknown-unknown`, but this is an
unsupported execution target. Its three regular lock-file openers return
`RegularFileLockError::UnsupportedPlatform` before filesystem effects; other APIs
retain their individual platform gates or native standard-library errors. Unix
descriptor-only APIs stay Unix-gated. This makes no promise of usable filesystem
IO in a canister, and downstream Wasm graphs should continue excluding Host FS
and native Build/Testkit harnesses. `make check-fs-non-unix` compiles the library, CLI,
test-support feature and non-Unix lock admission test without executing them.
It requires the Wasm standard-library target to be prepared explicitly; MSRV CI
selects that target during its existing toolchain setup.
That CI step scopes `RUSTUP_TOOLCHAIN=1.88.0` so Cargo uses the same compiler and
target installation, rather than the repository's independent development pin.

Select one changed crate with `PACKAGE=<crate>` for `make check`, `make clippy`,
`make docs-check` or `make test`. Artifact checks use its actual enabled gzip,
and Wasm features; production filesystem dependencies select none of them.
Check target-directory ownership before compilation. Preserve failure logs and artifacts.

`make test-artifacts-minimal` checks the artifact library without optional
features. `make test-artifacts-wasm` checks inspection without the encoder or
validator. `make test-artifacts-encoding` checks encoding alone; its normal/build
graph excludes wasmparser, while dev-only parser checks inspect emitted binaries.
`make test-artifacts-text` checks optional WAT compilation without a production
parser or validator. Source/output admission, symbolic names, stateful instructions
and separate full validation are covered in its all-feature profile.
Select `--features wasm-encoding,wasm-validation` for full instruction/type
validation of encoder fixtures. All-feature tests include both. The feature guard
checks these profiles on Linux and both macOS target graphs, including exclusion
of wasmparser's validation feature unless explicitly requested. These graph checks
do not replace native qualification. Both new capabilities require actual Rust
1.88 and native Linux/macOS evidence; native macOS remains separately reported.
The direct-dependency policy restricts cap-std, cap-fs-ext and tempfile to FS,
and wasmparser, wasm-encoder and wat to Artifacts, including dev/build and aliases.
All normal/build profiles except explicitly selected text compilation exclude
wat/wast. FS defaults, CLI, Process and the inspector exclude the cap/tempfile
backends. Independent `capability`, `temporary-file` and combined FS profiles
require only their selected backends on native targets; the non-Unix combined
profile excludes both. `make check-fs-non-unix` also compiles the unsupported-target contract.
FS tests cover held-parent operations, link/special-file refusal, bounded entries,
temporary-file cleanup and the read-only no-follow descriptor opener.
For feature qualification, run the FS library tests with no default features,
then with `capability`, `temporary-file` and all features. The latter includes
the real CLI tests. Native CI's all-feature package gate retains both backends'
regressions; Linux and both macOS graph checks independently enforce selection.
`make test-tools-default` checks response and ICP interpretation without
Platform or Candid. `make test-tools-candid` checks opt-in Candid decoding/service
facts. `make test-tools-interface` adds Wasm export
interpretation; `make test-tools-certificate` adds strict owned certificate/hash-tree
decoding. `make test PACKAGE=ic-host-text` checks bounded line normalization,
escaping, progress frames and Unicode table alignment.
`make test-tools-registry` checks golden Protobuf tags and shard admission.

Text's optional `yaml` feature and Tools' `icp-project` retain Rust 1.88.
Qualify YAML with `cargo +1.88.0 test -p ic-host-text
--features yaml --all-targets --locked --offline` and the equivalent Tools
`--features icp-project` selection. Tests cover ignored-field duplicate/tag/merge
policy, UTF-8/byte admission, alias handling and no external expansion. The byte
allowance is not a parser memory/CPU budget; backend recursion/repetition checks
are retained. The dependency guard confines serde_yaml to Text, rejects competing
frontends and excludes YAML from defaults and unrelated profiles. serde_yaml's
unsafe-libyaml backend remains selected in YAML-enabled graphs. All-feature MSRV
checks, including YAML, use the same 1.88 compiler and existing CI lane.

Certificate tests compare recursion refusal, semantic tags and indefinite byte
chunks with the same `serde_cbor` decoder used by IC Agent. Strict Host field,
array/byte-kind, trailing-input and redacted-error tests remain independent.
Certificate normal/build graphs require serde_cbor and exclude Minicbor; every
selected graph excludes Ciborium. The ledger profile retains its upstream
Minicbor dependency. Native Linux and both macOS hosts must qualify new decoder
source separately; named-target graph checks do not execute on those hosts.

`make test-tools-agent` exercises native construction, exact binding and response
ceilings. Its Wasm graph check observes dependency exclusion, not Tools execution.

The optional `icp-project` feature qualifies bounded effective-YAML projection;
default and certificate profiles exclude its YAML parser.

`make test-tools-encoding` checks canonical unsigned LEB128 admission. The decoder
adds no dependencies. `make tools-features-check` excludes Candid and its parser
from default, certificate-only and interface-only normal/build graphs, including
`certificate,ic-interface` together. `make test-tools-combined` exercises that
parser-free combination; all-feature tests retain the DID/Wasm comparison.
Native CI includes these profiles. Only explicit `candid` selects Platform's parser;
Agent and ledger evidence retain their required codec dependencies without it.
Registry selects Platform principal types without the codec. `ic-interface` adds
Host Artifacts and Wasm parsing. The standalone Wasm inspector
retains its parser-free graph. These non-YAML profiles must qualify on Rust 1.88
and native Linux/macOS matrix as the existing libraries.

Native CI prepares pinned Rust and the complete common toolsets, then verifies
the shared snapshot, declarations, formatter prerequisites, actual hook behavior,
release adapter,
common tool/LOC command wiring,
minimal artifact tests and each selected package with all features in sequence.
Release metadata backup/install/restoration and receipt custody use pinned Shared
helpers. Their synthetic failure matrices stay with Shared; Host's adapter fixture
checks the real rendering, source and committed-payload contracts with substituted
qualification commands and Git effects. Rendering, gates and recovery decisions
remain local; see [release custody](releases.md#reusing-metadata-preparation-and-evidence-custody).
Shared owns the synthetic Make-consumer, host/IC/Rust installer and LOC parser
matrices in `shared-tooling/scripts/ci/test-portable-tools.sh`. Host retains its actual
Make routing, aggregate failure ordering, LOC arguments/status and pipe-jobserver
checks in `scripts/ci/test-make-snapshot.sh`, plus prepared-tool admission and
native qualification. Native identity admission and pipe-jobserver observation
use the pinned Shared helpers; Host retains the runner matrix, selected macOS
major, PATH setup and real Make callers. Their synthetic observer matrices stay
with Shared. Common fixture copying and per-target unsafe-mode probes
use Shared helpers; exhaustive flag aliases and inherited/replaced combinations
remain producer checks. Native CI's shared common-tool action retains setup/check
logs and publishes PATH only after both succeed. Snapshot checks retain all
selected bytes and modes;
the CI cache-key helper is an explicit snapshot root for its workflow caller.
The release gate uses the common host and Cargo toolsets. Host has no product
IC executable caller, so its aggregate setup and CI do not select the IC bundle.
Shared 0.24.0's common Cargo set contains cargo-sort and cargo-sort-derives only.
Host has no extractor invocation: IC Build owns native extraction. Refresh the
common PATH through `make install-rust-tools`; retained extractor bundles are
not deleted or silently adopted as a product selection.
The individual IC setup/check commands remain available for explicit use.
Explicit setup runs `make install-tools`;
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
Release preparation uses the admitted IC Build metadata editor with source-bound
member identities, with rollback qualification in disposable release fixtures.
`make install-tools` includes the pinned release-only editor; `make tools-check`
admits it offline. The editor's Rust 1.91 floor is separate from Host's 1.88 MSRV.
Both native CI cache paths include its tool-store entries; its pin participates
in the cache key. `make release-adapter-check` exercises actual rendering with
substituted setup/Git effects, including first-use setup and offline refusal.
`make check-no-symlinks` checks indexed and tracked/nonignored working paths in CI
and the release gate. Publication checks complete regular source notices and the
prepared archive's exact notice bytes before upload.
The root catalog uses `Production dependencies`, and all six maintained library/
CLI members are `Production packages`. Shared test/production dependencies stay
in the production category. No current external catalog entry is test-only.
Internal aliases exist only for dependencies inherited by actual members;
workspace membership does not require a matching catalog dependency.
Clap is shared by production CLIs and maintained examples. FS gates its binary
and normal Clap edge behind `cli`; generic Artifacts/FS/Process library graphs
exclude it. Clap owns typed arguments, help, version and diagnostics; CLIs retain
their admission, output and operation semantics. Existing feature matrices use
Shared's Cargo graph checker for canonical names/counts while retaining Host's
enabled-validator/compiler feature checks. These are not deployed-artifact proof.
The Process alias is a test-only dependency of Tools' HTTP fixture and is excluded
from the other packages' normal/build graphs. Process itself remains a production
package. Tokio stays in the production category because Agent transport also
uses it transitively, although Host's direct Tokio declaration is dev-only.
Host's direct-dependency guard covers the known external IC package families in
`ci/cargo-dependency-boundaries.json`, including renamed, development, optional and
target-specific declarations. The broader Platform boundary also requires review
of new IC package identities and generated imports; the finite policy does not
automatically classify every future package. General-purpose dependencies remain
with their consuming crates. Current Host declarations already satisfy the rule.
The production section uses the canonical role header and grouped Cargo sorting:
`fmt`, `fmt-check` and release source qualification use the same ordering.
Prepared/committed release checks sort-check the exact saved root manifest;
unchanged member manifests belong to the previously qualified source.
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
validation surface. Native producer optimizer qualification moves to IC Build in
[Build PR #62](https://github.com/dragginzgame/ic-build/pull/62); integration and
native acceptance of that contribution remain separate. Host selects neither
the retired optimizer oracle nor the frozen Blob example-installation assessment.
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

## Dependency exclusions and deployment evidence

`make tools-features-check` checks transitive normal/build dependency exclusions
for the native target (or `CARGO_BUILD_TARGET`), including the combined Tools
features. `bash scripts/ci/check-tools-features.sh TARGET` inspects a named supported
Linux/macOS target without claiming native execution. The checker is in CI and
release validation. Cargo aliases are reported by their resolved package names.
Before those graph checks, Shared's canonical dependency-boundary checker reads
`ci/cargo-dependency-boundaries.json` and rejects direct Candid, parser,
certification and enumerated CDK dependencies in every workspace member, including
renamed, dev/build and target-specific declarations. Platform owns those upstream
edges. Unused root catalog declarations and future CDK package names still require
review; the checker covers declared member edges and exact policy names.
These checks preserve Tools/Text/inspection ownership boundaries; role comments
are explanatory and are not parsed as an executable exclusion policy.

The test-only Process alias is graph-checked out of the other production owners;
the selected Process root is intentionally excluded from that dependency rule.
There is no canister build or deployment entrypoint in Host. This repository publishes
libraries and native CLIs. It cannot certify a downstream deployment's features,
transitive test-only exclusions or final artifact digest. Deployment owners must
enforce their own exclusions against their locked target/feature graph and bind
successful build evidence to the installed artifact. Shared's sorter, pin checker
and snapshot verification do not provide that admission. Further test-only catalog
entries require explicit graph exclusions; comments alone do not activate them.
Native macOS and deployment evidence remain distinct from graph checks.
