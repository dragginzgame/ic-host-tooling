.DEFAULT_GOAL := help
PACKAGE ?= ic-host-artifacts
MSRV ?= 1.88.0
RELEASE_DELIVERY ?= direct
ifneq ($(RELEASE_DELIVERY),direct)
$(error This repository currently qualifies RELEASE_DELIVERY=direct only)
endif
# Host commands use this checkout's reviewed snapshot, including parse-time probes.
override SHARED_TOOLING_ROOT := $(CURDIR)
export SHARED_TOOLING_ROOT
include ci/tool-versions.env
include make/consumer.mk
export YQ := $(CURDIR)/.tools/host/bin/yq

.PHONY: help format-tools-test check clippy docs-check test test-artifacts-minimal test-tools-response tools-features-check msrv ci install-hooks tooling-command-check shared-tooling-check dependency-pins-check check-doc-links release-adapter-check publish publish-check publish-command-check release-version release-preflight release-verify release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check
help:
	@echo 'Selected package: check, clippy, docs-check, test, msrv (PACKAGE=<crate>)'
	@echo 'Minimal artifact configuration: test-artifacts-minimal'
	@echo 'Tools profiles: test-tools-response, test-tools-interface, test-tools-service, test-tools-extraction, test-tools-limits, test-tools-certificate and tools-features-check'
	@echo 'Formatting and metadata: fmt, fmt-check, shared-tooling-check, dependency-pins-check, check-doc-links, release-adapter-check'
	@echo 'Hook qualification: formatting-hook-check (disposable Git fixtures; prepared formatters required)'
	@echo 'Complete setup: install-tools (host, IC and Cargo); install-hooks is separate'
	@echo 'Offline setup checks: tools-check (complete set), host-tools-check, ic-tools-check, rust-tools-check'
	@echo 'Individual setup: install-host-tools, install-ic-tools, install-rust-tools'
	@echo 'Source reports: cloc (this workspace); fleet reports run in Shared Tooling'
	@echo 'Full release validation: ci, msrv-workspace; focused MSRV: msrv PACKAGE=<crate>'
	@echo 'Maintainer releases: release-patch, release-minor, release-major, release-resume VERSION=X.Y.Z'
	@echo 'crates.io publication: publish-check (dry run), publish (upload); offline fixture: publish-command-check'
format-tools-test:
	bash scripts/ci/test-format-tools.sh
.PHONY: formatting-hook-check
formatting-hook-check:
	+bash scripts/ci/test-git-hooks.sh
	+bash scripts/ci/check-host-formatting.sh
check:
	+cargo check -p $(PACKAGE) --all-targets --all-features --locked --offline
clippy:
	+cargo clippy -p $(PACKAGE) --all-targets --all-features --locked --offline -- -D warnings
docs-check:
	+RUSTDOCFLAGS="-D warnings" cargo doc -p $(PACKAGE) --all-features --locked --offline --no-deps
test:
	+cargo test -p $(PACKAGE) --all-targets --all-features --locked --offline
test-artifacts-minimal:
	+cargo test -p ic-host-artifacts --no-default-features --lib --locked --offline
test-tools-response:
	+cargo test -p ic-host-tools --no-default-features --all-targets --locked --offline
.PHONY: test-tools-interface
test-tools-interface:
	+cargo test -p ic-host-tools --no-default-features --features ic-interface --all-targets --locked --offline

.PHONY: test-tools-service
test-tools-service:
	+cargo test -p ic-host-tools --no-default-features --features candid-service --all-targets --locked --offline

.PHONY: test-tools-extraction test-tools-limits test-tools-certificate test-wasm-inspection
test-tools-extraction:
	+cargo test -p ic-host-tools --no-default-features --features candid-extraction --all-targets --locked --offline
test-tools-limits:
	+cargo test -p ic-host-tools --no-default-features --features ic-limits --all-targets --locked --offline
test-tools-certificate:
	+cargo test -p ic-host-tools --no-default-features --features certificate --all-targets --locked --offline
test-wasm-inspection:
	+cargo test -p ic-host-wasm-inspect --all-targets --locked --offline
tools-features-check:
	+bash scripts/ci/check-tools-features.sh
msrv:
	+cargo +$(MSRV) check -p $(PACKAGE) --all-targets --all-features --locked --offline
.PHONY: msrv-workspace
msrv-workspace:
	+cargo +$(MSRV) check --workspace --all-targets --all-features --locked --offline
dependency-pins-check:
	+bash scripts/ci/check-dependency-pins.sh --cargo-inheritance
check-doc-links:
	perl scripts/ci/check-documentation-links.pl --root "$(CURDIR)" README.md AGENTS.md CHANGELOG.md docs/changelog/0.4.md docs/changelog/0.5.md docs/changelog/0.6.md docs/changelog/0.7.md docs/changelog/0.8.md docs/changelog/0.9.md docs/changelog/0.10.md docs/changelog/0.11.md docs/changelog/0.12.md docs/changelog/0.13.md docs/changelog/0.14.md docs/changelog/0.15.md docs/changelog/0.16.md docs/changelog/0.17.md docs/changelog/0.18.md docs/changelog/0.19.md docs/changelog/0.20.md docs/extraction.md docs/hosts.md docs/status/current.md docs/status/archive-2026-10-15.md docs/publishing.md
release-adapter-check:
	+bash scripts/release/test-adapter.sh
tooling-command-check:
	+bash scripts/ci/test-host-fixture-retention.sh
	+bash scripts/ci/test-make-snapshot.sh
	+bash scripts/ci/test-make-consumer.sh
	+bash scripts/ci/test-tool-commands.sh
	+bash scripts/ci/check-release-commands.sh "$(CURDIR)" ci/tool-versions.env make/consumer.mk make/tools.mk make/rust-format.mk make/release.mk make/execution.mk scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh
publish:
	+bash scripts/publish/workspace.sh publish
publish-check:
	+bash scripts/publish/workspace.sh check
publish-command-check:
	+bash scripts/publish/test-workspace.sh
	+bash scripts/ci/test-cargo-publication-attempt.sh
ci:
	+$(MAKE) --no-print-directory shared-tooling-check
	+$(MAKE) --no-print-directory tools-check
	+$(MAKE) --no-print-directory dependency-pins-check
	+$(MAKE) --no-print-directory fmt-check
	+$(MAKE) --no-print-directory format-tools-test
	+$(MAKE) --no-print-directory formatting-hook-check
	+$(MAKE) --no-print-directory check-doc-links
	+$(MAKE) --no-print-directory release-adapter-check
	+$(MAKE) --no-print-directory tooling-command-check
	+$(MAKE) --no-print-directory publish-command-check
	+$(MAKE) --no-print-directory test-artifacts-minimal
	+$(MAKE) --no-print-directory tools-features-check
	+$(MAKE) --no-print-directory test-tools-response
	+$(MAKE) --no-print-directory test-tools-interface
	+$(MAKE) --no-print-directory test-tools-service
	+$(MAKE) --no-print-directory test-tools-extraction
	+$(MAKE) --no-print-directory test-tools-limits
	+$(MAKE) --no-print-directory test-tools-certificate
	+$(MAKE) --no-print-directory test-wasm-inspection
	+cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
	+RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --locked --offline --no-deps
	+cargo test --workspace --all-targets --all-features --locked --offline
release-version:
	+@bash scripts/release/adapter.sh version
release-preflight:
	+@bash scripts/release/adapter.sh preflight
release-verify:
	+@bash scripts/release/adapter.sh verify
release-prepare-version:
	+@bash scripts/release/adapter.sh prepare
release-prepared-check:
	+@bash scripts/release/adapter.sh check
release-files:
	@printf '%s\0' Cargo.toml Cargo.lock CHANGELOG.md
release-commit-check:
	+@bash scripts/release/adapter.sh commit-check
release-committed-check release-tagged-check release-push-check:
	@bash scripts/release/adapter.sh committed-check
