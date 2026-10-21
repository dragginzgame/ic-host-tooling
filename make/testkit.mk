# Testkit owns simulator setup and native/Wasm qualification.
REPO_ROOT := $(CURDIR)/
SERVER_CLI := $(CURDIR)/target/debug/ic-testkit-server
.PHONY: build-server-cli install-server server-check testkit-graph-check testkit-wasm-check testkit-msrv testkit-check testkit-browser-check testkit-package-check testkit-installed-check
build-server-cli:
	+cargo build -p ic-testkit --features cli --locked --offline --bin ic-testkit-server
install-server:
	+cargo fetch --locked
	+$(MAKE) --no-print-directory build-server-cli
	"$(SERVER_CLI)" setup
server-check:
	@test -x "$(SERVER_CLI)" || { echo 'Run make install-server during explicit setup.' >&2; exit 1; }
	"$(SERVER_CLI)" check
testkit-wasm-check:
	+cargo check -p ic-testkit -p ic-testkit-native --features ic-testkit/reply-fixtures --locked --offline --target wasm32-unknown-unknown
	+cargo check -p ic-testkit-perf-probe --locked --offline --target wasm32-unknown-unknown
testkit-msrv:
	+cargo +$(MSRV) check -p ic-testkit -p ic-testkit-native --features ic-testkit/reply-fixtures --locked --offline --target wasm32-unknown-unknown
	+cargo +$(MSRV) check -p ic-testkit-perf-probe --locked --offline --target wasm32-unknown-unknown
testkit-browser-check:
	node --test testing/browser/session.test.mjs
testkit-check: build-server-cli
	+@server="$$("$(SERVER_CLI)" check)" && POCKET_IC_BIN="$$server" cargo test -p ic-testkit -p ic-testkit-dev -p ic-testkit-native --all-features --locked --offline
testkit-installed-check: build-server-cli
	+@server="$$("$(SERVER_CLI)" check)" && POCKET_IC_BIN="$$server" cargo test -p ic-testkit --features cli --locked --offline --test server_runner real_server_runs_a_separate_process_using_environment_startup -- --ignored --exact
testkit-package-check: build-server-cli
	+@server="$$("$(SERVER_CLI)" check)" && POCKET_IC_BIN="$$server" cargo run -p ic-testkit-dev --bin check_packages --locked --offline
# Shared owns observation mechanics; these are Testkit's supported profiles.
TESTKIT_CARGO_GRAPH_POLICY = --consumer "$(REPO_ROOT)" --require ic-platform=1 \
	--forbid ic-build --forbid ic-host-text --forbid ic-metrics --forbid ic-agent --forbid candid_parser \
	--forbid-pattern '^ic-cdk(?!-executor$$)'

testkit-graph-check:
	@set -e; host="$$(rustc -vV | sed -n 's/^host: //p')"; test -n "$$host"; \
		perl scripts/ci/check-cargo-graph.pl --consumer "$(REPO_ROOT)" \
			--package ic-testkit-native --target "$$host" --all-features \
			--require ic-testkit-native=1 --forbid-pattern '^(?!ic-testkit-native$$)'; \
		perl scripts/ci/check-cargo-graph.pl $(TESTKIT_CARGO_GRAPH_POLICY) \
			--package ic-testkit --target "$$host" --forbid clap --forbid ic-wire --forbid ic-cdk-executor --require pocket-ic=1; \
		perl scripts/ci/check-cargo-graph.pl $(TESTKIT_CARGO_GRAPH_POLICY) \
			--package ic-testkit --target "$$host" --all-features \
			--require clap=1 --require ic-wire=1 --forbid ic-cdk-executor --require pocket-ic=1; \
		perl scripts/ci/check-cargo-graph.pl --consumer "$(REPO_ROOT)" \
			--package ic-testkit-dev --target "$$host" --all-features \
			--require ic-build=1 --require ic-platform=1 --require pocket-ic=1 \
			--require ic-host-artifacts=1 --require ic-host-fs=1 \
			--require ic-host-process=1 --require ic-host-text=1
	perl scripts/ci/check-cargo-graph.pl --consumer "$(REPO_ROOT)" \
		--package ic-testkit-native --target wasm32-unknown-unknown --all-features \
		--require ic-testkit-native=1 --forbid-pattern '^(?!ic-testkit-native$$)'
	perl scripts/ci/check-cargo-graph.pl $(TESTKIT_CARGO_GRAPH_POLICY) --package ic-testkit \
		--target wasm32-unknown-unknown --all-features --forbid-pattern '^ic-host-' \
		--forbid candid --forbid candid_derive --forbid pocket-ic --forbid ic-wire --forbid clap \
		--forbid ic-platform-macros --forbid ic-cdk-executor --require ic0=1
	perl scripts/ci/check-cargo-graph.pl $(TESTKIT_CARGO_GRAPH_POLICY) --package ic-testkit-perf-probe \
		--target wasm32-unknown-unknown --forbid-pattern '^ic-host-' \
		--forbid candid --forbid candid_derive --forbid pocket-ic --forbid clap \
		--require ic-wire=1 --require ic-cdk-executor=1 --require ic0=1
