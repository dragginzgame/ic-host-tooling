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
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture/local" make/consumer.mk
for input in Makefile ci/tool-versions.env ci/release-delivery.txt scripts/ci/check-format-tools.sh; do
    parent=.
    [[ "$input" != */* ]] || parent="${input%/*}"
    mkdir -p "$fixture/local/$parent"
    cp "$root/$input" "$fixture/local/$input"
done
export HOST_MAKE_EVENTS="$fixture/events" HOST_MAKE_EXTERNAL="$fixture/external-events"
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
    perl -e '
        ($ENV{MAKEFLAGS} // "") =~ /--jobserver-(?:auth|fds)=(\d+),(\d+)/ or die "missing pipe jobserver\n";
        open my $reader, "<&=$1" or die "closed jobserver read descriptor: $!\n";
        open my $writer, ">&=$2" or die "closed jobserver write descriptor: $!\n";
    '
fi
case "$*" in
    'sort --version') printf 'cargo-sort %s\n' "$HOST_MAKE_SORT_VERSION" ;;
    'fmt --version') echo rustfmt ;;
    *)
        printf '%s\n' "$*" >> "$HOST_MAKE_EVENTS"
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
            fmt-check) printf 'sort --workspace --check\nfmt --all -- --check\n' > "$fixture/expected" ;;
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
printf '%s %s\n' "$name" "$mode" >> "$HOST_MAKE_EVENTS"
if [[ "${HOST_MAKE_TOOL_FAIL:-}" == "$name" && "${HOST_MAKE_TOOL_FAIL_MODE:-}" == "$mode" ]]; then exit 23; fi
STUB
done
for mode in install check; do
    target=install-tools
    [[ "$mode" != check ]] || target=tools-check
    for failed in none host rust; do
        : > "$HOST_MAKE_EVENTS"
        status=0
        HOST_MAKE_TOOL_FAIL="install-$failed-tools.sh" HOST_MAKE_TOOL_FAIL_MODE="$mode" make --no-print-directory -j4 "$target" \
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
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
    done
done
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
    scripts/ci/test-cargo-publication-attempt.sh \
    scripts/ci/test-host-fixture-retention.sh scripts/ci/test-make-snapshot.sh \
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
cargo_targets=(check clippy docs-check test test-artifacts-minimal
    test-tools-default test-tools-interface
    test-wasm-inspection msrv msrv-workspace)
helper_targets=(publish publish-check dependency-pins-check
    release-adapter-check tooling-command-check publish-command-check)
for target in "${cargo_targets[@]}" "${helper_targets[@]}"; do
    : > "$HOST_MAKE_EVENTS"
    PATH="$fixture/local:$PATH" HOST_MAKE_VERIFY_JOBSERVER=1 make "${jobserver_options[@]}" "$target" \
        > "$fixture/jobserver-$target.log" 2>&1
    [[ -s "$HOST_MAKE_EVENTS" ]] || exit 1
done
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
echo 'Host Make snapshot routing and execution admission passed (substitute effects)'
fixture_complete=true
