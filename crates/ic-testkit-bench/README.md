# IC Testkit benchmark programs

Private workspace package for the fixture-reuse driver, its separate workload
process and live benchmark-marker qualification. It is not published.

```sh
cargo fetch --locked
cargo run -p ic-testkit-bench --locked --offline --bin fixture_reuse_benchmark_driver -- \
  --server /absolute/path/to/pocket-ic --profile dev \
  --iterations 2 --workers 1 --repeats 1 --capacities 1 --state-bytes 64
cargo test -p ic-testkit-bench --lib --locked
```

Prepare the selected PocketIC binary and Rust Wasm target through the normal
[developer setup](../../README.md#toolchains-and-checks) first. Full workload, cancellation,
measurement and retained-report contracts are in the
[benchmark guide](../../docs/fixture-reuse-benchmark.md). `make test` includes this
package; `make test-canisters` selects its live marker qualification. Both native
binaries share one protocol/report implementation while retaining independent
process lifetimes. Builds use IC Build's typed Cargo selections, exact reported
executable and retained Wasm ownership; build duration has no default ceiling.
