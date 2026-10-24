#!/usr/bin/env bash
set -euo pipefail

# Qualify Host's checkout-local routing using its actual Makefile and snapshot.
# Formatter, tool setup/check and release effects use disposable substitutes.
root="$0"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset SHARED_TOOLING_ROOT
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
# shellcheck source=scripts/ci/make-adapter-probes.sh
source "$root/scripts/ci/make-adapter-probes.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/host-make-snapshot.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Host Make evidence retained"' EXIT
export TMPDIR="${fixture%/*}"
mkdir -p "$fixture/local"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture/local" \
    make/consumer.mk scripts/ci/check-pipe-jobserver.pl scripts/release/editor.sh
for input in Makefile make/testkit.mk ci/tool-versions.env ci/release-delivery.txt scripts/ci/check-format-tools.sh; do
    parent=.
    [[ "$input" != */* ]] || parent="${input%/*}"
    mkdir -p "$fixture/local/$parent"
    cp "$root/$input" "$fixture/local/$input"
done
export HOST_MAKE_EVENTS="$fixture/events" HOST_MAKE_EXTERNAL="$fixture/external-events"
export HOST_MAKE_JOBSERVER_PROBE="$fixture/local/scripts/ci/check-pipe-jobserver.pl"
mkdir -p "$fixture/external/scripts/ci"
for helper in check-make-execution.sh check-format-tools.sh run-release.sh; do
    cat > "$fixture/external/scripts/ci/$helper" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$0" >> "$HOST_MAKE_EXTERNAL"
exit 23
STUB
done
cat > "$fixture/local/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${HOST_MAKE_VERIFY_JOBSERVER:-}" == 1 ]]; then
    perl "$HOST_MAKE_JOBSERVER_PROBE"
fi
if [[ "${HOST_MAKE_ASSERT_POCKET_IC:-0}" == 1 && "$1" == test ]]; then
    [[ "${POCKET_IC_BIN:-}" == '/prepared/pocket ic' ]] || exit 91
fi
case "$*" in
    'sort --version') printf 'cargo-sort %s\n' "$HOST_MAKE_SORT_VERSION" ;;
    'fmt --version') echo rustfmt ;;
    *)
        printf '%s\n' "$*" >> "$HOST_MAKE_EVENTS"
        if [[ "$1" == fetch ]]; then exit "${HOST_MAKE_FETCH_STATUS:-0}"; fi
        if [[ "${HOST_MAKE_ASSERT_MSRV:-0}" == 1 && "$1" == check ]]; then
            [[ "${RUSTUP_TOOLCHAIN:-}" == 1.91.0 ]] || exit 92
        fi
        if [[ "$1" == clippy && "$*" == *--workspace* ]]; then
            exit "${HOST_MAKE_CLIPPY_STATUS:-0}"
        fi
        if [[ "${HOST_MAKE_FAIL:-}" == sort && "$1" == sort ]]; then
            echo 'formatter stdout evidence'
            echo 'formatter stderr evidence' >&2
            exit 23
        fi
        ;;
esac
STUB
cat > "$fixture/local/scripts/ci/run-release.sh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$HOST_MAKE_EVENTS"
STUB
mkdir -p "$fixture/local/scripts/dev"
for helper in scripts/dev/install-git-hooks.sh scripts/ci/verify-shared-tooling-snapshot.sh; do
    cat > "$fixture/local/$helper" <<'STUB'
#!/usr/bin/env bash
printf '%s %s\n' "${0##*/}" "$*" >> "$HOST_MAKE_EVENTS"
STUB
done
chmod +x "$fixture/local/cargo"
# shellcheck source=/dev/null
. "$root/ci/tool-versions.env"
export HOST_MAKE_SORT_VERSION="$SHARED_TOOLING_CARGO_SORT_VERSION"
cd "$fixture/local"
printf 'override RELEASE_REMOTE := recursive\n' > overrides.mk
recursive_make="$(command -v make) --no-print-directory -f Makefile -f overrides.mk"
for selection in ordinary environment command nested recursive; do
    unset SHARED_TOOLING_ROOT
    remote=review
    command=(make --no-print-directory -j2)
    if [[ "$selection" == environment ]]; then
        export SHARED_TOOLING_ROOT="$fixture/external"
    elif [[ "$selection" != ordinary ]]; then
        command+=("SHARED_TOOLING_ROOT=$fixture/external")
    fi
    if [[ "$selection" == nested || "$selection" == recursive ]]; then
        # Keep Make's recursive command and automatic variable literal.
        # shellcheck disable=SC2016
        printf '%%:\n\t+$(MAKE) -C "%s" $@\n' "$PWD" > "$fixture/parent.mk"
        command+=(-f "$fixture/parent.mk")
    fi
    if [[ "$selection" == recursive ]]; then
        command+=("MAKE=$recursive_make")
        remote=recursive
    fi
    for target in help fmt-check release-resume install-hooks shared-tooling-check; do
        : > "$HOST_MAKE_EVENTS"
        "${command[@]}" "$target" "FORMAT_CARGO=$PWD/cargo" \
            VERSION=0.9.5 RELEASE_REMOTE=review RELEASE_BRANCH=release-review \
            > "$fixture/$selection-$target.log" 2>&1
        case "$target" in
            help) : > "$fixture/expected" ;;
            fmt-check) printf 'sort --workspace --grouped --check\nfmt --all -- --check\n' > "$fixture/expected" ;;
            release-resume) printf 'resume 0.9.5 %s release-review\n' "$remote" > "$fixture/expected" ;;
            install-hooks) printf 'install-git-hooks.sh --consumer %s\n' "$PWD" > "$fixture/expected" ;;
            shared-tooling-check) printf 'verify-shared-tooling-snapshot.sh --consumer %s --manifest .shared-tooling.snapshot\n' "$PWD" > "$fixture/expected" ;;
        esac
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
        if [[ "$target" == fmt-check ]]; then
            grep -Fx 'Checking formatting... ok' "$fixture/$selection-$target.log" >/dev/null
        fi
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
    done
done
unset SHARED_TOOLING_ROOT
# Unsupported release delivery refuses before adapter effects; unrelated goals
# remain available with the same environment selection.
: > "$HOST_MAKE_EVENTS"
status=0
make --no-print-directory release-version RELEASE_DELIVERY=pr \
    > "$fixture/unsupported-delivery.log" 2>&1 || status=$?
[[ "$status" == 2 && ! -s "$HOST_MAKE_EVENTS" ]] || exit 1
make --no-print-directory help RELEASE_DELIVERY=pr > "$fixture/unrelated-delivery.log" 2>&1
[[ ! -s "$HOST_MAKE_EVENTS" ]] || exit 1
# Exercise the actual Host aggregate under parallel Make. Each stage must run
# once in shared order, and a failure must prevent every later stage.
mkdir -p "$fixture/local/scripts/dev"
for tool in host ic rust; do
    cat > "$fixture/local/scripts/dev/install-$tool-tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
name="${0##*/}"
mode=install
for argument in "$@"; do
    if [[ "$argument" == --check ]]; then mode=check; fi
    if [[ "$argument" == --preflight ]]; then mode=preflight; fi
done
if [[ "$*" == *'--package ic-build'* ]]; then
    [[ "$*" == *'--bin ic-build-release-metadata --profile debug'* ]] || exit 1
    name=install-editor-tools.sh
fi
if [[ "$name" == install-ic-tools.sh ]]; then
    [[ "$*" == *'--tool wasm-opt'* && "$*" != *'--tool ic-wasm'* ]] || exit 93
    if [[ "${HOST_MAKE_OPTIMIZER_PATH:-}" != '' ]]; then printf '%s\n' "$HOST_MAKE_OPTIMIZER_PATH"; fi
fi
printf '%s %s\n' "$name" "$mode" >> "$HOST_MAKE_EVENTS"
if [[ "${HOST_MAKE_TOOL_FAIL:-}" == "$name" && "${HOST_MAKE_TOOL_FAIL_MODE:-}" == "$mode" ]]; then exit 23; fi
STUB
done
mkdir -p "$fixture/local/target/debug"
cat > "$fixture/local/target/debug/ic-testkit-server" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
mode=install
if [[ "$1" == check ]]; then mode=check; printf '%s\n' '/prepared/pocket ic'; fi
printf 'install-server-tools.sh %s\n' "$mode" >> "$HOST_MAKE_EVENTS"
if [[ "${HOST_MAKE_TOOL_FAIL:-}" == install-server-tools.sh && "${HOST_MAKE_TOOL_FAIL_MODE:-}" == "$mode" ]]; then exit 23; fi
STUB
chmod +x "$fixture/local/target/debug/ic-testkit-server"
for mode in install check; do
    target=install-tools
    [[ "$mode" != check ]] || target=tools-check
    for failed in none host rust editor server; do
        : > "$HOST_MAKE_EVENTS"
        status=0
        PATH="$fixture/local:$PATH" HOST_MAKE_TOOL_FAIL="install-$failed-tools.sh" HOST_MAKE_TOOL_FAIL_MODE="$mode" make --no-print-directory -j4 "$target" "MAKE=$(command -v make)" \
            "SHARED_TOOLING_ROOT=$fixture/external" > "$fixture/$target-$failed.log" 2>&1 || status=$?
        if [[ "$failed" == none ]]; then [[ "$status" == 0 ]] || exit 1;
        else [[ "$status" == 2 ]] || exit 1; fi
        : > "$fixture/expected"
        if [[ "$mode" == install ]]; then
            printf '%s\n' 'install-rust-tools.sh preflight' >> "$fixture/expected"
        fi
        for tool in host rust; do
            printf 'install-%s-tools.sh %s\n' "$tool" "$mode" >> "$fixture/expected"
            [[ "$tool" != "$failed" ]] || break
        done
        if [[ "$failed" == none || "$failed" == editor || "$failed" == server ]]; then
            printf 'install-editor-tools.sh %s\n' "$mode" >> "$fixture/expected"
        fi
        if [[ "$failed" == none || "$failed" == server ]]; then
            if [[ "$mode" == install ]]; then
                printf '%s\n' 'fetch --locked' >> "$fixture/expected"
                printf '%s\n' 'build -p ic-testkit --features cli --locked --offline --bin ic-testkit-server' >> "$fixture/expected"
            fi
            printf 'install-server-tools.sh %s\n' "$mode" >> "$fixture/expected"
        fi
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
    done
done
# A fetch refusal stops explicit server setup before the offline build, even
# under parallel Make. The complete successful order is checked above.
: > "$HOST_MAKE_EVENTS"
status=0
PATH="$fixture/local:$PATH" HOST_MAKE_FETCH_STATUS=73 make --no-print-directory -j4 \
    install-server "MAKE=$(command -v make)" > "$fixture/server-fetch-failure.log" 2>&1 || status=$?
[[ "$status" == 2 ]] || exit 1
printf '%s\n' 'fetch --locked' > "$fixture/expected"
cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
# Exercise the actual Make shell/environment handoff, including a space in the admitted path.
PATH="$fixture/local:$PATH" HOST_MAKE_ASSERT_POCKET_IC=1 make --no-print-directory \
    "MAKE=$(command -v make)" testkit-installed-check > "$fixture/server-environment.log" 2>&1
# Keep the actual Host LOC caller, argument selection and failure propagation
# qualified without replaying Shared's synthetic workspace/parser matrix.
cat > "$fixture/local/scripts/dev/cloc.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'cloc' >> "$HOST_MAKE_EVENTS"
printf ' <%s>' "$@" >> "$HOST_MAKE_EVENTS"
printf '\n' >> "$HOST_MAKE_EVENTS"
exit "${HOST_MAKE_CLOC_STATUS:-0}"
STUB
for selection in default manifest; do
    options=(--no-print-directory -j4 cloc)
    [[ "$selection" != manifest ]] || options+=(CLOC_MANIFEST='apps/selected app/Cargo.toml')
    for result in 0 23; do
        : > "$HOST_MAKE_EVENTS"
        status=0
        HOST_MAKE_CLOC_STATUS="$result" make "${options[@]}" \
            > "$fixture/cloc-$selection-$result.log" 2>&1 || status=$?
        if [[ "$result" == 0 ]]; then [[ "$status" == 0 ]] || exit 1;
        else [[ "$status" == 2 ]] || exit 1; fi
        if [[ "$selection" == default ]]; then
            printf 'cloc <%s>\n' "$PWD" > "$fixture/expected"
        else
            printf 'cloc <--manifest> <apps/selected app/Cargo.toml> <%s>\n' "$PWD" > "$fixture/expected"
        fi
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
    done
done
# Rust preflight must finish before either common installation stage may run.
: > "$HOST_MAKE_EVENTS"
status=0
HOST_MAKE_TOOL_FAIL=install-rust-tools.sh HOST_MAKE_TOOL_FAIL_MODE=preflight \
    make --no-print-directory -j4 install-tools > "$fixture/preflight-rust.log" 2>&1 || status=$?
[[ "$status" == 2 ]] || exit 1
printf '%s\n' 'install-rust-tools.sh preflight' > "$fixture/expected"
cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
[[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
# Real pipe-descriptor admission in substituted Cargo, without compiling.
for helper in scripts/ci/check-dependency-pins.sh \
    scripts/release/test-adapter.sh scripts/publish/test-workspace.sh \
    scripts/ci/test-host-fixture-retention.sh scripts/ci/test-product-evidence.sh scripts/ci/test-make-snapshot.sh \
    scripts/ci/check-release-commands.sh; do
    mkdir -p "$fixture/local/${helper%/*}"
    printf '#!/usr/bin/env bash\nexec cargo fixture "$@"\n' > "$fixture/local/$helper"
done
mkdir -p "$fixture/local/scripts/publish"
cat > "$fixture/local/scripts/publish/workspace.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == publish || "$1" == check ]] || exit 1
exec cargo "$1"
STUB
jobserver_options=(--no-print-directory -j4)
if make --help | grep -q -- --jobserver-style; then
    jobserver_options+=(--jobserver-style=pipe)
fi
cargo_targets=(check clippy clippy-workspace docs-check test test-artifacts-minimal check-fs-non-unix
    test-tools-default test-tools-interface
    test-wasm-inspection msrv msrv-workspace msrv-qualification)
helper_targets=(publish publish-check dependency-pins-check
    release-adapter-check tooling-command-check publish-command-check)
for target in "${cargo_targets[@]}" "${helper_targets[@]}"; do
    : > "$HOST_MAKE_EVENTS"
    PATH="$fixture/local:$PATH" HOST_MAKE_VERIFY_JOBSERVER=1 make "${jobserver_options[@]}" "MAKE=$(command -v make)" "$target" \
        > "$fixture/jobserver-$target.log" 2>&1
    [[ -s "$HOST_MAKE_EVENTS" ]] || exit 1
done
# Exercise CI's real sequential recipes under parallel Make. A lint refusal
# stops before formatting/tests; a passing lint reaches the next stage exactly once.
cat > "$fixture/local/scripts/ci/check-no-symlinks.pl" <<'STUB'
exec 'cargo', 'fixture-no-symlinks';
STUB
for lint_status in 23 0; do
    : > "$HOST_MAKE_EVENTS"
    status=0
    PATH="$fixture/local:$PATH" RUNNER_TEMP="$fixture" \
        HOST_MAKE_CLIPPY_STATUS="$lint_status" HOST_MAKE_FAIL=sort \
        make "${jobserver_options[@]}" "MAKE=$(command -v make)" ci \
        > "$fixture/ci-lint-$lint_status.log" 2>&1 || status=$?
    [[ "$status" == 2 ]] || exit 1
    grep -Fx 'clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' \
        "$HOST_MAKE_EVENTS" > "$fixture/clippy-events"
    [[ "$(wc -l < "$fixture/clippy-events")" -eq 1 ]] || exit 1
    if [[ "$lint_status" == 23 ]]; then
        [[ "$(tail -n 1 "$HOST_MAKE_EVENTS")" == clippy\ --workspace\ * ]] || exit 1
        if grep -E '^(sort |fmt )' "$HOST_MAKE_EVENTS"; then exit 1; fi
    else
        [[ "$(tail -n 1 "$HOST_MAKE_EVENTS")" == 'sort --workspace --grouped --check' ]] || exit 1
    fi
done
# Every package retains the same floor, including optional YAML.
for package in ic-host-text ic-host-tools; do
    : > "$HOST_MAKE_EVENTS"
    PATH="$fixture/local:$PATH" make --no-print-directory msrv "PACKAGE=$package" > "$fixture/msrv-$package.log" 2>&1
    grep -F -- "+1.91.0 check -p $package --all-targets --all-features" "$HOST_MAKE_EVENTS" >/dev/null
done
: > "$HOST_MAKE_EVENTS"
PATH="$fixture/local:$PATH" make --no-print-directory msrv-workspace > "$fixture/msrv-workspace.log" 2>&1
grep -F -- '+1.91.0 check --workspace --all-targets --all-features' "$HOST_MAKE_EVENTS" >/dev/null
# CI and release share the same native, Testkit Wasm and unsupported FS floor.
: > "$HOST_MAKE_EVENTS"
PATH="$fixture/local:$PATH" HOST_MAKE_ASSERT_MSRV=1 make --no-print-directory -j4 \
    msrv-qualification "MAKE=$(command -v make)" > "$fixture/msrv-qualification.log" 2>&1
cat > "$fixture/expected" <<'EVENTS'
+1.91.0 check --workspace --all-targets --all-features --locked --offline
+1.91.0 check -p ic-testkit -p ic-testkit-native --features ic-testkit/held-reply-fixtures,ic-testkit-native/stable-memory --locked --offline --target wasm32-unknown-unknown
+1.91.0 check -p ic-testkit-perf-probe --locked --offline --target wasm32-unknown-unknown
check -p ic-host-fs --lib --bins --test unsupported_lock --all-features --target wasm32-unknown-unknown --locked --offline
test -p ic-host-tools --no-default-features --features icp-project --all-targets --locked --offline icp_project::
test -p ic-host-tools --no-default-features --features icp-ledger --all-targets --locked --offline icp_ledger::
test -p ic-host-tools --no-default-features --features ledger-evidence --all-targets --locked --offline ledger_evidence::
test -p ic-host-text --no-default-features --features yaml --lib --locked --offline yaml::
EVENTS
cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
# Actual Host formatting stops after sorter failure and points at complete logs.
for target in fmt fmt-check; do
    : > "$HOST_MAKE_EVENTS"
    status=0
    HOST_MAKE_FAIL=sort RUNNER_TEMP="$fixture" make --no-print-directory "$target" \
        "FORMAT_CARGO=$PWD/cargo" > "$fixture/$target-failure.log" 2>&1 || status=$?
    [[ "$status" == 2 ]] || exit 1
    grep -F 'FAILED (exit 23)' "$fixture/$target-failure.log" >/dev/null
    detail="$(sed -n 's/^Details: //p' "$fixture/$target-failure.log")"
    [[ -f "$detail" ]] || exit 1
    printf 'formatter stdout evidence\nformatter stderr evidence\n' > "$fixture/expected-log"
    cmp "$fixture/expected-log" "$detail"
    [[ "$(wc -l < "$HOST_MAKE_EVENTS")" -eq 1 ]] || exit 1
done
# Qualify every actual Host target; Shared owns flag aliases and combinations.
make_probe_target_admission "$(command -v make)" "$PWD/Makefile" "$fixture/local" \
    "$HOST_MAKE_EVENTS" fmt fmt-check release-patch release-minor release-major release-resume \
    "${cargo_targets[@]}" "${helper_targets[@]}"
[[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
# Exercise the real documentation Make entrypoint against newly imported docs.
cp "$root/scripts/ci/check-product-doc-links.sh" "$root/scripts/ci/check-documentation-links.pl" scripts/ci/
git init -q
mkdir -p docs/build docs/testkit docs/design testing/browser crates/example target
printf 'target/\n' > .gitignore
printf '[ignored](missing)\n' > target/generated.md
for document in docs/build/guide.md docs/testkit/guide.md docs/design/tooling-consolidation.md \
    testing/browser/README.md crates/example/README.md; do
    printf '# Guide\n' > "$document"
done
make --no-print-directory check-doc-links > "$fixture/docs-pass.log" 2>&1
for document in docs/build/guide.md docs/testkit/guide.md docs/design/tooling-consolidation.md \
    testing/browser/README.md crates/example/README.md; do
    printf '[broken](missing-audit-target)\n' > "$document"
    if make --no-print-directory check-doc-links > "$fixture/docs-failure.log" 2>&1; then exit 1; fi
    grep -F "$document: missing local target missing-audit-target" "$fixture/docs-failure.log" >/dev/null
    printf '# Guide\n' > "$document"
done
# Qualify the optimizer's actual Make and workflow callers with no installations.
: > "$HOST_MAKE_EVENTS"
make --no-print-directory install-ic-tools ic-tools-check > "$fixture/optimizer-tools.log" 2>&1
printf 'install-ic-tools.sh install\ninstall-ic-tools.sh check\n' > "$fixture/expected"
cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
yq -o json '.' "$root/.github/workflows/build-optimizer.yml" | jq -er \
    '.jobs["wasm-optimizer"].steps[] | select(.name == "Qualify native artifact transforms") | .run' > "$fixture/optimizer-step"
cat > scripts/ci/qualify-wasm-opt.sh <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == "$HOST_MAKE_OPTIMIZER_PATH/wasm-opt" && "$2" == "$RUNNER_TEMP/wasm-qualification/optimizer" ]] || exit 94
printf 'qualify optimizer\n' >> "$HOST_MAKE_EVENTS"
STUB
: > "$HOST_MAKE_EVENTS"
HOST_MAKE_OPTIMIZER_PATH="$fixture/prepared optimizer" RUNNER_TEMP="$fixture" \
    bash -e -o pipefail "$fixture/optimizer-step"
printf 'install-ic-tools.sh check\nqualify optimizer\n' > "$fixture/expected"
cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
# Execute native CI's compiler setup: rustup's implicit-toolchain install reads
# rust-toolchain.toml and does not add the command-line target to that selection.
yq -o json '.' "$root/.github/workflows/ci.yml" | jq -er \
    '.jobs.native.steps[] | select(.name == "Prepare declared development compiler and Wasm target") | .run' > "$fixture/compiler-step"
cat > "$fixture/local/rustup" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$HOST_MAKE_EVENTS"
if [[ "$1" == toolchain ]]; then exit "${HOST_MAKE_COMPILER_STATUS:-0}"; fi
if [[ "$1" == target ]]; then exit "${HOST_MAKE_TARGET_STATUS:-0}"; fi
exit 98
STUB
chmod +x "$fixture/local/rustup"
for failed_step in none compiler target; do
    : > "$HOST_MAKE_EVENTS"
    compiler_status=0 target_status=0 expected_status=0
    if [[ "$failed_step" == compiler ]]; then compiler_status=37 expected_status=37; fi
    if [[ "$failed_step" == target ]]; then target_status=47 expected_status=47; fi
    status=0
    PATH="$fixture/local:$PATH" HOST_MAKE_COMPILER_STATUS="$compiler_status" \
        HOST_MAKE_TARGET_STATUS="$target_status" bash -e -o pipefail "$fixture/compiler-step" || status=$?
    [[ "$status" == "$expected_status" ]] || exit 1
    printf 'toolchain install --profile minimal --no-self-update\n' > "$fixture/expected"
    if [[ "$failed_step" != compiler ]]; then
        printf 'target add wasm32-unknown-unknown\n' >> "$fixture/expected"
    fi
    cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
done
echo 'Host Make snapshot routing and execution admission passed (substitute effects)'
fixture_complete=true
