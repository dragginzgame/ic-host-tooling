<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# Test Canisters

Small canisters used by `ic-testkit` to test its own PocketIC harness behavior.

The fixture package lives under `crates/ic_testkit_perf_probe` in the root
workspace. It is a test canister, not an application example.

## Current Fixtures

- `ic_testkit_perf_probe`: emits compact `ICTK|...` benchmark markers via
  `ic_testkit::performance::Performance::measure`. It exposes `ping`,
  `benchmark_once`, and `benchmark_start_then_trap` to exercise successful
  spans and unmatched start markers after a trap.

Install the Wasm target for the repository toolchain first:

```sh
rustup target add wasm32-unknown-unknown
```

For optional inspection of the probe artifact, build it with:

```sh
make build-test-canisters
```

This builds `ic_testkit_perf_probe.wasm` under
`target/pic-wasm/wasm32-unknown-unknown/debug/`.

Run only the live marker fixture test with:

```sh
cargo test -p ic-testkit-dev --locked --test canister_benchmark \
  perf_probe_canister_emits_parseable_benchmark_markers -- --exact --nocapture
```

The test uses IC Build to retain its own exact Wasm in a temporary target
directory; it does not consume the standalone artifact above. Both builds may
reuse Cargo's incremental state under `target/pic-wasm`.
Startup requires an explicit prepared `POCKET_IC_BIN` or caller-owned
`IC_TESTKIT_POCKET_IC_URL`; it does not download a server. Prepare and admit the
reviewed binary through `make install-server` and
`target/debug/ic-testkit-server check` before direct live-test invocations.

Run the live marker qualification through the repository's prepared-server
check with:

```sh
make test-canisters
```

The main CI test command includes this integration target. Generic build/cache
mechanics are qualified by IC Build; this consumer test verifies its retained
Wasm together with Testkit's actual marker emission, parsing and trap handling.
The standalone fixture build is a manual command, not a prerequisite for testing.
