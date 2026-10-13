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

Consumers can select the published package/version/binary through the
standard Shared Tooling Cargo installer. Checks remain offline and never install
it. Consumers own budgets, interpretation and performance evidence.
