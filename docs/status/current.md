# Current handoff

## IC Tooling consolidation

Work is on isolated branch migration/tooling-workspace at
/tmp/ic-tooling-migration/integration. The normal checkout and its crc32fast
lockfile update remain untouched. GitHub is renamed to dragginzgame/ic-tooling;
normal origin was updated. Source imports preserve Host c3f6f605, Build c84c64e4
and Testkit 29399a6e ancestry. See [the inventory](../design/tooling-consolidation.md).

The combined workspace selects 0.62.0 and Rust 1.91. Host's five crates,
Build and the two published Testkit crates remain separate owners; the Wasm
inspection app is also published. Testkit driver/probe packages remain private.
One Shared 0.25.8 snapshot, catalog and lockfile replace separate infrastructure.
Import staging directories have been retired; history retains their exact trees.
No source sibling checkout has been deleted, and no publication has occurred.

## Evidence

Evidence is retained under /tmp/ic-tooling-migration. Native compilation of
the moved packages passes on development Rust and actual Rust 1.91. Testkit's
Wasm MSRV check, package-selected native/Wasm graph exclusions, strict moved-
package Clippy, 290 focused library tests and browser-session tests pass.
All nine archives were qualified together through Build's existing engine,
including their enabled tests and doc tests against admitted PocketIC 16.1.0.
The initial missing-server and source-layout-only failures remain retained.
Publication refusal fixtures, dependency roles/boundaries, snapshot and moved
document links pass. Remaining final checks must be reported from their logs;
no native macOS execution or hosted CI for this branch is claimed.

## Preservation and retirement

Verified Git bundles preserve both original repositories, including Build's
old pre-sync stash. Remote branch heads and outstanding PRs are inventoried.
Testkit's prepared PocketIC bundle is also admitted in this checkout. The
maintainer authorized deleting the old local directories after completion;
do not delete them before integration and retained-artifact custody are settled.
Remote repository archival, release and consumer migration remain separate.

Final local checks also pass: release-adapter transactions, publication refusals,
Make setup ordering and the quoted PocketIC environment handoff, actual installed
server-runner execution, all 97 enabled Testkit all-feature library tests,
Rust 1.91 native/Wasm compilation, strict Clippy/Rustdoc, formatting, shell/workflow
lint and final feature graphs. The combined archives pass through the corrected
Make entrypoint against this checkout's own admitted server. Native macOS and
integration into main remain outstanding; sibling directories remain intact.
The three remote default branches were rechecked and still match the import
revisions. No remote repository was archived or source PR closed.

Final CI review preserves Build’s Linux lint/security job and adds combined
archive qualification to offline native CI. Publication dry-run verification
runs separately with registry access in CI: the local offline attempt passed
workspace Clippy, then refused an HTTP request. No upload was attempted.
Original source refs, including Build’s stash, also survive under
`refs/migration-archive/` in the original Host checkout.
