# Host qualification

Required hosts are Linux x86-64 and macOS 15 on Intel and Apple Silicon.
The imported filesystem/process APIs retain their platform gates; unsupported
platforms receive their existing explicit refusal rather than simulated behavior.

Select one changed crate with PACKAGE=<crate> for make check, make clippy,
make docs-check or make test. Artifact checks use its actual enabled gzip,
archive and Wasm features; production filesystem dependencies select none of them.
Check target-directory ownership before compilation. Preserve failure logs and artifacts.

Native CI prepares pinned Rust/cargo-sort and local host tools, then verifies
the shared snapshot, declarations, formatter prerequisites, release adapter,
minimal artifact tests and each selected package with all features in sequence.
The release gate uses the same host tools; installing the IC executable bundle
is separate explicit setup and is not required for library qualification.
CI also owns native macOS execution; Linux qualification is not macOS evidence.
Earlier Shared Tooling host-fixture failures on macOS remain historical evidence
in the original repository. The refreshed fixture has local Linux execution with
substituted host identities here; native macOS qualification remains separate.
