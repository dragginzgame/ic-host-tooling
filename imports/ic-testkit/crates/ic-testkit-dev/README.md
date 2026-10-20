# IC Testkit development programs

Private workspace package for the fixture-reuse driver, its separate workload
process, live benchmark-marker qualification and coordinated archive adapter. It is not published.

```sh
cargo fetch --locked
cargo run -p ic-testkit-dev --locked --offline --bin fixture_reuse_benchmark_driver -- \
  --server /absolute/path/to/pocket-ic --profile dev \
  --iterations 2 --workers 1 --repeats 1 --capacities 1 --state-bytes 64
cargo test -p ic-testkit-dev --lib --locked
```

Prepare the selected PocketIC binary and Rust Wasm target through the normal
[developer setup](../../README.md#toolchains-and-checks) first. Full workload, cancellation,
measurement and retained-report contracts are in the
[benchmark guide](../../docs/fixture-reuse-benchmark.md). `make test` includes this
package; `make test-canisters` selects its live marker qualification. Both native
binaries share one protocol/report implementation while retaining independent
process lifetimes. Builds use IC Build's typed Cargo selections, exact reported
executable and retained Wasm ownership; build duration has no default ceiling.

`make package` delegates exact coordinated archive qualification to IC Build.
It retains per-attempt evidence and a separate verification target under `target/`,
checks the extracted graph with the selected external dependencies, and runs the
archive tests and doctests offline. Prepare the selected cache and PocketIC binary
first; no tools or sources are installed by qualification. `make publish-dry-run`
uses normal Cargo registry admission and never the fixture-only path patches.
