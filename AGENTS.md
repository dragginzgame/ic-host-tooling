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

## Validation cadence during shared-policy adoption

Shared Tooling owns the [validation policy](DRAGGINZGAME.md#validation-and-evidence);
local guidance supplies commands and product qualification requirements only.
Maintainer-approved exception (2026-10-14): until this snapshot adopts Shared
Tooling's focused-handoff policy, replace its full-suite-before-handoff requirement
with focused checks of changed behavior and directly affected callers, including
relevant failure/recovery paths. Reuse unchanged passing evidence. Documentation
changes need link, consistency and diff checks only. Run local full suites only
for an explicitly authorized release or an explicit request for those checks;
implementation, continuation, PR preparation and readiness requests do not qualify.
Report tested scope; an unrun full suite alone is not a handoff blocker. Configured
CI, required branch checks, release gates and separate effect authority remain
unchanged. This exception avoids redundant local testing; it does not claim a
snapshot refresh. On reviewed adoption, remove this exception and retain only the
link to the canonical policy and repository-specific commands.
