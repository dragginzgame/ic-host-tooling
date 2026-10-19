# IC Build release and publication

The maintainer operates releases through the pinned Shared Tooling runner:

```bash
make install-release-tools
make release-patch
make publish
```

`make install-release-tools` explicitly prepares the pinned published Build
0.19.0 editor; `make release-tools-check` admits that prepared tool offline.
Validation and metadata preparation select its absolute path from the canonical
tool store and use this repository's default 8 MiB per-document allowance. No
manual environment exports are needed. CI prepares the same tool before its gate;
standard release preflight prepares and checks it after source/recovery admission.
Standalone preflight and validation only check, never install.

Explicit `CARGO_RELEASE_METADATA_EDITOR` and `CARGO_RELEASE_METADATA_MAX_BYTES`
overrides remain supported and validated. Invalid overrides refuse rather than
fall back. No current-source executable, sibling fallback, PATH lookup or implicit
installation runs during metadata rendering. The published editor's private
dependency graph is separate from the library graph being released.

Commit implementation and tooling changes before starting a release. Preflight
allows only `Cargo.toml`, `Cargo.lock` and `CHANGELOG.md` as uncommitted release
metadata; staged and working paths are checked independently. The default
destination is `origin` on `main`. Only direct delivery is qualified here;
`RELEASE_DELIVERY=pr` is refused before runner dispatch.

The runner computes the candidate, admits the source/destination, and runs the
same `make ci`, shell/workflow lint and history secret scan for every increment.
Standard entrypoints prepare the selected locked dependency cache and common
toolset. The declared development compiler, its Wasm target, the declared MSRV
and Linux validation tools must be prepared through the documented development
setup before the gate. Standalone preflight remains offline. Preparation updates
the workspace version, workspace-owned lock entries and dated changelog using
the canonical shared helpers; registry dependency selections are preserved.
Failed preparation restores the original metadata and retains its backup.
Complete preparation can be retried at the saved candidate without another bump.

The shared runner stages only the three declared metadata files, creates the
release commit and annotated version tag, and atomically pushes the selected
branch and tag. Normal release targets reconcile retained release intent;
`make release-resume VERSION=X.Y.Z` selects only that saved release. Recovery
checks use `RELEASE_COMMIT`, including when newer committed source exists.
Metadata checks apply the same grouped manifest ordering as `make fmt-check`,
including staged preparation, lost-reply recovery and the selected commit export.
Role annotations document dependency use; these checks do not enforce transitive
deployment exclusions or admit downstream canister artifacts.
An interrupted partial metadata write that cannot pass the prepared check needs
inspection of the retained backup; do not discard the release plan or artifacts.

Publication is separate. `make publish` requires a clean checkout at the exact
annotated tag for the workspace version, then invokes
`cargo publish -p ic-build --locked --registry crates-io`. Cargo selects credentials
and enforces registry eligibility. `make publish-check` uses the same admission
with `--dry-run`; `make package` creates and verifies a local archive.
Neither release nor publication invokes `cargo clean`. Retain release archives
and failed evidence until publication/recovery work is complete; cleanup remains
an explicit maintainer action.

`make check-release-adapters` qualifies metadata rollback, external-lock
preservation, real index admission, verification gate routing, candidate recovery,
selected-commit checks and publication failures in disposable repositories.
Registry calls are substitutes. The shared routing check owns the release
entrypoint dispatch. These fixtures do not establish live publication or native
macOS qualification; the required native macOS 15 Intel/Apple Silicon evidence
remains tracked in [IC Build #1](https://github.com/dragginzgame/ic-build/issues/1).
See the [shared release contract](releases.md) for runner ownership and recovery.
