# Current handoff

The maintainer authorized creating /home/adam/projects/ic-host-tooling with
four ic-host-* packages and copying the existing implementation as extraction input.
The committed library import is IC Host Tools 0.2.0 at
be7d73908225e73ab5babf8fbb9fe959f40d439b. Its dirty tooling/docs were excluded.
Canic's durable source is clean at the revision recorded in ci/extraction-sources.json.
The original IC Host Tools local checkout was subsequently retired as recorded
below. Other sibling checkouts and consumer dependencies remain unchanged.

Workspace setup separates artifact streams/inspection, filesystem operations,
admitted execution/Git observations and IC-specific adapters. Tests and examples
use their new owners. Bootstrap packages were non-publishable with inherited
0.2.0 metadata; the maintainer subsequently completed the breaking Git 0.3.0
release at 0456b40e116c7d070428b070d1c3befc36a9345d with tag v0.3.0. Package
publication policy now permits crates.io through the explicit commands below.
Registry publication has not been performed by the agent.

Shared Tooling remains separate and is adopted at exact reviewed revision
21f3ec3dd97f2968c9f0b08924451bb2f71770d1. The original bootstrap baseline was
d957d1f8801885c5b69e4a9ef900155f5f2a8a9d. All automation uses its local snapshot.
The original bootstrap authorization excluded commits, GitHub repository
creation, pushes, publication, installation and consumer adoption. Native macOS
execution remains CI-owned.
Focused Linux setup checks passed for each of the four changed packages:
all-target/all-feature tests and examples, Clippy with warnings denied,
Rustdoc, and Rust 1.88.0 all-target/all-feature compilation. Artifact tests
also passed with default features disabled. All 38 imported registry lock
selections remain unchanged. Workflow syntax and release Make adapters passed;
the adapter check used a substitute runner with no release effects.

The local extraction setup is ready for maintainer review. Consumer adoption,
native macOS qualification and actual registry publication remain separate work.

The installed workspace's 52-file Shared Tooling snapshot, parsed dependency
and inheritance declarations, formatting, documentation links and selected
package tests passed during bootstrap. The local formatting hook is installed.
The original initial-commit blocker is superseded: the local checkout has the
maintainer-owned commit 67c10830922f9249bfa30ec4f37010bf746519ce and a tracked
Cargo.lock. The public dragginzgame/ic-host-tooling repository was created on
request, and origin uses https://github.com/dragginzgame/ic-host-tooling.git.

The consumer release adapter now validates locked/offline Cargo metadata without
requiring registry-publishable packages. Git version/tag releases remain separate
from registry publication. The extraction's 0.3.0 notes are finalized; the new
compatible publication commands select pending 0.3.1 without changing the current
0.3.0 manifests or lockfile. Release preflight
now prepares and checks only pinned host parsers; the IC executable bundle stays
explicit setup. Changelog selection and rewriting use the shared finalizer with
the saved previous version, retaining the local top-entry and placement guards.
Focused Linux checks cover non-publishable packages, failed metadata queries,
dirty-source refusal, complete-ledger conflicts, historical-note preservation,
saved receipts and exact committed payloads beneath different working metadata.
Cargo metadata admission is real and locked/offline; Git, setup, qualification
and version mutation in these regression fixtures are substitutes.

The refreshed 54-file snapshot includes the shared formatter prerequisite checker
and regression fixture. It was exported from a clean temporary checkout of the
committed revision, excluding the sibling's unrelated working edits. Formatter
refusal checks, actual formatting, hook integration and Make release delegation
passed on Linux, as did the shared host-tool and release-runner substitute
fixtures. The new minimal artifact target passed 18 tests with optional
features disabled and is included in configured native CI. This batch has no
native macOS execution or complete release-gate evidence. The agent did not
execute a release command or change package versions, dependencies or tool pins.
The hook check also passed dependency-order perturbation and working/index
preservation. An earlier malformed perturbation moved a comment's dependency
association and failed the expected-byte comparison; its fixture remains at
/tmp/formatting-adoption.hOzBNO. The corrected input changed only dependency
order and passed; the failed attempt is not relabeled as a product failure.

The explicit `make publish` target delegates the four-package upload to native
Cargo with a clean-source check, locked dependencies, all features and an explicit
crates.io destination. Its separate `make publish-check` admits working edits for
a no-upload dry run. Both retain source identity, metadata, exact arguments and
Cargo logs under target/publish; upload recovery remains caller-owned as described
in [the publication procedure](../publishing.md). No automatic retry, version bump,
Git effect or artifact cleanup occurs. Credentials remain with Cargo.

Focused Linux publication fixtures passed with real locked/offline metadata and
substituted Git/upload commands: dry-run flags, dirty-upload refusal, package
policy and version rejection, failure propagation, pre-dispatch intent, concurrency
refusal and artifact preservation. Release adapter and Make delegation checks also
passed with substituted effects, retaining non-publishable-package coverage.
Formatting, declaration pinning, shell lint and documentation links passed.
With separately authorized network access, Cargo
1.99.0 `make publish-check` packaged and compiled all four 0.3.0 crates with all
features; every upload was aborted by Cargo's dry-run flag. Evidence is retained
in target/publish/check.ZutGR9. Inspection then found missing license text; inherited
license-file metadata now includes the canonical root MIT notice in every crate.
The repeated dry run passed in target/publish/check.JrsZix, and all four packaged
LICENSE files matched the root bytes. Both attempts retain their exact archives.
Cargo emitted an informational warning about having both the SPDX license and
license-file fields; both are retained to name the license and include its notice.
This validates packaging on Linux, not credentials, registry ownership, actual
uploads, consumer adoption or native macOS.

The earlier offline dry run failed at registry access and remains recorded in
target/publish/check.DadcVI. An offline package-only attempt assembled all four
archives and verified artifacts, then failed verifying filesystem with Cargo's
internal "no hash listed" error for the temporary registry; its evidence remains
in target/publish/package-check.XyNI7p. The subsequent online pass does not
reclassify either failed attempt. No dependency selections or tool pins changed,
and this batch has no full CI or native macOS execution evidence.

The maintainer authorized deleting /home/adam/projects/ic-host-tools. No sibling
local dependency paths pointed at that checkout. Its complete Git metadata,
uncommitted tooling/docs, untracked files, ignored files and retained artifacts
were archived and compared against the original before removal. The verified
backup is /tmp/ic-host-tools-retirement.an3fDZ/repository.tar, with its SHA-256
beside it. This is temporary storage, not permanent archival. The GitHub source
repository and existing consumer dependency selections were unchanged.
