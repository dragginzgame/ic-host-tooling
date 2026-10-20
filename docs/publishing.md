# Releasing and publishing the workspace

## Git releases

The maintainer-operated `make release-patch`, `make release-minor` and
`make release-major` commands use the [shared release contract](releases.md).
Host's [delivery policy](../ci/release-delivery.txt) selects `RELEASE_DELIVERY=direct`:
preparation, commit, tag and atomic branch/tag push. Other delivery selections
are rejected before execution;
the shared runner's optional PR flow is not qualified by this adapter.
These commands do not publish crates to a registry.

Prepare the development toolchain selected in [rust-toolchain.toml](../rust-toolchain.toml),
then run `make install-tools` and `make tools-check`. Shared Tooling owns
[common setup and executable storage](local-setup.md). Release preparation uses
the released IC Build `ic-build-release-metadata` editor, pinned separately in
`ci/release-editor.env` and selected through its verified tool-store receipt.
Host admits its
explicit workspace member paths and reads member identities from the selected
Git source; IC Build projects owned dependency requirements for live and saved
manifests. Recovery does not need today's member paths or a reconstructed old
workspace. It validates candidate TOML and the prepared
locked Cargo graph, restoring original bytes/modes after failure or catchable
interruption and retaining recovery evidence. Shared retains the preparation
transaction, receipts and rollback. `make install-tools` includes the release
editor; `make tools-check`/`make release-editor-check` admit it offline. Preflight
can prepare a missing editor after source admission without per-shell exports;
standalone verification never installs. The editor requires Rust 1.91 or later;
Host libraries retain MSRV 1.91. No Cargo-edit command renders release metadata.

The read-only `make release-version` query needs prepared host parsers and never
installs them; missing parsers direct the caller to `make install-tools` and
`make tools-check`. Explicit server setup fetches locked dependencies before
building its setup CLI offline. Selected release preflight prepares the complete toolset and
fetches the selected dependencies with `cargo fetch --locked` before offline
validation. Cargo honours explicit offline settings: missing inputs stop the
release without an online retry. Standalone `make release-verify` checks prepared
inputs offline and requires the runner's selected release environment; it is not
a standalone release command. Neither cache preparation nor validation changes
the selected dependency versions.

The complete gate runs `make clippy-workspace` after prerequisite admission and
before expensive tests or builds, with the existing workspace/all-target/all-feature
scope and warnings denied. Failure stops later stages under parallel Make too.
Release verification and CI both use `make msrv-qualification` for native
workspace, Testkit Wasm and unsupported-filesystem compilation at Rust 1.91.
Both numbered pending-heading spellings, `## [X.Y.Z]` and
`## [X.Y.Z] - Unreleased`, are finalized to the saved UTC date during preparation.

Follow the [shared recovery procedure](releases.md#source-changes-after-saved-preparation-intent)
for interrupted preparation and exact-version resume. Host preserves prior
validation receipts, notes and logs before fresh verification. Retain unfinished
plans, earlier tool installations and failed artifacts; a retry must not relabel
old evidence as validation of new source. Native qualification belongs to the
[host guide](hosts.md), independently of release and publication effects.

## Registry publication

`make publish-check` runs Cargo's publication checks with all features
and `--locked --registry crates-io --dry-run`. It packages and verifies the selected
workspace packages through exact `--package NAME@VERSION` selectors without
uploading: the five Host libraries, `ic-build`, `ic-testkit`,
`ic-testkit-native` and the `ic-host-wasm-inspect` CLI. The development driver
and Wasm probe remain private. It admits working edits so packaging can be reviewed
before committing. It can access the registry; set `CARGO_NET_OFFLINE=true` for
an offline attempt against already prepared caches. Registry lookup can require
network access even with cached dependencies. Offline failure never triggers an
online retry.

Both publication modes require prepared dependencies and pass the same offline
`make clippy-workspace` gate after source admission, before packaging or registry
effects. The wrapper never installs tools or fetches missing lint dependencies.

`make publish` uploads the current committed workspace version to crates.io.
It requires a clean working tree, synchronized selected publishable packages and
prepared Cargo 1.99, Git, tar and the pinned local jq. Cargo owns dependency ordering,
package verification and index waiting. Package policy restricts each package to
crates.io; the wrapper always selects that registry explicitly. Credentials stay
with Cargo's normal login or credential-provider configuration. Neither command
creates commits, changes versions, creates tags or pushes Git refs. Git release
commands never invoke publication automatically.

Before any upload, the wrapper prepares all selected archives with `cargo package
--package NAME@VERSION` for every selected public package, with
`--all-features --locked --no-verify`. This step resolves package
dependencies but does not compile or upload; the final Cargo publication still
verifies packages. Each archive's `.cargo_vcs_info.json` must name the selected
clean `HEAD` and the package directory recorded by admitted Cargo metadata.
Package paths must be physical directories beneath the checkout, without parent
traversal or CR/LF. License notices are checked in those same directories after
metadata admission and before package preparation or upload. The explicit package
allowlist remains separate from workspace membership. All package metadata must
declare `https://github.com/dragginzgame/ic-tooling` as the repository.

Shared's [canonical provenance helper](../scripts/ci/check-cargo-package-provenance.sh)
checks the prepared archives and fetches that exact commit from the declared HTTPS repository
into a fresh bare object store beneath the publication attempt. Failed fetches,
mismatched commit identities, missing/dirty package provenance and source changes
stop before upload. Git prompts are disabled; ordinary trusted Git transport
configuration still applies. The source request, archive provenance, fetch log
and retrieved objects are retained in the admission attempt's `provenance/`
directory. This guard is a remote-read effect of the
explicit upload command; it never pushes a ref or creates a commit.

The [reviewed Shared snapshot](../.shared-tooling.snapshot) supplies this helper and the
[selected-source reader](../scripts/ci/read-cargo-source-context.sh), alongside
the existing release metadata/receipt custody. Host selects packages, manifests,
versions and recovery; Shared owns exact commit retrieval, archive provenance
and selected-source member observation. There are no local source-evidence copies.
Both Git object formats, ambiguous parser-stream refusal and retained failure
evidence remain part of admission; replacing the helper owner removes no check.

`publish-check` retains its dirty-worktree dry-run contract and does not claim
remote provenance verification. The upload guard does not establish CI success
or native host qualification, and callers must keep source stable throughout
packaging/publication. See [#7](https://github.com/dragginzgame/ic-tooling/issues/7).

Package metadata uses the standard MIT SPDX identifier. Each selected package's
`LICENSE` is a regular copy of the canonical root notice, compatible with the formatting
hook and included in the package archive without redundant `license-file`
metadata. Both publication commands reject missing, symlinked or differing
notices before invoking Cargo. The CI publication fixture preserves real package
notices, so the complete release gate observes source drift instead of synthesizing
matching notices. Update every selected package's copy when the root notice changes.

Before upload, review the dry run and confirm that the committed version is the
intended registry release. Inspect current registry state for that exact version;
never republish an existing version with changed source or metadata. Pending
changelog entries do not change package versions. A Git tag does not prove
registry publication or host qualification.

Host retains its selection, metadata and provenance in
`target/publish/admission/check.*` or `target/publish/admission/publish.*`.
The `publication-attempt` file links to the corresponding fresh command directory
under `target/publish/check.*` or `target/publish/publish.*`. The selected
[Shared attempt helper](cargo-publication.md) records exact package/version
selectors, source, arguments and working status before its dispatch marker. It
retains Cargo output plus separate `cargo-status`, `log-status` and combined
`status` files. Cargo failure takes priority over a logger failure; a logger failure
after successful Cargo execution also refuses success.

Build/package artifacts and prior attempt evidence remain in `target`; there is
no artifact cleanup. Host's `target/publish/lock` covers admission through command
completion. Shared's stable `target/publish/.cargo-publication.lock` also blocks
another command attempt and remains after interruption or incomplete evidence.
A dispatch marker without completed statuses means the upload result is uncertain.
Before removing only a stale lock, inspect its recorded owner and reconcile the
exact source/packages against registry state. Neither command removes stale locks,
retries an upload or overwrites earlier evidence. Directory paths containing CR/LF
are refused before Cargo admission or dispatch; ordinary spaces remain supported.

Workspace uploads are not atomic. A failure, timeout or lost reply can leave some
packages published. The wrapper stops and does not retry, skip existing versions,
bump versions or interpret failure as absence. Retain the intent and artifacts,
inspect the exact name/version entries on crates.io, and compare their published
source with the recorded source before continuing. Stop if registry access or
payload identity is uncertain. After confirming which packages are present,
publish only confirmed missing packages using Cargo's `--package` selections,
in dependency order: artifacts, filesystem, process, IC tools, then the Wasm CLI.
Do not rerun the full workspace upload blindly or change source under the same version.

The [Cargo publish documentation](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
defines dry-run, registry, packaging and index-waiting behavior. The offline
`make publish-command-check` fixtures verify the consumer wrapper and shared
attempt owner using substituted Git, Cargo and logger commands, including source
drift, original status preservation, interruption and refused replay. They do not
contact a registry or publish.
