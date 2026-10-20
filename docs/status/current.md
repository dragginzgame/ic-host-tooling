# Current handoff

## Authoritative state

Host 0.31.0 is released at `ee03594`; package versions remain 0.31.0.
[Cargo.toml](../../Cargo.toml), [CHANGELOG.md](../../CHANGELOG.md), Git tags and
[AGENTS.md](../../AGENTS.md) own release state and local rules. The next draft is
[0.31.1](../changelog/0.31.md#0311) for compatible Platform 0.4 adoption.
Staging, commits, package versions, release/push and registry publication remain
user-owned. Sibling checkouts are read-only; GitHub issues own follow-up tracking.

## Ownership boundary

The five crates retain the [extraction contract](../extraction.md): Artifacts owns
byte/Wasm mechanics; FS owns filesystem mechanics; Process owns bounded native
execution; Text owns generic text/decoding; Tools owns IC-specific interpretation.
Platform owns external IC dependencies; Build owns native extraction/release
rendering. Callers retain schemas, byte budgets, targets and effect recovery.

Released 0.31.0 owns the YAML and Principal migrations. Text's optional YAML uses
serde_yaml 0.9.34+deprecated to align with ICP CLI, retaining its unsafe-libyaml
maintenance limitation and backend checks. Byte admission is not a configurable
parser memory/CPU ceiling. Registry uses Platform-owned principals; SDK/Candid
DTOs retain external principals with explicit conversions. See [migration notes](../changelog/0.31.md#0310).
Completed work and earlier qualification receipts remain in the committed handoff
at `ee03594`; do not restore its superseded pending-state narrative.

## Platform 0.4.0 adoption

The incoming manifest and lock already selected published Platform 0.4.0, defaults
disabled. Preserve that selection and all unrelated dependency/artifact state.
The live registry index and cached archive match the lock checksum; released
source is `26a07c510ab74413ec30779cd69af84db13cff11`.

The release's raw-call, RejectCode and signature ownership cuts are outside Host's
selected capabilities. Host-consumed Agent, certification, parser, decoder,
contract bodies, ledger exports, Principal and Account sources match 0.3.0.
The Candid facade adds `encode_args_ref`; existing Host uses remain compatible.
No Rust implementation change or Host public contract cut is required. The lock
changes only the selected Platform version/checksum from the released base.

Receipt: `/tmp/host-platform-040.1OEwh5/`. It preserves incoming source/index/HEAD,
registry/source review and focused qualification logs. Rust 1.88 passes all 52
Tools tests, including loopback HTTP, and independent default, certificate,
interface/combined, Candid, Agent, ledger and Registry profiles. Strict MSRV and
development Clippy, Rustdoc, metadata, formatting, snapshot, pins and links pass.
Linux and both macOS graph matrices pass. Exact-source Platform hosted checks were
running at source admission; do not claim they passed.

## Shared baseline and qualification limits

The [Shared snapshot](../../.shared-tooling.snapshot) remains 0.25.3 at
`c55c1065c0061455dab7f0e2be8ddd04fdeb9910`, 125 files. Its explicit-feature tool
installer, unchanged pins and existing prepared tools are retained.
The prior adoption receipt `/tmp/host-shared-0253.dNBj5e/` records passing focused
snapshot, installer, formatting/hook, Make and release-adapter checks. At that
review, the [upstream run](https://github.com/dragginzgame/shared-tooling/actions/runs/39387644688)
failed cross-checkout tool reuse on Linux/Apple Silicon; Intel was incomplete.
Diagnostics were retained upstream; root cause was not verified in this checkout.

No full local workspace/release gate, native macOS execution, package-version
change, real index/HEAD mutation, release or sibling edit ran as part of this
adoption. Named-target graphs do not qualify native macOS. Release execution and
native host qualification remain with the maintainer's authorized workflow.
