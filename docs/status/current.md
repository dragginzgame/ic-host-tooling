# Current handoff

## Authoritative state

Host 0.30.1 is released at `391a2dfe63c994473fc68135d62900b9a38861a2`.
[Cargo.toml](../../Cargo.toml), [CHANGELOG.md](../../CHANGELOG.md), Git tags and
[AGENTS.md](../../AGENTS.md) own release state and local rules. The next draft is
[0.31.0](../changelog/0.31.md#0310), with YAML and Platform-owned principals.
Package versions remain 0.30.1. Staging, commits, version changes, release/push
and registry publication remain user-owned. Sibling checkouts are read-only;
GitHub issues are the sole follow-up tracker.

## Ownership and pending work

The five crates retain the [extraction contract](../extraction.md): Artifacts owns
byte/Wasm mechanics; FS owns filesystem mechanics; Process owns bounded native
execution; Text owns generic text/decoding; Tools owns IC-specific interpretation.
Platform owns upstream IC interoperability; Build owns native extraction/release
rendering. Callers retain schemas, byte budgets, targets and effect recovery.

Preserve all pending YAML source, manifests, graph checks, documentation and
Cargo.lock changes. Preserve unrelated incoming lock selections, including
webpki-root-certs 1.0.10, and all previous source/build/evidence artifacts. No new
API hard cut was justified by the Shared adoption review. The distinct filesystem
custody adapters and Platform aggregate-reply adapter preserve supported contracts.

## Optional YAML boundary

Text owns `yaml::from_slice` through independent, default-off `yaml` and private
serde_yaml 0.9.34+deprecated, matching ICP CLI main
`0ea5fc5f0198b01f0d8491182647e62503adf6a5`'s `0.9.34` requirement. The maintainer
selected package alignment, retaining unsafe-libyaml's maintenance limitation.

Byte admission precedes one private-tree parse. Duplicate keys and default
non-core-tag/`<<` refusals cover ignored content and quoted `<<` keys before owned
schema projection. Callers retain schemas, redaction, IO and configuration policy.
Host exposes no configurable depth/event/alias budgets or memory/CPU ceiling;
backend checks remain. No external includes or property expansion run.

Tools retains actual-map/key/name admission and effective ICP meaning, delegating
decoding to Text. Its original byte-limit argument is preserved; its upstream
error payload becomes Host-owned. [Migration](../changelog/0.31.md) describes the
breaking error/duplicate-key contract. Rust 1.88 remains the floor for all features;
the single MSRV lane is retained. Canic's raw-manifest schema and public errors
remain local; adoption can follow publication and focused consumer verification.

## Platform 0.3.0

Manifest and lock select published 0.3.0 with defaults disabled. Registry shard
APIs now use Platform-owned principals; SDK/Candid DTOs retain external principals
with explicit conversions. The Agent fixture converts at its SDK call; Registry
tests preserve golden identity bytes and round trips. See the [migration](../changelog/0.31.md).
Registry's public type identity changes; byte formats and admission are unchanged.
Receipt: `/tmp/host-platform-030.yodwBG/`. The live index/archive/lock checksums
agree; released source is `390644fe75fc0421efa98dc0319db34101639e64`.
Rust 1.88 all-feature tests (52), independent profiles, strict MSRV/development
Clippy, Rustdoc and Linux/both macOS graphs pass. Incoming selections are preserved.

## Shared Tooling 0.25.3

The [snapshot](../../.shared-tooling.snapshot) admits 125 files from pushed source
`c55c1065c0061455dab7f0e2be8ddd04fdeb9910`, VERSION 0.25.3. Tool pins are unchanged.
Selected Cargo tools accept explicit features on setup/check, binding canonical
sets to immutable installations and Cargo receipts. Defaults stay enabled.
Featureless Host formatters/editor keep their selections; the helper fingerprint
requires explicit setup again. Old generations stay retained; checks never repair.

The adopted Wire endpoint and imported runtime/crypto rules do not relocate Host's
external-protocol helpers. Host owns no canister endpoints or IC runtime. The
handoff is now a current-state summary rather than accumulated release history.
Historical completed work remains in the committed handoff at `391a2df` and the
existing [archive](archive-2026-10-18.md); do not append old narratives here.

## Validation and next action

YAML receipt: `/tmp/host-yaml-ic.l0TcwB/`. Actual Rust 1.88 passed Text's 40 and
Tools' 52 all-feature unit cases, including the local HTTP case with loopback
permission, plus focused/default profiles. Strict MSRV/development Clippy and
Rustdoc passed. Linux and both macOS graph matrices passed, including backend
ownership and default/unrelated-profile exclusion. Formatting, pins, snapshot,
links, ShellCheck and Make routing passed. Saphyr-only packages were removed;
serde_yaml, Ryu and unsafe-libyaml replaced them without changing other selections.

Shared adoption receipt: `/tmp/host-shared-0253.dNBj5e/`. It retains the complete
incoming source, preview/refresh and focused check logs. Unchanged formatter/editor
selections were explicitly prepared offline. A competing install lock cleared
without intervention; the refused attempt is retained. Snapshot, pins, links,
symlinks, tools, formatting/hooks, ShellCheck, explicit-feature/path-context fixtures
and Host Make routing pass. Release-adapter fixtures pass with real offline
rendering and substituted setup, qualification and Git effects.
The exact-source [hosted run](https://github.com/dragginzgame/shared-tooling/actions/runs/39387644688)
failed cross-checkout native-tool reuse on Linux and Apple Silicon; Intel is still
running. Detailed diagnostics are in its retained artifacts; root cause remains
unverified. Earlier evidence stays in `/tmp/host-shared-0252.KXQmMF/`.

No full local workspace/release gate, native macOS execution, package version
change, index/HEAD change, release or sibling edit ran in these pending tasks.
Graph checks do not qualify native macOS. Reassess upstream CI before release.
Leave release execution and native host qualification to the maintainer's
authorized workflow. Earlier receipts remain evidence,
not substitute qualification of this pending source.
