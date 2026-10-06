# Host qualification

Required hosts are Linux x86-64 and macOS 15 on Intel and Apple Silicon.
The imported filesystem/process APIs retain their platform gates; unsupported
platforms receive their existing explicit refusal rather than simulated behavior.

Select one changed crate with PACKAGE=<crate> for make check, make clippy,
make docs-check or make test. Artifact checks use its actual enabled gzip,
archive and Wasm features; production filesystem dependencies select none of them.
Check target-directory ownership before compilation. Preserve failure logs and artifacts.

Native CI prepares pinned Rust/cargo-sort and local host tools, then verifies
the shared snapshot, declarations, formatting and each selected package in sequence.
CI also owns native macOS execution; Linux qualification is not macOS evidence.
The current Shared Tooling host fixture's upstream macOS failures remain recorded
in the original repository's evidence and are not relabeled by this bootstrap.
