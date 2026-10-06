# IC Host Tooling Agent Rules

Read docs/status/current.md first, then [DRAGGINZGAME.md](DRAGGINZGAME.md).
The baseline is Shared Tooling revision 21f3ec3dd97f2968c9f0b08924451bb2f71770d1,
recorded with exact file digests in [.shared-tooling.snapshot](.shared-tooling.snapshot).

- Mutate only this repository. Existing sibling repositories remain read-only.
- Never create or amend Git commits. GitHub creation, publication, tags, pushes,
  tool downloads and deployment require their own target/effect authorization.
- Keep the four crate boundaries in [the extraction contract](docs/extraction.md).
  Do not add compatibility reexports for moved APIs or duplicate production owners.
- Consumers own tool pins, byte budgets, credentials, targets, retries, schemas,
  installation policy and paid-operation recovery. Libraries never claim lifecycle ownership.
- Rust edition is 2024, MSRV is 1.88.0. Use ordinary directory modules and
  inherited workspace versions/dependencies. Do not add Python tooling.
- Preserve dirty source and artifacts. Check for active Cargo/rustc processes
  using this repository's target directory before editing or compiling.
- During development, use selected package/module checks only. Full CI/release
  gates require an explicit request or configured CI. Never run a release command
  that creates commits, and never clean consumer artifacts.
- Linux x86-64 and macOS 15 on Intel and Apple Silicon are required hosts.
  Report native macOS qualification separately from Linux evidence.
- Follow shared changelog rules. GitHub issues are the sole follow-up tracker;
  source provenance, design and handoff documents are evidence, not issue queues.
