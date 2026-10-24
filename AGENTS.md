# IC Tooling Agent Rules

Read docs/status/current.md first, then [DRAGGINZGAME.md](DRAGGINZGAME.md).
The baseline is Shared Tooling 0.26.10 revision a8141e775983c7fe504c4dd911e978152e657fdf,
recorded with exact file digests in [.shared-tooling.snapshot](.shared-tooling.snapshot).

- Mutate only this repository. Existing sibling repositories remain read-only.
- Follow [the contribution rules](rules/contributions.md). Ordinary fixes remain
  local; an explicit commit or PR request authorizes its scoped Git workflow.
  Merges, direct integration-branch pushes, releases, publication, tool downloads
  and deployment retain their own target/effect authorization.
- Keep the Host, Build and Testkit crate boundaries in [the extraction contract](docs/extraction.md).
  Do not add compatibility reexports for moved APIs or duplicate production owners.
- Consumers own product-tool pins, byte budgets, credentials, targets, retries, schemas,
  installation policy and paid-operation recovery. Libraries never claim lifecycle ownership.
- Rust edition is 2024, MSRV is 1.91.0. Use ordinary directory modules and
  inherited workspace versions/dependencies. Do not add Python tooling.
- Preserve dirty source and artifacts. Check for active Cargo/rustc processes
  using this repository's target directory before editing or compiling.
- Follow [the shared validation policy](DRAGGINZGAME.md#validation-and-evidence)
  using the local commands in [the host guide](docs/hosts.md).
  Release commands retain explicit release authorization; never clean consumer artifacts.
- Linux x86-64 and macOS 15 on Intel and Apple Silicon are required hosts.
  Report native macOS qualification separately from Linux evidence.
- Follow shared changelog rules. GitHub issues are the sole follow-up tracker;
  source provenance, design and handoff documents are evidence, not issue queues.
