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
fixture="$(mktemp -d "${TMPDIR:-/tmp}/host-make-snapshot.XXXXXX")"
finish() {
    local status=$?
    if [[ "$status" == 0 ]]; then rm -rf "$fixture";
    else echo "Host Make evidence retained: $fixture" >&2; fi
}
trap finish EXIT
for input in Makefile ci/tool-versions.env make/tools.mk make/rust-format.mk \
    make/release.mk make/execution.mk scripts/ci/check-make-execution.sh \
    scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh; do
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
    for target in help fmt-check release-resume; do
        : > "$HOST_MAKE_EVENTS"
        "${command[@]}" "$target" "FORMAT_CARGO=$PWD/cargo" \
            VERSION=0.9.5 RELEASE_REMOTE=review RELEASE_BRANCH=release-review \
            > "$fixture/$selection-$target.log" 2>&1
        case "$target" in
            help) : > "$fixture/expected" ;;
            fmt-check) printf 'sort --workspace --check\nfmt --all -- --check\n' > "$fixture/expected" ;;
            release-resume) printf 'resume 0.9.5 %s release-review\n' "$remote" > "$fixture/expected" ;;
        esac
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
        if [[ "$target" == fmt-check ]]; then
            grep -Fx 'Checking formatting... ok' "$fixture/$selection-$target.log" >/dev/null
        fi
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]]
    done
done
unset SHARED_TOOLING_ROOT
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
done
printf '%s %s\n' "$name" "$mode" >> "$HOST_MAKE_EVENTS"
if [[ "${HOST_MAKE_TOOL_FAIL:-}" == "$name" ]]; then exit 23; fi
STUB
done
for mode in install check; do
    target=install-tools
    [[ "$mode" != check ]] || target=tools-check
    for failed in none host ic rust; do
        : > "$HOST_MAKE_EVENTS"
        status=0
        HOST_MAKE_TOOL_FAIL="install-$failed-tools.sh" make --no-print-directory -j4 "$target" \
            "SHARED_TOOLING_ROOT=$fixture/external" > "$fixture/$target-$failed.log" 2>&1 || status=$?
        if [[ "$failed" == none ]]; then [[ "$status" == 0 ]];
        else [[ "$status" == 2 ]]; fi
        : > "$fixture/expected"
        for tool in host ic rust; do
            printf 'install-%s-tools.sh %s\n' "$tool" "$mode" >> "$fixture/expected"
            [[ "$tool" != "$failed" ]] || break
        done
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]]
    done
done
# Actual Host formatting stops after sorter failure and points at complete logs.
for target in fmt fmt-check; do
    : > "$HOST_MAKE_EVENTS"
    status=0
    HOST_MAKE_FAIL=sort RUNNER_TEMP="$fixture" make --no-print-directory "$target" \
        "FORMAT_CARGO=$PWD/cargo" > "$fixture/$target-failure.log" 2>&1 || status=$?
    [[ "$status" == 2 ]]
    grep -F 'FAILED (exit 23)' "$fixture/$target-failure.log" >/dev/null
    detail="$(sed -n 's/^Details: //p' "$fixture/$target-failure.log")"
    [[ -f "$detail" ]]
    printf 'formatter stdout evidence\nformatter stderr evidence\n' > "$fixture/expected-log"
    cmp "$fixture/expected-log" "$detail"
    [[ "$(wc -l < "$HOST_MAKE_EVENTS")" == 1 ]]
done
# Outer Make must reject unsafe modes before any substituted effect can run.
for mode in -i --ignore-errors -n -t -q; do
    for selection in direct inherited cleared replaced erased; do
        for target in fmt fmt-check release-patch release-minor release-major release-resume; do
            : > "$HOST_MAKE_EVENTS"
            args=(--no-print-directory "$target" "FORMAT_CARGO=$PWD/cargo" "SHARED_TOOLING_ROOT=$fixture/external" "MAKE=$recursive_make")
            status=0
            if [[ "$selection" == cleared ]]; then
                make "$mode" "${args[@]}" MAKEFLAGS= > "$fixture/$selection-$mode-$target.log" 2>&1 || status=$?
            elif [[ "$selection" == replaced ]]; then
                make "$mode" "${args[@]}" MAKEFLAGS=-j2 > "$fixture/$selection-$mode-$target.log" 2>&1 || status=$?
            elif [[ "$selection" == erased ]]; then
                make "$mode" "${args[@]}" MAKEFLAGS= MFLAGS= > "$fixture/$selection-$mode-$target.log" 2>&1 || status=$?
            elif [[ "$selection" == direct ]]; then
                make "$mode" "${args[@]}" > "$fixture/$selection-$mode-$target.log" 2>&1 || status=$?
            else
                MAKEFLAGS="$mode" make "${args[@]}" > "$fixture/$selection-$mode-$target.log" 2>&1 || status=$?
            fi
            [[ "$status" == 2 && ! -s "$HOST_MAKE_EVENTS" && ! -e "$HOST_MAKE_EXTERNAL" ]]
        done
    done
done
echo 'Host Make snapshot routing and execution admission passed (substitute effects)'
