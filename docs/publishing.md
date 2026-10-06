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
prepared Cargo 1.99 plus the pinned local jq. Cargo owns dependency ordering,
package verification and index waiting. Package policy restricts each crate to
crates.io; the wrapper always selects that registry explicitly. Credentials stay
with Cargo's normal login or credential-provider configuration. Neither command
creates commits, changes versions, creates tags or pushes Git refs. Git release
commands never invoke publication automatically.

Package metadata uses the standard MIT SPDX identifier. Each crate's `LICENSE`
is a regular copy of the canonical root notice, compatible with the formatting
hook and included in the package archive without redundant `license-file`
metadata. Both publication commands reject missing, symlinked or differing
notices before invoking Cargo. Update all four copies when the root notice changes.

Cargo's latest registry lookup reports all four `0.3.0` versions already present.
Do not republish those versions with changed metadata. The pending compatible
fixes belong to `0.3.1`; package versions remain unchanged until the maintainer
prepares that release. Before upload, review the dry run and confirm that the
committed version is the intended registry release. A Git tag does not prove
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
