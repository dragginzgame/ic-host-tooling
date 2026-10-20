# Current handoff

## Source and pending work

Host 0.31.2 is released on main at
`7f3a147b0d5c1a8b136f1e9f53aec1526fb79a93`; package versions remain 0.31.2.
[Cargo.toml](../../Cargo.toml), [release notes](../../CHANGELOG.md) and Git tags own
release state. Pending [0.32.0 notes](../changelog/0.32.md) combine Platform 0.5's
public Principal dependency identity cut with Shared gate/heading support.
Registry callers must align their Platform selection to 0.5. The real index,
HEAD, versions and release effects remain user-owned. Earlier evidence is retained at
`7f3a147:docs/status/current.md`.

## Ownership boundary

The [five-crate contract](../extraction.md) remains: Artifacts owns byte/Wasm
mechanics; FS owns filesystem mechanics; Process owns bounded execution; Text
owns generic decoding/rendering; Tools owns IC interpretation. Platform owns
external IC dependencies and Build owns native extraction/release rendering.
Consumers retain schemas, budgets, targets and effect recovery.
Released Host selects Platform 0.4/defaults-off. The pending Platform 0.5 selection
is now qualified for Host's selected capabilities without defaults or runtime
activation. HTTP extraction remains an unresolved ownership choice
in [#117](https://github.com/dragginzgame/ic-host-tooling/issues/117), since Query's
early refusal and Blob's partial-evidence retention have different contracts.

## IC Platform adoption

The registry index and matching archive confirm latest published Platform 0.5.0
from `b840423`; unpublished sibling 0.5.1 preparation was not consumed or edited.
The already-pending root manifest and lock selection are preserved. Host's Registry
API exposes Platform-owned Principal values, requiring a 0.32 minor release for
cross-crate Rust identity alignment; wire/storage bytes and external DTOs are unchanged.
Host does not use the changed runtime or service-generation APIs.

Actual Rust 1.88 Tools all-feature tests and independent default/certificate/
interface/combined profiles pass. The initial sandbox HTTP socket refusal is
retained; the native-permission rerun passes all 52 all-feature cases. Focused
development Clippy/Rustdoc and Linux/both macOS target graph checks pass; native
macOS execution remains for CI. Evidence: `/tmp/host-platform-050/`.

## Shared Tooling adoption

The snapshot now selects published Shared 0.25.8 at
`0a2d2ee3b5e0ae77755c2915ac28836d4d5ba359`, recorded in [AGENTS.md](../../AGENTS.md)
and [.shared-tooling.snapshot](../../.shared-tooling.snapshot).
All 125 selected files verify. Host now calls its existing complete lint command
through `clippy-workspace` before expensive CI/release stages. Both publication
modes call that gate after source admission and before packaging/registry dispatch.
The finalizer accepts numbered `- Unreleased` headings and uses the saved release
date; Host's consumer fixture qualifies both pending spellings and retained history.
Tool catalogs and executable build-helper identity are unchanged, so no reinstall
is required. Host retains its release adapter rather than selecting Shared's
standalone Cargo-metadata checker.

Focused parallel-Make ordering, publication refusal and the actual release-adapter
transaction fixtures pass with substituted lint/upload/Git effects. Both pending
heading spellings finalize correctly. Snapshot, pins, links and source-following
ShellCheck pass. No full local workspace gate, real workspace Clippy or native
macOS qualification is claimed by these substitutes. No executable preparation,
release or publication was run.
Current evidence is retained at `/tmp/host-shared-0258/`; earlier snapshot evidence
remains in `/tmp/host-shared-0256/`.

## Delivered repairs and consumer evidence

Host 0.31.2 [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/39409280675)
passes Linux, macOS 15 Intel, macOS 15 Apple Silicon and MSRV. It includes the
restored independent FS profiles and corrected 0.28.1 publication guidance.
The registry-only Rust 1.88 probe retains scoped-file lifecycle, encoding/full
validation and response-decoding evidence in `/tmp/host-followups.oBBZP6/`.
Producer issues and superseded proposal PRs are reconciled against delivered main;
remaining Query acceptance belongs to [Query #87](https://github.com/dragginzgame/ic-query/issues/87).
Testkit source `96c01242` has native fixture/PocketIC query/update and MSRV passes
on all three hosts in its [CI](https://github.com/dragginzgame/ic-testkit/actions/runs/39401962713).

[Auth PR #80](https://github.com/dragginzgame/ic-auth/pull/80) is merged and Auth #78
is closed. Its exact PR source passed Linux and both macOS hosts, including the
published file CLI, input seals and consumer recovery fixtures. Contract generation,
Build lock admission and Auth recovery policy remain there. Normal sibling
checkouts were read-only; the contribution used an isolated clone.

[Shared PR #290](https://github.com/dragginzgame/shared-tooling/pull/290) remains
separate from this adopted baseline. Its latest observed source `1be3cf9` passes
Linux and Apple Silicon; Intel macOS and owner integration remain outstanding.
Preserve its isolated checkout and evidence in `/tmp/host-followups.oBBZP6/`.
Follow-up failure logs and earlier closure records remain in
`/tmp/host-audit-20261020/`; do not replay unchanged checks or erase these artifacts.
