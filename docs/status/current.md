# Current handoff

## Latest qualification: 2026-10-07

The maintainer released 0.3.2 at c7c0d85765054909c05d86f6d3fd2c9965510335.
The current uncommitted batch targets compatible 0.3.3; package manifests,
Cargo.lock, tool pins and the adopted Shared Tooling snapshot remain unchanged.
The history below records earlier batches and their evidence, not current versions.

[CI for the released source](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37589678525)
passed Linux and MSRV but failed on macOS 15 Intel and ARM at the same path test:
native canonicalization accepted `file/..` where Linux rejects it. The repaired
test compares native results, including after missing-prefix normalization,
without changing production resolution or weakening actual native errors.
Seven focused path tests pass on Linux. Native macOS requalification is pending.

The shared capture entry accepts the caller's existing `Command`, preserving
opaque platform setup and executable policy while using the admitted runner's
single bounded capture engine. `ToolError` now exposes borrowed evidence and
execution failure accessors. No production symbols were removed or compatibility
reexports added. Scope and consumer coordination remain on
[Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5) and
[Host #9](https://github.com/dragginzgame/ic-host-tooling/issues/9).

Focused offline Linux checks passed: 32 process tests, 19 dependent tools tests
and examples, process/filesystem strict Clippy and Rust 1.88 compilation, strict
process Rustdoc, formatting, documentation links, pins, feature graphs and exact
58-file snapshot verification. Logs remain under /tmp/ic-host-033-*.log, including
the original macOS CI failure and the initial test-style Clippy failure. No full
local CI/release gate or native macOS execution was performed.

All sibling repositories stayed read-only. A Canic candidate against inspected
5e0dd13f7b494e800201841223a6167b544a5892 is retained at
/tmp/ic-host-033-canic-capture/consumer.patch with source digests and logs. It
removes the local child owner/nonblocking reader loop, retains the product error
adapter and requires future published 0.3.3. The patch applies to that source;
two tests pass in a private reduced-error fixture, not Canic's full dependency
graph. Consumer adoption is coordinated on
[Canic #458](https://github.com/dragginzgame/canic/issues/458).
No commit, tag, push, publication, dependency upgrade or tool download occurred.

## Historical evidence

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
25e7ce83149e081e4dcc52c55c33724e44153f2a (0.1.14). The prior adoption was
46c02774a8335cb3949d6f04284c4f53375353c1. The original bootstrap baseline was
d957d1f8801885c5b69e4a9ef900155f5f2a8a9d; the earlier adopted revision was
21f3ec3dd97f2968c9f0b08924451bb2f71770d1. All automation uses its local snapshot.
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
That attempt emitted Cargo's redundant SPDX license/license-file warning. The
subsequent correction retains only inherited MIT SPDX metadata, with per-crate
LICENSE symlinks to the canonical root notice. Cargo packages those links as
regular files; all four archived notices matched the root bytes. The online dry
run passed without the license warning in target/publish/check.Y2p7MG, with its
exact archives retained. Cargo also reported all four 0.3.0 versions already
present in the crates.io index. The compatible correction remains in pending
0.3.1; no package version changed and no upload occurred during these checks.
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

The requested sibling extraction review covered the 13 sibling repositories at
discovery level and traced the eight consumers still selecting ic-host-tools 0.2.
Only this workspace was mutated; sibling working trees and retained artifacts were
read-only. The previous license-warning correction remains in the working tree.
Three compatible shared additions extend the pending 0.3.1 batch: MatchingWriter,
bounded zero-timestamp gzip encoding, and streamed durable replacement through
write_with. Byte-based writes converge through the same existing commit engine.
Exact committed source file digests and extraction selections are recorded in
ci/extraction-sources.json; selected files matched HEAD despite unrelated sibling
dirty work. The existing Canic copyright notice is preserved in the root license.

Focused Linux checks passed: all-feature artifact library tests (46), filesystem
library tests (25), minimal artifact tests (21), strict all-target/all-feature
Clippy, Rustdoc with warnings denied, and Rust 1.88 all-target/all-feature checks
for the two changed packages. Offline cache preparation, formatting, declaration
pinning and documentation links also passed. An initial Clippy attempt rejected
test-local items placed after statements; moving the fixture declarations before
statements resolved it. No dependencies, versions, tool pins, commits, releases,
publication, consumer builds or full CI were performed in this extraction batch.
Native macOS qualification remains outstanding.

The immutable [sibling review evidence](../reports/audits/2026/10/06/sibling-host-extraction/01/report.md)
records ownership traces, retained stronger contracts and qualification limits.
The authorized adoption feedback is on GitHub: existing
[Canic #458](https://github.com/dragginzgame/canic/issues/458#issuecomment-6021451439)
was updated; new issues are
[IC Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13),
[IC Query #11](https://github.com/dragginzgame/ic-query/issues/11),
[IC Backup #11](https://github.com/dragginzgame/ic-backup/issues/11),
[IC Memory #16](https://github.com/dragginzgame/ic-memory/issues/16),
[IcyDB #307](https://github.com/dragginzgame/icydb/issues/307),
[IC Blob Storage #14](https://github.com/dragginzgame/ic-blob-storage/issues/14) and
[Toko Miner #30](https://github.com/dragginzgame/toko-miner/issues/30).
Shared API implementation is not consumer adoption: sibling copies still exist
until their owners complete those changes. New APIs remain uncommitted pending
0.3.1; issue bodies explicitly distinguish them from the released split 0.3.0 APIs.

The maintainer-requested second pass found three more consumer reuse sites using
existing 0.3.0 primitives: Canic network SHA-256 codecs, Query's confined bounded
stream collection and IcyDB's diagnostic file/stdin collection. Existing adoption
issues received source-backed comments with error/accounting acceptance checks;
no additional shared API was justified. The [second-pass evidence](../reports/audits/2026/10/06/sibling-host-extraction/02/report.md)
records current source identities separately from the first run. write_with docs
now distinguish returned errors and attempted staging cleanup from panic/interruption
retention. Focused filesystem Rustdoc, formatting and documentation checks passed;
this documentation-only correction adds no new runtime or native macOS evidence.

A follow-on maintainer-authorized extraction adds explicit-base missing-suffix
path resolution and descriptor lock waiting to the pending 0.3.1 batch. The
existing pathname progress helper shares the acquisition engine while preserving
its one-second reporting policy. The descriptor entry point lets Testkit retain
its opener, 25ms polling cap and phase heartbeat mapping. The source observation
and selected mechanics are recorded in ci/extraction-sources.json; cache domains,
pruning and retained-lock lifecycle remain in Testkit.

Consumer preparation exposed a maximum-filename regression in durable staging:
including a 255-byte destination name in the staging name failed with Linux
ENAMETOOLONG. The failed behavior log is
/tmp/ic-testkit-host-long-name-regression.log; its private fixture is retained.
An earlier attempt failed compiling the new collision fixture because its
existing hook's mode argument was missing; that attempt remains separately in
/tmp/ic-testkit-host-long-name-before.log. Short owned staging names now support
maximum-length destinations, exclude the output name even case-insensitively,
retry collisions within the existing bound and preserve unowned files.

Focused filesystem library tests (34), all-target strict Clippy, Rust 1.88
all-target checks and strict Rustdoc pass on Linux. The interruption test injects
Interrupted at the acquisition boundary; it does not claim native signal delivery
qualification. Real descriptor contention, callback failure, unowned collision
preservation, selected staging-namespace destinations and complete publication
are exercised. Native macOS qualification remains outstanding.

The prepared Testkit consumer patch is retained at
/tmp/ic-testkit-host-adoption/consumer-adoption.patch. A private consumer fixture
at /tmp/ic-testkit-host-extraction.b9rs0cq9 tested it with explicit local-source
patch selections and unchanged package versions; source hashes are recorded in
its reviewed-host-inputs.json. Focused digest/publication, missing-parent path,
lock heartbeat, all 28 transaction, linked/restricted Wasm publication, copied
stamp and partial-materialization tests pass using Testkit's normal target
folder. This is source qualification, not registry or live consumer adoption.
The actual consumer remains on published 0.3.0 until reviewed shared publication;
no version, commit, tag, push, upload or full gate was performed. Implementation
feedback is on [host-tooling #1](https://github.com/dragginzgame/ic-host-tooling/issues/1),
[host-tooling #2](https://github.com/dragginzgame/ic-host-tooling/issues/2) and
[Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13).

The requested [consumer crate-usage review](../reports/audits/2026/10/06/consumer-crate-usage/01/report.md)
records current working-tree imports separately from published dependency edges.
Its saved manifests, lock edges and source digests cover the eight direct consumers
and discovery across all 13 siblings. It identifies remaining Canic/Toko source
alignment failures and older published transitive owners without claiming new
consumer execution evidence. Siblings remained read-only; the new concrete Toko
and Backup findings were added to their existing adoption issues.

The requested Shared Tooling review refreshed the same 54-file adoption from
21f3ec3dd97f2968c9f0b08924451bb2f71770d1 to committed revision
46c02774a8335cb3949d6f04284c4f53375353c1 (0.1.11). The sibling's dirty pending
0.1.12 work was excluded by refreshing from a private checkout of that commit.
The new snapshot includes exact large-component changelog comparisons and the
paired maintenance guidance; no new shared file surfaces were added.

The release adapter now delegates manifest version observations to the existing
shared Cargo/TOML reader. Its consumer-owned AWK projection retains exact
version-only payload preparation, synchronized path requirements and owned
lockfile reads. Valid inline version comments survive preparation and recovery;
failed parser output, failed Cargo path observations, duplicate workspace keys
and unsynchronized path versions are rejected before setup or mutation. Version
queries require the pinned jq/yq set to be prepared first, as documented in README.

Focused offline Linux checks passed: release version observation, the expanded
adapter fixture, shared Cargo metadata fixtures, release-runner command-stub
fixtures, ShellCheck, Bash syntax, snapshot integrity, dependency declarations,
documentation links and formatting. Git effects, tool setup and version mutation
were substituted in release fixtures; no release operation was run. Cargo.lock,
tool pins and package versions are unchanged by this review. Native macOS
qualification and full release gates remain separate from this evidence; these
changes are uncommitted additions to the pending 0.3.1 batch.

The maintainer's commit attempt exposed an integration failure: the four crate
LICENSE symlinks conflict with the adopted formatting hook's regular-file-only
snapshot contract. They are replaced by regular byte-identical copies of the
canonical root notice; the shared hook remains unchanged. Both publication modes
now reject missing, symlinked or differing notices before Cargo dispatch. Focused
publication fixtures cover those refusals with substituted Git/upload effects.
Offline Cargo package listings include LICENSE for each crate and all four
working copies match the root. ShellCheck, Bash syntax, formatting, documentation
links and snapshot integrity passed. No new archive verification, native macOS
qualification, commit, release or publication occurred in this repair.
The repaired files were staged without disturbing the existing selection, and
the actual pre-commit hook passed against that selected index payload on Linux.

After the maintainer committed the batch as 5e73861, a filesystem test failed at
the immediate close/reacquisition assertion. The descriptor test lacked the
subprocess isolation already used by the older path-lock test: a parallel process
spawn can inherit a locked descriptor until exec, briefly retaining exclusion
after the originating descriptor closes. The descriptor test now runs alone in
an exact-test subprocess, preserving its contention, returned ownership and
immediate close-release assertions. Production acquisition behavior is unchanged.
This remains compatible pending 0.3.1 work; no tag or package version changed.

The reported failure was not reproduced in the isolated baseline run or 20
parallel baseline runs; the inherited-descriptor explanation is source-based,
not a captured failing interleaving. Baseline logs remain in
/tmp/ic-host-lock-before-*.log. After isolation, all four lock-wait module tests,
all 34 filesystem tests and 20 further library runs with 32 test threads passed;
the repetition logs remain in /tmp/ic-host-lock-after-*.log. Strict all-target
Clippy, Rust 1.88 all-target checks, formatting and documentation checks passed
offline on Linux. Full release gates and native macOS qualification were not
performed; this repair remains uncommitted for the maintainer.

The maintainer released 0.3.1 at 38a2a5127be064014e6d39d72d0300ffb2cf20be.
The current [host reuse audit](../reports/audits/2026/10/07/host-reuse/01/report.md)
repairs missing-path traversal/dangling-target resolution and adds an explicit
caller-owned missing-target depth allowance in pending 0.3.2; manifests remain
0.3.1. Five unused extracted fixture copies are deleted. All 39 filesystem tests,
strict Clippy, Rust 1.88 compilation and strict Rustdoc pass on Linux, as do the
selected process/tools tests after cleanup. Snapshot, pins and lock selections
are unchanged. Native macOS and full CI/release qualification remain separate.

The maintainer explicitly approved replacements in Canic and Toko Miner.
Canic now reuses published 0.3.1 gzip/hash/digest mechanics; 32 focused native
checks pass. Its source was separately committed by the maintainer during review,
not by this agent. Toko's direct bounded reader uses filesystem ownership and its
native User Hub tests compile. Offline lock sync removes only unreachable direct
0.3.1 tools/process selections. Query's prepared resolver deletion remains a
handoff patch until shared publication; five existing alias/export tests pass in
a private reduced-error fixture, not its full feature graph. Existing owning
GitHub issues record the applied fixes and remaining published graph convergence.
No release, publication, push or deployment was performed in this batch.

The maintainer authorized implementing Host issues
[#3](https://github.com/dragginzgame/ic-host-tooling/issues/3),
[#4](https://github.com/dragginzgame/ic-host-tooling/issues/4) and
[#6](https://github.com/dragginzgame/ic-host-tooling/issues/6). The response-only
tools profile excludes artifact/filesystem/process production dependencies when
default features are disabled; default Candid extraction retains its API and
required owners. The new bounded artifact HashingWriter hashes only successful
accepted writes and returns sink/prefix identity. Producer success, encoding,
flush/sync, descriptor admission and publication remain caller-owned. No JSON
production dependency or fifth crate was added.

The 58-file Shared Tooling snapshot was exported from a clean private checkout
of committed 25e7ce8, excluding the sibling's dirty pending 0.1.15 bytes. It adds
the workspace rule, explicit governance membership, delegated installer and Make
execution guard. Snapshot verification is independent of inspected helpers;
release pushes recheck their destination and dispatch through its captured URL.
The existing four crate locations conform, including inherited metadata and
dependencies. The committed rule permits application-owned apps/ trees without
requiring this repository to move packages. Upstream
[CI for that exact source](https://github.com/dragginzgame/shared-tooling/actions/runs/37586649650)
passed. Pending upstream LOC reporting and the IcyDB layout exception were
reviewed separately as dirty source and were not adopted or executed; cloc is
not installed locally.

Focused offline Linux checks passed: 52 artifact unit and seven consumer-contract
tests, 27 minimal artifact tests, 19 default tools tests and ten response-only
tests with applicable examples. An independent response-only consumer compiled
and ran with no extraction owners in its normal graph. Strict Clippy, Rustdoc and
Rust 1.88 checks passed for the changed packages, including the response-only
MSRV configuration. The dependency-graph guard, formatting, pins, local links,
exact snapshot verification, scoped consumer formatting-hook preservation,
snapshot corruption fixtures, host installer fixtures and release adapter/runner
substitutes passed. Release/installer effects were substituted; no real Git
mutation, upload or tool download occurred. Logs remain under
/tmp/ic-host-next-*.log. Initial compile/style/format failures remain separately
recorded rather than being relabeled as passing attempts.

Query's proposed response profile and private DigestingWriter replacement are
retained at /tmp/ic-query-host-next-candidate/consumer.patch with before/candidate
source digests. During review Query committed that inspected source as clean
0.47.7 at 73dd32bcbe7cdada61dec1cfcd966b252564e48a; all three before-file digests
still match and the patch applies there. It has not been applied or compiled in
Query's full graph. Query retains schema, canonical encoding, descriptor checks
and capability-confined publication. The consumer follow-up is
[Query #14](https://github.com/dragginzgame/ic-query/issues/14). Automatic review
rejected posting the full patch/source digests; the accepted issue contains only
already-public API proposals and Host issue links. The patch remains local.
Actual consumer archive/response tests and native macOS qualification must
follow reviewed publication. These compatible
additions extend pending 0.3.2; manifests and Cargo.lock remain 0.3.1, with tool
pins unchanged. Prior path fixes, audit evidence and fixture deletions are
preserved. Full CI/release gates and consumer native macOS were not run locally.
No commit, tag, push, publication or new sibling file edit was performed.
