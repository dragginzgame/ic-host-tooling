# Publishing the workspace

`make publish-check` runs Cargo's workspace publication checks with all features
and `--locked --registry crates-io --dry-run`. It packages and verifies all four
crates without uploading. It admits working edits so packaging can be reviewed
before committing. It can access the registry; set `CARGO_NET_OFFLINE=true` for
an offline attempt against already prepared caches. Registry lookup can require
network access even with cached dependencies. Offline failure never triggers an
online retry.

`make publish` uploads the current committed workspace version to crates.io.
It requires a clean working tree, four synchronized publishable packages and
prepared Cargo 1.99, Git, tar and the pinned local jq. Cargo owns dependency ordering,
package verification and index waiting. Package policy restricts each crate to
crates.io; the wrapper always selects that registry explicitly. Credentials stay
with Cargo's normal login or credential-provider configuration. Neither command
creates commits, changes versions, creates tags or pushes Git refs. Git release
commands never invoke publication automatically.

Before any upload, the wrapper prepares all four archives with `cargo package
--workspace --all-features --locked --no-verify`. This step resolves package
dependencies but does not compile or upload; the final Cargo publication still
verifies packages. Each archive's `.cargo_vcs_info.json` must name the selected
clean `HEAD` and its exact `crates/<package>` path. All package metadata must
declare `https://github.com/dragginzgame/ic-host-tooling` as the repository.

The wrapper then fetches that exact commit from the declared HTTPS repository
into a fresh bare object store beneath the publication attempt. Failed fetches,
mismatched commit identities, missing/dirty package provenance and source changes
stop before upload. Git prompts are disabled; ordinary trusted Git transport
configuration still applies. The source request, archive provenance, fetch log
and retrieved objects are retained. This guard is a remote-read effect of the
explicit upload command; it never pushes a ref or creates a commit.

`publish-check` retains its dirty-worktree dry-run contract and does not claim
remote provenance verification. The upload guard does not establish CI success
or native host qualification, and callers must keep source stable throughout
packaging/publication. See [#7](https://github.com/dragginzgame/ic-host-tooling/issues/7).

Package metadata uses the standard MIT SPDX identifier. Each crate's `LICENSE`
is a regular copy of the canonical root notice, compatible with the formatting
hook and included in the package archive without redundant `license-file`
metadata. Both publication commands reject missing, symlinked or differing
notices before invoking Cargo. Update all four copies when the root notice changes.

Before upload, review the dry run and confirm that the committed version is the
intended registry release. Inspect current registry state for that exact version;
never republish an existing version with changed source or metadata. Pending
changelog entries do not change package versions. A Git tag does not prove
registry publication or host qualification.

Admitted invocations retain source identity, metadata, exact arguments and Cargo's
output under `target/publish/check.*` or `target/publish/publish.*`. Upload intent
is recorded before Cargo runs. Build/package artifacts remain in `target`; there
is no artifact cleanup. A concurrent invocation stops at the publication lock.
If a process is killed, inspect the recorded owner before removing only a stale
`target/publish/lock` directory.

Workspace uploads are not atomic. A failure, timeout or lost reply can leave some
crates published. The wrapper stops and does not retry, skip existing versions,
bump versions or interpret failure as absence. Retain the intent and artifacts,
inspect the exact name/version entries on crates.io, and compare their published
source with the recorded source before continuing. Stop if registry access or
payload identity is uncertain. After confirming which crates are present,
publish only confirmed missing packages using Cargo's `--package` selections,
in dependency order: artifacts, filesystem, process, then IC tools. Do not rerun
the full workspace upload blindly or change source under the same version.

The [Cargo publish documentation](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
defines dry-run, registry, packaging and index-waiting behavior. The offline
`make publish-command-check` fixture verifies the consumer wrapper using
substituted Git and upload commands; it does not contact a registry or publish.
