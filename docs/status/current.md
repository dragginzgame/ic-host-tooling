# Current handoff

## Local cleanup after 0.63.2

The 0.63.3 draft narrows six single-feature Host Tools targets to their own
tests while retaining isolated all-target compilation. Default/combined suites
remain complete. The workspace publication adapter retains its failure/recovery
checks; the separate generic Shared publication-runner replay is retired here.
All six focused targets (30 cases), publication and Make fixtures, shell lint,
snapshot integrity and documentation checks pass on Linux. New native macOS
qualification remains asynchronous. Evidence: `/tmp/ic-tooling-cleanup-0633`.
These edits remain local; incoming Cargo.lock and handoff changes are preserved.
The second pass replaces Build's duplicate extractor/CLI fixture cleanup with
Host FS's tested guard. Eight extractor cases, five actual CLI cases and strict
all-target Build Clippy pass on Linux. Private lockfile field helpers remain
shared by archive admission and Wasm cache projection. Second-pass evidence:
`/tmp/ic-tooling-cleanup-pass2`.
The third pass derives the private benchmark worker's Clap schema and mode
admission. Help output is byte-identical; nine worker/driver argument cases pass
on development Rust and actual Rust 1.91, with strict all-target Clippy. Evidence:
`/tmp/ic-tooling-cleanup-pass3`. Incoming lock selections remain unchanged.
Testkit's server asset/runner and benchmark-driver fixtures now use the same
guard; the single-caller integration directory helper is removed. All 34 affected
cases, strict Clippy, Rust 1.91 compilation and native/Wasm graph checks pass on
Linux. Installed-server cases remain explicit. Evidence:
`/tmp/ic-tooling-cleanup-pass4`; native macOS qualification remains outstanding.
The dependency/production audit found no unused direct dependencies with
cargo-machete. Eighteen redundant Testkit lint entries were removed after
confirming their enabled group membership on both compilers; strict Clippy
passes. [Issue #231](https://github.com/dragginzgame/ic-tooling/issues/231) now
has a local private capture composition that returns the durable copy's digest
to Wasm evidence export. The second byte traversal is removed; staged-file
readability admission and freshness/retention ordering remain. All 20 affected
cases pass on development Rust and Rust 1.91, with strict Clippy/Rustdoc. No
timing claim or native macOS qualification is made. Evidence: `/tmp/ic-tooling-231`.
Evidence: `/tmp/ic-tooling-cleanup-audit5`.

## Integrated workspace

Delivered main is release 0.63.2, `e5b4095150f49069fc51c3b497e192a8f8eda016`.
The maintainer committed the Wire 0.5.11 lock update and completed publication;
the earlier lockfile choice and release-preparation blockers are superseded.
Host, Build and Testkit were consolidated through
[PR #212](https://github.com/dragginzgame/ic-tooling/pull/212), retaining crate
boundaries and imported histories. Ten packages are public and two are private.
Rust edition 2024 and MSRV 1.91 remain approved. See the
[consolidation design](../design/tooling-consolidation.md) and
[0.63 migration notes](../changelog/0.63.md).

Delivered 0.63 source removes Testkit-owned Candid and Metrics coupling,
uses Platform 0.6 and makes Build/Testkit binaries opt in through `cli`.
Native PocketIC still brings upstream Candid; Testkit's Wasm graph excludes it.
The optional HTTP crate and native CI setup regression are committed at local HEAD.
Publication of the observer and HTTP packages is verified below; consumer
adoption and complete native qualification remain distinct.

## Published interfaces and consumer ownership

[#117](https://github.com/dragginzgame/ic-tooling/issues/117): the HTTP reader now
also accepts caller-owned storage so an outer timeout preserves accepted bytes.
Returned errors move retained evidence into the existing error; cancellation
leaves the prefix for the caller to retain or discard. Status, requests, timeout
and redaction remain caller-owned. Query and Blob adoption have their own issues:
[Query #93](https://github.com/dragginzgame/ic-query/issues/93) and
[Blob #96](https://github.com/dragginzgame/ic-blob-storage/issues/96).
Blob's streaming provider download must remain a verifier/file stream.

[#227](https://github.com/dragginzgame/ic-tooling/issues/227): published opt-in
`ic-testkit-native/stable-memory` adapts preserved Testkit PR #97 source
`3cd17999a53689a035dc86b6f01155eca78c7140` through Platform. The already-selected
registry Platform 0.6.2 supplies Memory and VectorMemory; the manifest floor now
requires that supplying version. Default native and all Wasm graphs remain
independent of the substrate. Clones share backing/counters, resets preserve
bytes, and overflow is sticky and must refuse exact reports. Metrics stays out
of Tooling. Adoption belongs to
[Persistence #98](https://github.com/dragginzgame/ic-persistence/issues/98) and
[Jobs #56](https://github.com/dragginzgame/ic-jobs/issues/56).
Metrics #82 was closed as not planned and is not an acceptance requirement.

Both non-yanked 0.63.2 archives were downloaded and authenticated against the
canonical registry index; their Rust sources match the release checkout.
Actual Rust 1.91 execution from the published archives passes six native
observer/allocation tests, the observer Wasm check and eight HTTP loopback tests.
Evidence and archive hashes are recorded in #227/#117 and retained under
`/tmp/ic-tooling-registry-0632`. Earlier focused Clippy, Rustdoc and dependency
graph evidence remains valid for the matching source.

## Existing contributions and qualification

Local Shared snapshot is 0.26.2 at `1959ef75d0b86ed7725d41ae5056303d35eb633d`,
exported from the exact qualified upstream commit. The optimizer-only routing
from [PR #226](https://github.com/dragginzgame/ic-tooling/pull/226) is incorporated
locally without duplicating its already-present native setup regression.
Upstream passed native Linux, Intel and Apple Silicon matrices; these do not
qualify this combined local tree. Refresh evidence and
incoming dirty-file backups are under `/tmp/ic-tooling-refresh-0262`.
[PR #229](https://github.com/dragginzgame/ic-tooling/pull/229) merged the ICP
decoder at `5b41e182`. Its original native matrix passed. PR #226 merged at
`3e61a613` with retained release notes. The maintainer explicitly authorized
merging while integration tests run asynchronously; fresh native qualification
on `41624dcd` was still running at merge, with all other checks passing.
The combined source passes focused decoder
tests on development Rust and Rust 1.91, strict Clippy, feature graphs and Make
routing checks. These focused Linux checks do not qualify the release
on native macOS. Integration evidence is under
`/tmp/ic-tooling-pr-integration`.

For 0.63.2, browser, optimizer and Testkit runtime matrices passed all three
hosts. [Main CI](https://github.com/dragginzgame/ic-tooling/actions/runs/39588528910)
passed MSRV and lint; native Linux and Apple Silicon were still running at this
handoff. Intel failed before product tests on DNS resolution of
`static.rust-lang.org` while installing the Wasm standard library.
[Issue #230](https://github.com/dragginzgame/ic-tooling/issues/230) owns recovery;
GitHub refused a targeted rerun while the containing workflow was active.
Full original logs are retained at `/tmp/ic-tooling-0632-intel.log`.

Ordinary integration CI runs asynchronously under the maintainer's explicit
cadence instruction. [Shared #298](https://github.com/dragginzgame/shared-tooling/issues/298)
tracks the canonical wording; its implementation was still undelivered on the
default branch at this review, so the recorded snapshot remains unchanged.
Required merge/release/issue acceptance gates remain separate. There are no
open Tooling PRs at this review. [Issue #225](https://github.com/dragginzgame/ic-tooling/issues/225)
still depends on Canic #613 supplying the concrete Wire schema/artifact contract.

## Preserved evidence

Original HTTP evidence: `/tmp/ic-tooling-http`; native setup regression:
`/tmp/ic-tooling-0632`. Earlier implementation evidence remains under
`/tmp/ic-tooling-adoption`, `/tmp/ic-tooling-cleanup`, `/tmp/ic-tooling-candid`,
`/tmp/ic-tooling-platform-06`, `/tmp/ic-tooling-no-metrics` and `/tmp/ic-tooling-cli`.
Migration backups, full checkout archives and retirement evidence remain under
`/tmp/ic-tooling-migration`, including `retirement/`. Original proposal objects
remain readable in the ordinary former Host checkout; siblings are read-only.
The verified stale release lock was moved to
`/tmp/ic-tooling-stale-release-lock.qHQFTn/lock`; saved plans were preserved.
