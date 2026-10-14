# IC Host Tooling Agent Rules

Read docs/status/current.md first, then [DRAGGINZGAME.md](DRAGGINZGAME.md).
The baseline is Shared Tooling revision ac68c74439e13f19229a35f424f968c64325cc44,
recorded with exact file digests in [.shared-tooling.snapshot](.shared-tooling.snapshot).

- Mutate only this repository. Existing sibling repositories remain read-only.
- Follow [the contribution rules](rules/contributions.md). Ordinary fixes remain
  local; an explicit commit or PR request authorizes its scoped Git workflow.
  Merges, direct integration-branch pushes, releases, publication, tool downloads
  and deployment retain their own target/effect authorization.
- Keep the four crate boundaries in [the extraction contract](docs/extraction.md).
  Do not add compatibility reexports for moved APIs or duplicate production owners.
- Consumers own product-tool pins, byte budgets, credentials, targets, retries, schemas,
  installation policy and paid-operation recovery. Libraries never claim lifecycle ownership.
- Rust edition is 2024, MSRV is 1.88.0. Use ordinary directory modules and
  inherited workspace versions/dependencies. Do not add Python tooling.
- Preserve dirty source and artifacts. Check for active Cargo/rustc processes
  using this repository's target directory before editing or compiling.
- Use selected package/module checks under the validation cadence below.
  Release commands retain explicit release authorization; never clean consumer artifacts.
- Linux x86-64 and macOS 15 on Intel and Apple Silicon are required hosts.
  Report native macOS qualification separately from Linux evidence.
- Follow shared changelog rules. GitHub issues are the sole follow-up tracker;
  source provenance, design and handoff documents are evidence, not issue queues.

## Validation policy

Apply Shared Tooling's [canonical validation cadence](https://github.com/dragginzgame/shared-tooling/blob/4256171b5b7ec4c4cc4078548c1c1617ee150699/DRAGGINZGAME.md#validation-and-evidence),
adopted by exact revision `4256171b5b7ec4c4cc4078548c1c1617ee150699` (0.12.5).
This revision-bound policy reference supersedes only the validation cadence in
our older snapshot; executable tooling and all other shared rules retain their
recorded snapshot revision. Local guidance supplies commands and product
qualification requirements. Configured CI and release gates remain unchanged.
On snapshot adoption of this policy, replace this separate reference with the
local baseline link; do not keep a duplicate cadence or temporary exception.
