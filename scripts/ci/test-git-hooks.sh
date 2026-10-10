#!/usr/bin/env bash
# Shared companions: .githooks/pre-commit scripts/dev/install-git-hooks.sh scripts/ci/check-make-execution.sh scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh scripts/ci/check-formatting-hooks.sh scripts/dev/format-frontend.sh make/tools.mk make/rust-format.mk make/execution.mk ci/tool-versions.env
set -euo pipefail

unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES

ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
# shellcheck source=/dev/null
source "$ROOT/ci/tool-versions.env"
bash "$ROOT/scripts/ci/check-format-tools.sh" "$SHARED_TOOLING_CARGO_SORT_VERSION"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/git-hooks-test.XXXXXX")"
# Bash 3.2 can enter EXIT with status zero after nounset; require completion too.
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf -- "$FIXTURE"
    else printf "Failed hook fixture retained: %s\n" "$FIXTURE" >&2; fi
    exit "$status"
}
trap finish EXIT
# Reuse an existing source commit read-only, without creating fixture commits.
source_commit="$(git -C "$ROOT" rev-parse HEAD)"
source_objects="$(git -C "$ROOT" rev-parse --git-path objects)"
case "$source_objects" in /*) ;; *) source_objects="$ROOT/$source_objects" ;; esac
mkdir "$FIXTURE/templates"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TEMPLATE_DIR="$FIXTURE/templates"

new_fixture() {
    mkdir -p "$FIXTURE/$1"
    cd "$FIXTURE/$1"
    git init --quiet
    mkdir -p .git/objects/info .githooks scripts/dev
    printf '%s\n' "$source_objects" > .git/objects/info/alternates
    git update-ref HEAD "$source_commit"
    git read-tree HEAD
    git checkout-index --all
    cp "$ROOT/.githooks/pre-commit" .githooks/pre-commit
    cp "$ROOT/scripts/ci/check-make-execution.sh" scripts/ci/check-make-execution.sh
    cp "$ROOT/scripts/dev/install-git-hooks.sh" scripts/dev/install-git-hooks.sh
    cat > Makefile <<'MAKE'
.PHONY: fmt
fmt:
	@bash scripts/fixture-fmt.sh
MAKE
    cat > scripts/fixture-fmt.sh <<'FORMAT'
#!/usr/bin/env bash
set -euo pipefail
[[ -z "${GIT_INDEX_FILE:-}" && -z "${GIT_DIR:-}" ]] || exit 1
for path in *.rs Cargo.toml README.md; do
    [[ -f "$path" ]] || continue
    sed 's/unformatted/formatted/g' "$path" > "$path.fmt"
    cat "$path.fmt" > "$path"
    rm "$path.fmt"
done
case "${FORMAT_TEST_RACE:-}" in
    index)
        printf 'concurrent staged edit\n' > "$FORMAT_TEST_REAL_ROOT/concurrent.txt"
        git -C "$FORMAT_TEST_REAL_ROOT" add concurrent.txt
        ;;
    worktree) printf 'concurrent working edit\n' > "$FORMAT_TEST_REAL_ROOT/staged.rs" ;;
esac
[[ "${FORMAT_TEST_FAIL:-}" != yes ]] || exit 1
FORMAT
    printf 'unformatted\n' > Cargo.toml
    git add Makefile scripts/fixture-fmt.sh scripts/ci/check-make-execution.sh Cargo.toml
}
expect_failure() {
    if "$@" > output 2>&1; then
        echo 'git hook test unexpectedly accepted a failing fixture' >&2
        exit 1
    fi
}

new_fixture 'fully staged'
printf 'unformatted\n' > 'staged [1].rs'
newline_path=$'staged\nnewline.rs'
printf 'unformatted\n' > "$newline_path"
printf 'unformatted untracked\n' > unselected.rs
printf 'unformatted unrelated working edit\n' > README.md
git add -- 'staged [1].rs' "$newline_path"
bash .githooks/pre-commit > output
[[ "$(git show ':staged [1].rs')" == formatted && "$(git show ":$newline_path")" == formatted ]] || exit 1
[[ "$(git show :Cargo.toml)" == formatted && "$(cat README.md)" == 'unformatted unrelated working edit' ]] || exit 1
[[ "$(git show :README.md)" == "$(git show HEAD:README.md)" ]] || exit 1
[[ "$(cat unselected.rs)" == 'unformatted untracked' && -z "$(git ls-files -- unselected.rs)" ]] || exit 1
tree="$(git write-tree)"
bash .githooks/pre-commit > output
[[ "$(git write-tree)" == "$tree" ]] || exit 1

new_fixture no-selection
git read-tree HEAD
tree="$(git write-tree)"
bash .githooks/pre-commit > output
[[ "$(git write-tree)" == "$tree" && "$(cat Cargo.toml)" == unformatted ]] || exit 1

# Prepared checkout-local tools stay discoverable without a caller PATH export.
# The tools are untracked; formatter inputs still come from the selected index.
for tools in prepared missing wrong; do
    new_fixture "local tools $tools"
    mkdir -p .tools/host/bin .tools/ic/bin .tools/rust/bin
    cat > .tools/rust/bin/hook-fixture-cargo <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    'sort --version') printf 'cargo-sort %s\n' "$(cat "${0%/*}/version")" ;;
    'fmt --version') echo 'rustfmt fixture' ;;
    'sort --workspace')
        hook-fixture-host
        hook-fixture-ic
        bash scripts/fixture-fmt.sh ;;
    *) exit 1 ;;
esac
TOOL
    printf '%s\n' "$SHARED_TOOLING_CARGO_SORT_VERSION" > .tools/rust/bin/version
    for kind in host ic; do
        printf '#!/usr/bin/env bash\nexit 0\n' > ".tools/$kind/bin/hook-fixture-$kind"
        chmod +x ".tools/$kind/bin/hook-fixture-$kind"
    done
    chmod +x .tools/rust/bin/hook-fixture-cargo
    cat > Makefile <<'MAKE'
include make/tools.mk
.PHONY: fmt
fmt:
	bash scripts/ci/check-format-tools.sh $(SHARED_TOOLING_CARGO_SORT_VERSION) hook-fixture-cargo
	env hook-fixture-cargo sort --workspace
include ci/tool-versions.env
MAKE
    git add Makefile
    printf 'unformatted unrelated working edit\n' > README.md
    # env resolves the exported recipe PATH even with Apple's posix_spawn Make.
    # Its direct-command lookup uses Make's original process PATH instead.
    PATH=/usr/bin:/bin make --no-print-directory fmt > output
    printf 'unformatted\n' > Cargo.toml
    printf 'unformatted unrelated working edit\n' > README.md
    tree="$(git write-tree)"
    case "$tools" in
        missing) rm .tools/rust/bin/hook-fixture-cargo ;;
        wrong) printf '0.0.0\n' > .tools/rust/bin/version ;;
    esac
    if [[ "$tools" == prepared ]]; then
        PATH=/usr/bin:/bin "$BASH" .githooks/pre-commit > output
        [[ "$(git show :Cargo.toml)" == formatted && "$(cat Cargo.toml)" == formatted ]] || exit 1
    else
        PATH=/usr/bin:/bin expect_failure "$BASH" .githooks/pre-commit
        [[ "$(git write-tree)" == "$tree" && "$(cat Cargo.toml)" == unformatted ]] || exit 1
    fi
    [[ "$(cat README.md)" == 'unformatted unrelated working edit' ]] || exit 1
done

new_fixture partial
printf 'unformatted selected\n' > partial.rs
git add partial.rs
printf 'unformatted unstaged\n' >> partial.rs
tree="$(git write-tree)"
cp partial.rs before
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" && "$(cat Cargo.toml)" == unformatted ]] || exit 1
cmp before partial.rs

new_fixture formatter-failure
printf 'unformatted\n' > staged.rs
git add staged.rs
tree="$(git write-tree)"
FORMAT_TEST_FAIL=yes expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" && "$(cat staged.rs)" == unformatted && "$(cat Cargo.toml)" == unformatted ]] || exit 1

# A tree ID on stdout is not a successful observation. Fail before any real
# file refresh or staging, including when the snapshot formatter already ran.
real_git="$(command -v git)"
for observation in 1 2 3; do
    new_fixture "hook-tree-failure-$observation"
    printf 'unformatted\n' > staged.rs
    printf 'unrelated working edit\n' > README.md
    cat >> scripts/fixture-fmt.sh <<'FORMAT'
printf 'called\n' >> "$HOOK_TEST_FORMAT_MARKER"
FORMAT
    git add staged.rs scripts/fixture-fmt.sh
    tree="$(git write-tree)"
    cp .git/index index-before
    mkdir mock-bin
    cat > mock-bin/git <<'GIT'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$HOOK_TEST_STATE/commands"
if [[ "$*" == write-tree ]]; then
    count=0
    [[ ! -f "$HOOK_TEST_STATE/count" ]] || read -r count < "$HOOK_TEST_STATE/count"
    count=$((count + 1))
    printf '%s\n' "$count" > "$HOOK_TEST_STATE/count"
    "$HOOK_TEST_GIT" "$@"
    if [[ "$count" == "$HOOK_TEST_OBSERVATION" ]]; then
        echo 'injected write-tree observation failure' >&2
        exit 23
    fi
    exit 0
fi
exec "$HOOK_TEST_GIT" "$@"
GIT
    chmod +x mock-bin/git
    if PATH="$PWD/mock-bin:$PATH" HOOK_TEST_GIT="$real_git" \
        HOOK_TEST_STATE="$PWD/mock-bin" HOOK_TEST_OBSERVATION="$observation" \
        HOOK_TEST_FORMAT_MARKER="$PWD/formatter-called" \
        bash .githooks/pre-commit > output 2>&1; then
        echo 'pre-commit accepted a failed Git tree observation' >&2
        exit 1
    else
        [[ $? == 23 ]] || exit 1
    fi
    rg -F 'injected write-tree observation failure' output >/dev/null
    [[ "$(cat mock-bin/count)" == "$observation" && "$(tail -n 1 mock-bin/commands)" == write-tree ]] || exit 1
    cmp index-before .git/index
    [[ "$(git write-tree)" == "$tree" && "$(cat staged.rs)" == unformatted && "$(cat Cargo.toml)" == unformatted ]] || exit 1
    [[ "$(cat README.md)" == 'unrelated working edit' ]] || exit 1
    if [[ "$observation" == 3 ]]; then
        [[ "$(cat formatter-called)" == called ]] || exit 1
    else
        [[ ! -e formatter-called ]] || exit 1
    fi
done

for flags in i n q t v --ignore-errors; do
    new_fixture "make-mode-$flags"
    printf 'unformatted\n' > staged.rs
    git add staged.rs
    tree="$(git write-tree)"
    MAKEFLAGS="$flags" FORMAT_TEST_FAIL=yes expect_failure bash .githooks/pre-commit
    rg -F 'requires recipe execution and failure propagation' output >/dev/null
    [[ "$(git write-tree)" == "$tree" && "$(cat staged.rs)" == unformatted && "$(cat Cargo.toml)" == unformatted ]] || exit 1
done

for race in index worktree; do
    new_fixture "concurrent-$race"
    printf 'unformatted\n' > staged.rs
    git add staged.rs
    tree="$(git write-tree)"
    FORMAT_TEST_RACE="$race" FORMAT_TEST_REAL_ROOT="$PWD" expect_failure bash .githooks/pre-commit
    [[ "$(git show :staged.rs)" == unformatted ]] || exit 1
    case "$race" in
        index) [[ "$(git show :concurrent.txt)" == 'concurrent staged edit' && "$(cat staged.rs)" == unformatted ]] || exit 1 ;;
        worktree) [[ "$(git write-tree)" == "$tree" && "$(cat staged.rs)" == 'concurrent working edit' ]] || exit 1 ;;
    esac
done

new_fixture alternate-index
printf 'unformatted\n' > staged.rs
git add staged.rs
real_tree="$(git write-tree)"
cp .git/index "$FIXTURE/alternate.index"
GIT_DIR="$PWD/.git" GIT_INDEX_FILE="$FIXTURE/alternate.index" bash .githooks/pre-commit > output
[[ "$(git write-tree)" == "$real_tree" && "$(git show :staged.rs)" == unformatted ]] || exit 1
[[ "$(GIT_INDEX_FILE="$FIXTURE/alternate.index" git show :staged.rs)" == formatted && "$(cat staged.rs)" == formatted ]] || exit 1

new_fixture symlink
printf 'unformatted\n' > outside.rs
ln -s outside.rs linked.rs
git add linked.rs
tree="$(git write-tree)"
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" && "$(cat outside.rs)" == unformatted ]] || exit 1

new_fixture deleted
git rm --quiet README.md
bash .githooks/pre-commit > output
[[ ! -e README.md && -z "$(git ls-files -- README.md)" ]] || exit 1

new_fixture installer
bash scripts/dev/install-git-hooks.sh > output
[[ "$(git config --local --get core.hooksPath)" == .githooks ]] || exit 1
bash scripts/dev/install-git-hooks.sh > output

# Logical aliases such as macOS /var versus /private/var name the same root.
new_fixture installer-path-alias
ln -s "$PWD" "$FIXTURE/installer-alias"
(
    cd "$FIXTURE/installer-alias"
    bash scripts/dev/install-git-hooks.sh > output
)
[[ "$(git config --local --get core.hooksPath)" == .githooks ]] || exit 1

new_fixture installer-configured
git config --local core.hooksPath custom-hooks
expect_failure bash scripts/dev/install-git-hooks.sh
[[ "$(git config --get core.hooksPath)" == custom-hooks ]] || exit 1

# Git values are literal paths: trailing newlines must not become .githooks.
for scope in local inherited; do
    new_fixture "installer-newline-$scope"
    config=.git/config
    if [[ "$scope" == inherited ]]; then config="$FIXTURE/newline-global-config"; fi
    git config --file "$config" core.hooksPath $'.githooks\n\n'
    cp .git/config local-before
    cp "$config" config-before
    if [[ "$scope" == inherited ]]; then
        GIT_CONFIG_GLOBAL="$config" expect_failure bash scripts/dev/install-git-hooks.sh
    else
        expect_failure bash scripts/dev/install-git-hooks.sh
    fi
    cmp local-before .git/config
    cmp config-before "$config"
done

new_fixture $'installer-root\n'
bash scripts/dev/install-git-hooks.sh > output
bash scripts/dev/install-git-hooks.sh >> output
[[ "$(git config --local --get core.hooksPath)" == .githooks ]] || exit 1
printf 'unformatted\n' > staged.rs
git add staged.rs
bash .githooks/pre-commit >> output
[[ "$(git show :staged.rs)" == formatted && "$(cat staged.rs)" == formatted ]] || exit 1

# A failed observation must retain its status, even after printing a value.
for observation in config root; do
    new_fixture "installer-read-failure-$observation"
    mkdir mock-bin
    cat > mock-bin/git <<'GIT'
#!/usr/bin/env bash
if [[ "$HOOK_TEST_OBSERVATION" == config && "$*" == 'config --get core.hooksPath' ]]; then
    printf '.githooks\n'
    exit 23
fi
if [[ "$HOOK_TEST_OBSERVATION" == root && "$*" == 'rev-parse --show-toplevel' ]]; then
    printf '%s\n' "$PWD"
    exit 23
fi
exec "$HOOK_TEST_GIT" "$@"
GIT
    chmod +x mock-bin/git
    cp .git/config config-before
    if PATH="$PWD/mock-bin:$PATH" HOOK_TEST_GIT="$real_git" HOOK_TEST_OBSERVATION="$observation" \
        bash scripts/dev/install-git-hooks.sh > output 2>&1; then
        echo 'hook installer accepted a failed Git observation' >&2
        exit 1
    else
        [[ $? == 23 ]] || exit 1
    fi
    cmp config-before .git/config
done

new_fixture installer-disabled
git config --local core.hooksPath ''
expect_failure bash scripts/dev/install-git-hooks.sh
[[ "$(git config --local --get core.hooksPath)" == '' ]] || exit 1

new_fixture installer-inherited
printf '[core]\n\thooksPath = inherited-hooks\n' > "$FIXTURE/global-config"
GIT_CONFIG_GLOBAL="$FIXTURE/global-config" expect_failure bash scripts/dev/install-git-hooks.sh
[[ -z "$(git config --local --get core.hooksPath || true)" ]] || exit 1

new_fixture installer-private-hook
mkdir .git/hooks
printf '#!/bin/sh\nexit 0\n' > .git/hooks/pre-push
chmod +x .git/hooks/pre-push
cp .git/hooks/pre-push before
expect_failure bash scripts/dev/install-git-hooks.sh
cmp before .git/hooks/pre-push
[[ -z "$(git config --get core.hooksPath || true)" ]] || exit 1

new_fixture installer-non-executable
chmod -x .githooks/pre-commit
expect_failure bash scripts/dev/install-git-hooks.sh
[[ ! -x .githooks/pre-commit && -z "$(git config --get core.hooksPath || true)" ]] || exit 1

# Qualify the optional shared formatter against real Cargo and the actual hook,
# including the adoption checker's partial-stage and preservation cases.
new_fixture $'shared-format-include\n'
cp "$ROOT/make/rust-format.mk" "$ROOT/make/execution.mk" make/
cp "$ROOT/scripts/ci/check-format-tools.sh" "$ROOT/scripts/ci/run-formatting.sh" scripts/ci/
cat > Makefile <<'MAKE'
include make/tools.mk make/rust-format.mk
MAKE
cat > Cargo.toml <<'CARGO'
[package]
name = "shared-format-fixture"
version = "0.0.0"
edition = "2021"

[workspace]
CARGO
mkdir -p src
printf 'pub fn fixture( ){}\n' > src/lib.rs
git add Makefile Cargo.toml src/lib.rs make/rust-format.mk make/execution.mk scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh
bash .githooks/pre-commit > output
[[ "$(git show :src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
cp .git/index "$FIXTURE/newline-index"
cp src/lib.rs "$FIXTURE/newline-source"
CDPATH="$FIXTURE" bash "$ROOT/scripts/ci/check-formatting-hooks.sh" "$PWD" src/lib.rs Cargo.toml --no-dependency-tables \
    make/tools.mk make/rust-format.mk make/execution.mk ci/tool-versions.env scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh
cmp .git/index "$FIXTURE/newline-index"
cmp src/lib.rs "$FIXTURE/newline-source"

# Exercise real Cargo/rustfmt on both a root and a standalone nested workspace.
# No dependencies, builds, Git commits or network access are needed.
for lockfiles in inherited absent present; do
    new_fixture "cargo-$lockfiles"
    cat > Makefile <<'MAKE'
.PHONY: fmt
fmt:
	cargo sort --workspace
	cargo sort --workspace testing
	cargo fmt --all
	cargo fmt --manifest-path testing/Cargo.toml --all
MAKE
    cat > Cargo.toml <<'CARGO'
[workspace]
[package]
name = "hook-root-fixture"
version = "0.0.0"
edition = "2021"
CARGO
    mkdir -p src testing/src
    cp Cargo.toml testing/Cargo.toml
    printf 'pub fn fixture( ){}\n' > src/lib.rs
    printf 'pub fn fixture( ){}\n' > testing/src/lib.rs
    case "$lockfiles" in
        absent) rm -f Cargo.lock testing/Cargo.lock ;;
        present)
            for workspace in . testing; do
                CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo generate-lockfile --offline --manifest-path "$workspace/Cargo.toml" > output 2>&1
            done
            ;;
    esac
    for workspace in . testing; do
        if [[ -f "$workspace/Cargo.lock" ]]; then cp "$workspace/Cargo.lock" "$workspace/selected-lock"; fi
    done
    git add Makefile Cargo.toml src/lib.rs testing/Cargo.toml testing/src/lib.rs
    CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > output
    [[ "$(git show :src/lib.rs)" == 'pub fn fixture() {}' && "$(git show :testing/src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
    for workspace in . testing; do
        if [[ -f "$workspace/selected-lock" ]]; then
            cmp "$workspace/selected-lock" "$workspace/Cargo.lock"
        else
            [[ ! -e "$workspace/Cargo.lock" ]] || exit 1
        fi
    done
    [[ ! -e target && ! -e testing/target ]] || exit 1
done

# Sorting must cover inherited member tables and separate workspace catalogs,
# preserving the effective metadata and already selected lockfiles.
new_fixture cargo-sort
cat > Makefile <<'MAKE'
.PHONY: fmt fmt-check
fmt:
	cargo sort --workspace
	cargo sort --workspace testing
	cargo fmt --all
	cargo fmt --manifest-path testing/Cargo.toml --all
fmt-check:
	cargo sort --workspace --check
	cargo sort --workspace --check testing
	cargo fmt --all -- --check
	cargo fmt --manifest-path testing/Cargo.toml --all -- --check
MAKE
for workspace in . testing; do
    mkdir -p "$workspace/src" "$workspace/alpha/src" "$workspace/zeta/src" "$workspace/consumer/src"
    cat > "$workspace/Cargo.toml" <<'CARGO'
[workspace]
members = ["zeta", "consumer", "alpha"]
resolver = "2"

[workspace.package]
version = "0.0.0"
edition = "2021"

[workspace.dependencies]
# Preserve the selected source and feature policy.
zeta = { path = "zeta", default-features = false }
alpha = { path = "alpha" }

[package]
name = "sort-root-fixture"
version.workspace = true
edition.workspace = true

[dependencies]
zeta = { workspace = true, features = ["extra"] }
alpha.workspace = true
CARGO
    for member in alpha zeta; do
        printf '[package]\nname = "%s"\nversion.workspace = true\nedition.workspace = true\n\n[features]\nextra = []\n' "$member" > "$workspace/$member/Cargo.toml"
        printf 'pub fn fixture() {}\n' > "$workspace/$member/src/lib.rs"
    done
    cat > "$workspace/consumer/Cargo.toml" <<'CARGO'
[package]
name = "sort-consumer-fixture"
version.workspace = true
edition.workspace = true

[dependencies]
zeta.workspace = true
alpha.workspace = true

[dev-dependencies]
zeta.workspace = true
alpha.workspace = true

[build-dependencies]
zeta.workspace = true
alpha.workspace = true

[target.'cfg(unix)'.dependencies]
zeta.workspace = true
alpha.workspace = true
CARGO
    printf 'pub fn fixture() {}\n' > "$workspace/src/lib.rs"
    printf 'pub fn fixture() {}\n' > "$workspace/consumer/src/lib.rs"
    CARGO_NET_OFFLINE=true cargo generate-lockfile --offline --manifest-path "$workspace/Cargo.toml" > output 2>&1
    cp "$workspace/Cargo.lock" "$workspace/selected-lock"
    CARGO_NET_OFFLINE=true cargo metadata --offline --locked --no-deps --format-version 1 --manifest-path "$workspace/Cargo.toml" |
        jq -S '[.packages[] | .dependencies |= sort_by(.name, .kind, .target)] | sort_by(.name)' > "$workspace/selected-metadata"
done
git add Makefile Cargo.toml src alpha zeta consumer testing
cp consumer/Cargo.toml "$FIXTURE/unsorted-consumer.toml"
tree="$(git write-tree)"
expect_failure make --no-print-directory fmt-check
[[ "$(git write-tree)" == "$tree" ]] || exit 1
git --literal-pathspecs diff --quiet -- Cargo.toml consumer/Cargo.toml testing/Cargo.toml testing/consumer/Cargo.toml
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > output
make --no-print-directory fmt-check > output
for workspace in . testing; do
    cmp "$workspace/selected-lock" "$workspace/Cargo.lock"
    CARGO_NET_OFFLINE=true cargo metadata --offline --locked --no-deps --format-version 1 --manifest-path "$workspace/Cargo.toml" |
        jq -S '[.packages[] | .dependencies |= sort_by(.name, .kind, .target)] | sort_by(.name)' > "$workspace/sorted-metadata"
    cmp "$workspace/selected-metadata" "$workspace/sorted-metadata"
    grep -q '# Preserve the selected source and feature policy.' "$workspace/Cargo.toml"
    awk '$0 == "[workspace.dependencies]" { catalog=1; next } /^\[/ { catalog=0 } catalog && /^(alpha|zeta) / { sub(/ .*/, ""); print }' "$workspace/Cargo.toml" > observed-order
    printf 'alpha\nzeta\n' > expected-order
    cmp expected-order observed-order
done
tree="$(git write-tree)"
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > output
[[ "$(git write-tree)" == "$tree" && ! -e target && ! -e testing/target ]] || exit 1
# Exercise the adoption helper with this consumer's real Cargo targets, including
# inherited Make/Git identities that must not redirect the disposable checkout.
mkdir -p .tools/host/bin
printf '#!/usr/bin/env bash\nexit 0\n' > .tools/host/bin/hook-fixture-prerequisite
chmod +x .tools/host/bin/hook-fixture-prerequisite
cat >> Makefile <<'MAKE'
.PHONY: local-tools
fmt fmt-check: local-tools
local-tools:
	hook-fixture-prerequisite
MAKE
git add Makefile
overlays=(Cargo.toml Cargo.lock src/lib.rs)
for workspace in . testing; do
    for member in alpha zeta consumer; do
        prefix="$member"; [[ "$workspace" == . ]] || prefix="$workspace/$member"
        overlays[${#overlays[@]}]="$prefix/Cargo.toml"
        overlays[${#overlays[@]}]="$prefix/src/lib.rs"
    done
done
overlays[${#overlays[@]}]=testing/Cargo.toml
overlays[${#overlays[@]}]=testing/Cargo.lock
overlays[${#overlays[@]}]=testing/src/lib.rs
GIT_INDEX_FILE="$FIXTURE/incorrect-index" MAKEFLAGS='--just-print' \
    VALIDATION_REPOSITORY_ROOT=/incorrect \
    bash "$ROOT/scripts/ci/check-formatting-hooks.sh" "$PWD" consumer/src/lib.rs \
    consumer/Cargo.toml "$FIXTURE/unsorted-consumer.toml" "${overlays[@]}"
# alpha has no dependency tables; qualify the same real formatter without
# inventing dependencies or weakening manifest/index preservation checks.
bash "$ROOT/scripts/ci/check-formatting-hooks.sh" "$PWD" alpha/src/lib.rs \
    alpha/Cargo.toml --no-dependency-tables "${overlays[@]}"
# Mixed frontend selection uses prepared tools but only snapshot file inputs.
new_fixture frontend
mkdir frontend
cp "$ROOT/scripts/dev/format-frontend.sh" scripts/dev/format-frontend.sh
cat > Makefile <<'MAKE'
.PHONY: fmt fmt-check
fmt:
	@bash scripts/dev/format-frontend.sh --write frontend
fmt-check:
	@bash scripts/dev/format-frontend.sh --check frontend
MAKE
export PRETTIER_BIN="$FIXTURE/prepared-prettier" PRETTIER_VERSION=99.1.0
cat > "$PRETTIER_BIN" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == --version ]]; then echo 99.1.0; exit "${PRETTIER_VERSION_STATUS:-0}"; fi
[[ -z "${PRETTIER_CALL_MARKER:-}" ]] || touch "$PRETTIER_CALL_MARKER"
[[ "${PRETTIER_FIXTURE_FAIL:-}" != yes ]] || exit 9
mode="$1"
[[ "$2" == -- ]] || exit 1
shift 2
for path in "$@"; do
    if [[ "$mode" == --check ]]; then
        if grep -q unformatted "$path"; then exit 1; fi
    else
        sed 's/unformatted/formatted/g' "$path" > "$path.tmp"
        mv "$path.tmp" "$path"
    fi
done
SCRIPT
chmod +x "$PRETTIER_BIN"
frontend_path=$'frontend/selected [1]\nname.ts'
printf 'unformatted\n' > "$frontend_path"
printf 'formatted\n' > frontend/unselected.ts
git add Makefile scripts/dev/format-frontend.sh "$frontend_path" frontend/unselected.ts
bash .githooks/pre-commit > output
[[ "$(cat "$frontend_path")" == formatted ]] || exit 1
tree="$(git write-tree)"
bash .githooks/pre-commit > output
[[ "$(git write-tree)" == "$tree" ]] || exit 1
# A manual check covers full tracked scope; partial staging still fails before
# invoking the prepared frontend formatter.
printf 'unformatted working edit\n' > frontend/unselected.ts
expect_failure make --no-print-directory fmt-check
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]] || exit 1
printf 'formatted\n' > frontend/unselected.ts
printf 'unformatted\n' > "$frontend_path"
git add "$frontend_path"
tree="$(git write-tree)"
PRETTIER_FIXTURE_FAIL=yes expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" && "$(cat "$frontend_path")" == unformatted ]] || exit 1
PRETTIER_VERSION=wrong expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]] || exit 1
PRETTIER_VERSION_STATUS=23 PRETTIER_CALL_MARKER="$FIXTURE/formatter-called" \
    expect_failure bash .githooks/pre-commit
[[ ! -e "$FIXTURE/formatter-called" && "$(git write-tree)" == "$tree" ]] || exit 1
[[ "$(cat "$frontend_path")" == unformatted ]] || exit 1
for mode in --check --write; do
    expect_failure bash scripts/dev/format-frontend.sh "$mode" fronted
done
# No selected frontend input is legitimate, even when the staged tree has no
# frontend directory (for example after deleting its final tracked file).
: > "$FIXTURE/no-frontend-selection"
SHARED_TOOLING_FORMAT_FILES="$FIXTURE/no-frontend-selection" PRETTIER_BIN=/absent \
    bash scripts/dev/format-frontend.sh --write absent-frontend
# The explicit list also demonstrates that an unselected frontend file is never
# passed to the formatter. Use the helper in a disposable snapshot directory.
printf '%s\0' "$frontend_path" > "$FIXTURE/frontend-selection"
printf 'unformatted unselected\n' > frontend/unselected.ts
SHARED_TOOLING_FORMAT_FILES="$FIXTURE/frontend-selection" \
    bash scripts/dev/format-frontend.sh --write frontend
[[ "$(cat "$frontend_path")" == formatted && "$(cat frontend/unselected.ts)" == 'unformatted unselected' ]] || exit 1
echo 'Git hook preservation, installation, Cargo sorting and frontend selection tests passed'
fixture_complete=true
