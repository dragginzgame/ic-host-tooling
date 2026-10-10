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
completed=false
finish() {
    local status=$?
    # Bash 3.2 may enter EXIT with status zero after a nounset error.
    [[ "$completed" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture";
    else echo "Host Make evidence retained: $fixture" >&2; fi
    exit "$status"
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
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
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
    if [[ "$argument" == --preflight ]]; then mode=preflight; fi
done
printf '%s %s\n' "$name" "$mode" >> "$HOST_MAKE_EVENTS"
if [[ "${HOST_MAKE_TOOL_FAIL:-}" == "$name" && "${HOST_MAKE_TOOL_FAIL_MODE:-}" == "$mode" ]]; then exit 23; fi
STUB
done
mkdir -p "$fixture/local/scripts/release"
cat > "$fixture/local/scripts/release/tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'install-release-tools.sh %s\n' "$1" >> "$HOST_MAKE_EVENTS"
[[ "${HOST_MAKE_TOOL_FAIL:-}" != install-release-tools.sh ]] || exit 23
STUB
for mode in install check; do
    target=install-tools
    [[ "$mode" != check ]] || target=tools-check
    for failed in none host ic rust release; do
        : > "$HOST_MAKE_EVENTS"
        status=0
        HOST_MAKE_TOOL_FAIL="install-$failed-tools.sh" HOST_MAKE_TOOL_FAIL_MODE="$mode" make --no-print-directory -j4 "$target" \
            "SHARED_TOOLING_ROOT=$fixture/external" > "$fixture/$target-$failed.log" 2>&1 || status=$?
        if [[ "$failed" == none ]]; then [[ "$status" == 0 ]] || exit 1;
        else [[ "$status" == 2 ]] || exit 1; fi
        : > "$fixture/expected"
        if [[ "$mode" == install ]]; then
            printf '%s\n' 'install-ic-tools.sh preflight' 'install-rust-tools.sh preflight' >> "$fixture/expected"
        fi
        for tool in host ic rust release; do
            printf 'install-%s-tools.sh %s\n' "$tool" "$mode" >> "$fixture/expected"
            [[ "$tool" != "$failed" ]] || break
        done
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
    done
done
# Admission must finish before any of the four installation stages may run.
for failed in ic rust; do
    : > "$HOST_MAKE_EVENTS"
    status=0
    HOST_MAKE_TOOL_FAIL="install-$failed-tools.sh" HOST_MAKE_TOOL_FAIL_MODE=preflight \
        make --no-print-directory -j4 install-tools > "$fixture/preflight-$failed.log" 2>&1 || status=$?
    [[ "$status" == 2 ]] || exit 1
    printf '%s\n' 'install-ic-tools.sh preflight' > "$fixture/expected"
    if [[ "$failed" == rust ]]; then printf '%s\n' 'install-rust-tools.sh preflight' >> "$fixture/expected"; fi
    cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
    [[ ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
done
# Real pipe-descriptor admission in substituted Cargo, without compiling.
for helper in scripts/ci/check-dependency-pins.sh scripts/release/test-tools.sh \
    scripts/release/test-adapter.sh scripts/publish/test-workspace.sh \
    scripts/ci/test-host-fixture-retention.sh scripts/ci/test-make-snapshot.sh \
    scripts/ci/test-tool-commands.sh scripts/ci/test-rust-tools.sh \
    scripts/ci/check-release-commands.sh scripts/ci/test-cloc.sh; do
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
for target in check clippy docs-check test test-artifacts-minimal test-tools-response msrv publish publish-check \
    dependency-pins-check release-tools-test release-adapter-check tooling-command-check publish-command-check; do
    PATH="$fixture/local:$PATH" HOST_MAKE_VERIFY_JOBSERVER=1 make "${jobserver_options[@]}" "$target" \
        > "$fixture/jobserver-$target.log" 2>&1
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
    [[ "$(wc -l < "$HOST_MAKE_EVENTS")" == 1 ]] || exit 1
done
# Outer Make must reject unsafe modes before any substituted effect can run.
for mode in -i --ignore-errors -n -t -q; do
    for selection in direct inherited cleared replaced erased; do
        for target in fmt fmt-check release-patch release-minor release-major release-resume \
            check clippy docs-check test test-artifacts-minimal test-tools-response msrv install-release-tools publish publish-check \
            dependency-pins-check release-tools-test release-adapter-check tooling-command-check publish-command-check; do
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
            [[ "$status" == 2 && ! -s "$HOST_MAKE_EVENTS" && ! -e "$HOST_MAKE_EXTERNAL" ]] || exit 1
        done
    done
done
echo 'Host Make snapshot routing and execution admission passed (substitute effects)'
completed=true
