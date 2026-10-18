# Cargo release candidates

IC Build owns format-preserving Cargo manifest and lockfile candidate editing.
Shared Tooling retains release authorization, backups, changelog preparation,
source custody, independent Cargo graph checks and rollback. Rendering never
installs tools, invokes Cargo, resolves dependencies, writes inputs or publishes.

## Explicit command

Prepare and qualify the released `ic-build-release-metadata` executable before
release adapter startup. A consumer selects its exact package/tool identity and
an explicit absolute executable path; rendering performs no implicit installation.
The CLI uses Unix descriptor-based, no-follow regular-file reads and requires an
explicit per-document input byte allowance. Linux and macOS share this contract.
Callers retain trusted ancestors and exclude concurrent writers.

```sh
/absolute/prepared/ic-build-release-metadata --max-input-bytes 8388608 \
  manifest Cargo.toml 0.1.0 0.2.0 workspace-input.json > candidate.manifest
/absolute/prepared/ic-build-release-metadata --max-input-bytes 8388608 \
  lock Cargo.lock 0.1.0 0.2.0 app core > candidate.lock
```

The manifest context is optional. With it, supply the previously admitted
`manifest.workspace.dependencies` JSON projection and `members`, each with a
`name` and absolute `manifest_path`. Dependencies must agree with the complete
source declarations. Local aliases use `package` when present. Without
`source_root`, ownership is checked against canonical live dependency paths;
with an absolute `source_root`, lexical saved identities are used without opening
the old checkout. Duplicate, missing or conflicting identities refuse.

Only owned simple numeric requirements (optional `^`, `~` or `=` and one to three
components) advance at their existing precision. Composite ranges and external
requirements remain caller policy and unchanged. The root version requires
canonical stable `X.Y.Z`; components are compared as strings without integer limits.
TOML owns syntax and literal spans, including quoted/dotted keys. Selected literals
may change quoting; all bytes outside those literal spans remain identical,
including Unicode, CRLF, comments and missing final newlines.

Lock rendering selects exactly one unsourced record per package and rewrites
only its version plus exact unqualified `name previous` dependency strings.
Source-qualified registry/Git references and all other fields remain intact.
The independent semantic checker admits only that projection. The existing
version-only projection API retains its stricter dependency-array contract.

Exit 0 returns a complete candidate on stdout. Refusal returns 2 with diagnostics
on stderr and no candidate. Output transport errors also fail. Filesystem and
stream failures do not authorize retrying a release or upload. Callers admit the
candidate with their real locked Cargo graph before replacing source, and retain
originals and failed attempts according to their own release recovery contract.

## Library boundary and qualification

`cargo::release_metadata` exposes pure manifest and lock renderers. Callers supply
bounded UTF-8 input and exact version/dependency projections; ownership discovery
and process/filesystem effects stay outside these functions.
`cargo::lock_projection::verify_workspace_release_projection` independently
checks version plus unqualified-reference changes. The original
`verify_workspace_version_projection` continues to forbid reference changes.

Focused checks are the `cargo::release_metadata` and `cargo::lock_projection`
library tests and `release_metadata_cli` integration target. These include a real
local-only Cargo graph without network or upload. Producer publication and
Shared Tooling adoption remain separate; a locally compiled candidate is not a
released dependency and must not become a mutable sibling/tool fallback.
