# Parent publication review

Scope: post-0.10.0 local-file boundary review under
[code hygiene](../../../../../../../../audits/code-hygiene.md), with the unchanged
Shared baseline `b2646cde9abbc8861857a4379c683a0c19eba43e` and root AGENTS.md
overlay. Source started clean at released Host
`98562bea26a98993d93b80ed908bea4876c32a91`; the user authorized continued repairs.
Inspection covers pathname/descriptor publication, lock admission and regular,
optional and private reads. This is not a crash simulation or whole-repository
security audit. Earlier structural verdicts are not reused as runtime proof.

## Finding and repair

**MEDIUM, confirmed:** `durable::supported::create_parent_hierarchy` skips both
directory sync barriers when a competing creator wins an initially missing
directory. That creator may stop before either sync. A successful output sync
does not establish the new parent directory's own link in its parent.
Owner and follow-up: [Host #43](https://github.com/dragginzgame/ic-host-tooling/issues/43).

Two deterministic hook-driven regressions fail with the released implementation:
no directory/parent sync is observed before staging, and injected sync failures
are bypassed. The repair converges successful creation and an admitted
AlreadyExists directory on the existing `sync_created_directory` routine.
Non-directory rejection and no-follow opening remain intact. No function,
method or type was removed; no new production abstraction or public API was added.

Intentional retention: pathname admission remains distinct from descriptor-only
publication; read prechecks and descriptor admission have separate race/error
roles; optional reads retain their initial-absence contract; existing lock opens
still avoid staging and synchronization. Consumer namespace custody, concurrent
writes and paid-operation recovery remain outside this library's ownership.

## Evidence and limits

Linux x86-64, Rust 1.99.0, locked cached graph, default filesystem package features:
`cargo test --locked --offline -p ic-host-fs --lib durable::` passes all 48 selected
tests after repair. Strict all-target package Clippy and Rust 1.88.0 library check
pass. Before/after logs remain in `/tmp/ic-host-0101/`; the two before-fix tests fail
for the expected missing barriers. This proves execution ordering and typed
error propagation, not hardware-independent crash persistence.

Release 0.10.0 is on remote main; its
[CI run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37961457616)
has passing Linux/MSRV jobs and queued macOS 15 Intel/Apple Silicon jobs at
inspection. It does not qualify these uncommitted changes. Shared's remote main
matches the adopted baseline; newer sibling changes are dirty and were not copied.
The six existing 0.10 consumer migration issues remain open. No sibling edits,
consumer compilation, full gate, release, publication, commit or push occurred.

Disposition: the bounded Linux repair is verified, with native macOS qualification
and delivery remaining under the owning issue. Pending notes select compatible
0.10.1; manifests and lockfile stay at 0.10.0.
