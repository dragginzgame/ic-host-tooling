.DEFAULT_GOAL := help
override SHARED_TOOLING_ROOT := $(CURDIR)
export SHARED_TOOLING_ROOT
export RELEASE_DELIVERY ?= direct
export YQ := $(CURDIR)/.tools/host/bin/yq
export CARGO_RELEASE_METADATA_MAX_BYTES ?= 8388608
include make/consumer.mk

.PHONY: help version tasks check ci check-doc-links check-workspace check-pins \
        check-package check-boundary clippy docs-check test msrv \
        check-release-commands install-validation-tools lint check-shell \
        check-workflows check-secrets release-version release-preflight \
        release-verify release-prepare-version release-prepared-check release-files \
        release-commit-check release-committed-check release-tagged-check release-push-check \
        check-release-adapters install-release-tools release-tools-check package publish publish-check

_release_targets := release-patch release-minor release-major release-resume \
                    release-version release-preflight release-verify release-prepare-version \
                    release-prepared-check release-files release-commit-check \
                    release-committed-check release-tagged-check release-push-check
ifneq ($(filter $(_release_targets),$(MAKECMDGOALS)),)
ifneq ($(RELEASE_DELIVERY),direct)
$(error IC Build supports RELEASE_DELIVERY=direct only)
endif
endif
release-patch release-minor release-major release-resume: export IC_BUILD_RELEASE_CACHE_PREPARE := 1
SHELLCHECK ?= $(CURDIR)/.tools/ci/bin/shellcheck
ACTIONLINT ?= $(CURDIR)/.tools/ci/bin/actionlint
GITLEAKS ?= $(CURDIR)/.tools/ci/bin/gitleaks

help:
	@echo 'version / tasks: workspace version and shared maintenance catalog'
	@echo 'check / ci: snapshot, tools, docs, pins, formatting, package/MSRV checks and release routing'
	@echo 'install-tools / tools-check: explicit shared setup and offline admission'
	@echo 'install-release-tools / release-tools-check: prepare / admit the pinned published Cargo editor'
	@echo 'fmt / fmt-check: sort Cargo manifests and format/check all workspace Rust source'
	@echo 'install-hooks: activate the shared repository-local pre-commit formatter'
	@echo 'check-package / clippy / docs-check / test / msrv: compile, lint, document and qualify the extracted engine'
	@echo 'check-boundary: reject direct Candid/CDK declarations outside the Platform owner'
	@echo 'cloc: shared workspace Rust LOC report'
	@echo 'install-validation-tools: pinned Linux x86-64 ShellCheck, actionlint and Gitleaks'
	@echo 'lint / check-secrets: shell/workflow lint and history secret scan'
	@echo 'release-*: shared runner; validate, prepare, commit, tag and atomically push the selected release'
	@echo 'package / publish-check / publish: local archive / tagged-release dry run / crates.io upload'
	@echo 'check-release-adapters: isolated metadata and publication fixtures, with no live release effects'

version:
	@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

tasks:
	@cat tasks/README.md

check ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast \
		shared-tooling-check tools-check check-doc-links check-workspace check-pins check-boundary \
		fmt-check check-package clippy docs-check test msrv check-release-commands check-release-adapters

check-doc-links:
	perl scripts/ci/check-documentation-links.pl --root "$(CURDIR)" \
		*.md audits/*.md rules/*.md tasks/*.md docs/*.md docs/principles/*.md \
		docs/design/*.md docs/status/*.md

check-workspace:
	+@CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo metadata \
		--offline --locked --no-deps --format-version 1 > /dev/null

check-pins:
	+@bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance

check-boundary:
	@bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)"

check-package:
	+CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo check \
		--workspace --all-targets --all-features --offline --locked

clippy:
	+CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo clippy \
		-p ic-build --all-targets --all-features --offline --locked -- -D warnings

docs-check:
	+CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 RUSTDOCFLAGS='-D warnings' \
		cargo doc --workspace --all-features --no-deps --offline --locked

# Carry the repository-selected compiler into independent fixture workspaces.
test:
	+@task_rustc="$$(RUSTUP_AUTO_INSTALL=0 rustup which rustc)" && \
		CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 RUSTC="$$task_rustc" cargo test -p ic-build \
		--all-features --offline --locked

msrv:
	+@bash -ec 'task_msrv="$$(yq -p toml -o json -r '\''.workspace.package."rust-version"'\'' Cargo.toml)"; \
		RUSTUP_AUTO_INSTALL=0 rustc +"$$task_msrv" --version; \
		RUSTUP_AUTO_INSTALL=0 cargo +"$$task_msrv" --version; \
		CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo +"$$task_msrv" check \
			--workspace --all-targets --all-features --offline --locked'

check-release-commands:
	@bash scripts/ci/check-release-commands.sh "$(CURDIR)" \
		make/consumer.mk make/tools.mk make/rust-format.mk make/release.mk \
		make/execution.mk scripts/ci/check-make-execution.sh

check-release-adapters:
	bash scripts/release/test-adapters.sh

install-release-tools:
	@bash scripts/release/editor.sh prepare > /dev/null

release-tools-check:
	@bash scripts/release/editor.sh check > /dev/null

install-validation-tools:
	@bash -ec 'test "$$(uname -s):$$(uname -m)" = Linux:x86_64; \
		. ci/tool-versions.env; \
		bash scripts/ci/install-ci-tool.sh shellcheck --version "$$SHARED_TOOLING_SHELLCHECK_VERSION" \
			--sha256 "$$SHARED_TOOLING_SHELLCHECK_SHA256_LINUX_X86_64" --install-dir "$(CURDIR)/.tools/ci/bin"; \
		bash scripts/ci/install-ci-tool.sh actionlint --version "$$SHARED_TOOLING_ACTIONLINT_VERSION" \
			--sha256 "$$SHARED_TOOLING_ACTIONLINT_SHA256_LINUX_AMD64" --install-dir "$(CURDIR)/.tools/ci/bin"; \
		bash scripts/ci/install-ci-tool.sh gitleaks --version "$$SHARED_TOOLING_GITLEAKS_VERSION" \
			--sha256 "$$SHARED_TOOLING_GITLEAKS_SHA256_LINUX_X64" --install-dir "$(CURDIR)/.tools/ci/bin"'

lint: check-shell check-workflows

check-shell:
	"$(SHELLCHECK)" scripts/ci/*.sh scripts/dev/*.sh scripts/release/*.sh .githooks/pre-commit

check-workflows:
	"$(ACTIONLINT)" .github/workflows/*.yml

check-secrets:
	"$(GITLEAKS)" git --redact=100 --no-banner --no-color --log-level error "$(CURDIR)"

release-version:
	@bash scripts/release/metadata.sh version

release-preflight:
	+@bash scripts/release/metadata.sh preflight

release-verify:
	+@VALIDATION_FAILURE_LOG_DIR="$$(git rev-parse --git-path release-state)/validation-failures" \
		bash scripts/ci/run-validation-targets.sh --fail-fast ci lint check-secrets

release-prepare-version:
	+@bash scripts/release/metadata.sh prepare

release-prepared-check release-committed-check release-tagged-check release-push-check:
	+@bash scripts/release/metadata.sh check

release-files:
	@printf '%s\0' Cargo.toml Cargo.lock CHANGELOG.md

release-commit-check:
	+@bash scripts/release/metadata.sh commit-check

package:
	+cargo package -p ic-build --locked

publish publish-check:
	+@bash scripts/release/publish.sh "$@"
