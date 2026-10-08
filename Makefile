.DEFAULT_GOAL := help
PACKAGE ?= ic-host-artifacts
MSRV ?= 1.88.0
RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main
RELEASE_DELIVERY ?= direct
ifneq ($(RELEASE_DELIVERY),direct)
$(error This repository currently qualifies RELEASE_DELIVERY=direct only)
endif
include ci/tool-versions.env
include make/tools.mk
export YQ := $(CURDIR)/.tools/host/bin/yq

ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

.PHONY: help fmt fmt-check format-tools-check format-tools-test check clippy docs-check test test-artifacts-minimal test-tools-response tools-features-check msrv ci install-hooks tooling-command-check shared-tooling-check dependency-pins-check check-doc-links release-adapter-check publish publish-check publish-command-check release-patch release-minor release-major release-resume release-version release-preflight release-verify release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check
help:
	@echo 'Selected package: check, clippy, docs-check, test, msrv (PACKAGE=<crate>)'
	@echo 'Minimal artifact configuration: test-artifacts-minimal'
	@echo 'Response-only configuration: test-tools-response and tools-features-check'
	@echo 'Formatting and metadata: fmt, fmt-check, shared-tooling-check, dependency-pins-check, check-doc-links, release-adapter-check'
	@echo 'Explicit setup: install-tools, install-host-tools, install-ic-tools, install-hooks'
	@echo 'Offline setup checks: tools-check, host-tools-check, ic-tools-check'
	@echo 'Optional Rust tool bundle: install-rust-tools, rust-tools-check'
	@echo 'Source reports: cloc (this workspace), cloc-tooling (sibling tooling)'
	@echo 'Full gate: ci (explicit request or configured CI only)'
	@echo 'Maintainer releases: release-patch, release-minor, release-major, release-resume VERSION=X.Y.Z'
	@echo 'crates.io publication: publish-check (dry run), publish (upload); offline fixture: publish-command-check'
format-tools-check:
	bash scripts/ci/check-format-tools.sh "$(SHARED_TOOLING_CARGO_SORT_VERSION)"
format-tools-test:
	bash scripts/ci/test-format-tools.sh
fmt: format-tools-check
	bash scripts/ci/check-make-execution.sh
	cargo sort --workspace
	cargo fmt --all
fmt-check: format-tools-check
	bash scripts/ci/check-make-execution.sh
	cargo sort --workspace --check
	cargo fmt --all -- --check
check:
	cargo check -p $(PACKAGE) --all-targets --all-features --locked --offline
clippy:
	cargo clippy -p $(PACKAGE) --all-targets --all-features --locked --offline -- -D warnings
docs-check:
	RUSTDOCFLAGS="-D warnings" cargo doc -p $(PACKAGE) --all-features --locked --offline --no-deps
test:
	cargo test -p $(PACKAGE) --all-targets --all-features --locked --offline
test-artifacts-minimal:
	cargo test -p ic-host-artifacts --no-default-features --lib --locked --offline
test-tools-response:
	cargo test -p ic-host-tools --no-default-features --all-targets --locked --offline
tools-features-check:
	bash scripts/ci/check-tools-features.sh
msrv:
	cargo +$(MSRV) check -p $(PACKAGE) --all-targets --all-features --locked --offline
install-hooks:
	bash scripts/dev/install-git-hooks.sh
shared-tooling-check:
	bash scripts/ci/verify-shared-tooling-snapshot.sh
dependency-pins-check:
	bash scripts/ci/check-dependency-pins.sh --cargo-inheritance
check-doc-links:
	perl scripts/ci/check-documentation-links.pl --root "$(CURDIR)" README.md AGENTS.md CHANGELOG.md docs/changelog/0.4.md docs/changelog/0.5.md docs/changelog/0.6.md docs/changelog/0.7.md docs/changelog/0.8.md docs/extraction.md docs/hosts.md docs/status/current.md docs/publishing.md
release-adapter-check:
	bash scripts/release/test-adapter.sh
tooling-command-check:
	bash scripts/ci/test-tool-commands.sh
	bash scripts/ci/test-rust-tools.sh
	bash scripts/ci/check-release-commands.sh "$(CURDIR)" ci/tool-versions.env make/tools.mk
	bash scripts/ci/test-cloc.sh
publish:
	bash scripts/publish/workspace.sh publish
publish-check:
	bash scripts/publish/workspace.sh check
publish-command-check:
	bash scripts/publish/test-workspace.sh
ci:
	+$(MAKE) --no-print-directory shared-tooling-check
	+$(MAKE) --no-print-directory host-tools-check
	+$(MAKE) --no-print-directory dependency-pins-check
	+$(MAKE) --no-print-directory fmt-check
	+$(MAKE) --no-print-directory format-tools-test
	+$(MAKE) --no-print-directory check-doc-links
	+$(MAKE) --no-print-directory release-adapter-check
	+$(MAKE) --no-print-directory tooling-command-check
	+$(MAKE) --no-print-directory publish-command-check
	+$(MAKE) --no-print-directory test-artifacts-minimal
	+$(MAKE) --no-print-directory tools-features-check
	+$(MAKE) --no-print-directory test-tools-response
	@for package in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools; do \
		$(MAKE) --no-print-directory clippy PACKAGE=$$package && \
		$(MAKE) --no-print-directory docs-check PACKAGE=$$package && \
		$(MAKE) --no-print-directory test PACKAGE=$$package || exit $$?; \
	done
release-patch release-minor release-major:
	+@bash scripts/ci/run-release.sh "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"
release-resume:
	+@bash scripts/ci/run-release.sh resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"
release-version:
	@bash scripts/release/adapter.sh version
release-preflight:
	@bash scripts/release/adapter.sh preflight
release-verify:
	+@bash scripts/release/adapter.sh verify
release-prepare-version:
	@bash scripts/release/adapter.sh prepare
release-prepared-check:
	@bash scripts/release/adapter.sh check
release-files:
	@printf '%s\0' Cargo.toml Cargo.lock CHANGELOG.md
release-commit-check:
	@bash scripts/release/adapter.sh commit-check
release-committed-check release-tagged-check release-push-check:
	@bash scripts/release/adapter.sh committed-check
