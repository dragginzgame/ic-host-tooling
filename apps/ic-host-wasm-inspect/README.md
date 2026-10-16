# IC Host Wasm inspector

Inspect an explicit raw Wasm file without executing or transforming it:

```sh
ic-host-wasm-inspect input.wasm 1048576 1000 1000 1000
```

The limits are module bytes, sections, exports and custom sections. The CLI uses
canonical artifact/filesystem owners and prints a twelve-column TSV report
including SHA-256 only after admission. Invalid arguments/files/budgets return nonzero without a
report; output failures retain their I/O cause. It estimates no cycles or
instructions. Paths use native OS strings, including non-UTF-8 input names.

Append `--expect-sha256 SHA256` to verify an exact 64-digit lowercase digest
before any report is written. A mismatch fails with no report. Append `--details`
to include export names/kinds/indices and custom-section names/payload sizes after
the TSV summary. These options can be used separately or together, in either
order, after the four limits. Repeated or unknown options are rejected.
Names in detail records use quoted Rust debug escaping; custom payload bytes are
never printed. Input and collection limits still apply. The default two-line,
twelve-column TSV report is unchanged.

Consumers can select the published package/version/binary through the
standard Shared Tooling Cargo installer. Checks remain offline and never install
it. Consumers own budgets, interpretation and performance evidence.
