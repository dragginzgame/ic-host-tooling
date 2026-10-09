#!/usr/bin/env bash
set -euo pipefail

# Qualify Host's checkout-local routing using its actual Makefile and snapshot.
# Only the formatter and release effects are replaced in the disposable copy.
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
    scripts/ci/check-format-tools.sh; do
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
    *) printf '%s\n' "$*" >> "$HOST_MAKE_EVENTS" ;;
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
for selection in ordinary environment command nested; do
    unset SHARED_TOOLING_ROOT
    command=(make --no-print-directory -j2)
    if [[ "$selection" == environment ]]; then
        export SHARED_TOOLING_ROOT="$fixture/external"
    elif [[ "$selection" == command || "$selection" == nested ]]; then
        command+=("SHARED_TOOLING_ROOT=$fixture/external")
    fi
    if [[ "$selection" == nested ]]; then
        # Keep Make's recursive command and automatic variable literal.
        # shellcheck disable=SC2016
        printf '%%:\n\t+$(MAKE) -C "%s" $@\n' "$PWD" > "$fixture/parent.mk"
        command+=(-f "$fixture/parent.mk")
    fi
    for target in help fmt-check release-resume; do
        : > "$HOST_MAKE_EVENTS"
        "${command[@]}" "$target" "FORMAT_CARGO=$PWD/cargo" \
            VERSION=0.9.5 RELEASE_REMOTE=review RELEASE_BRANCH=release-review \
            > "$fixture/$selection-$target.log" 2>&1
        case "$target" in
            help) : > "$fixture/expected" ;;
            fmt-check) printf 'sort --workspace --check\nfmt --all -- --check\n' > "$fixture/expected" ;;
            release-resume) printf 'resume 0.9.5 review release-review\n' > "$fixture/expected" ;;
        esac
        cmp "$fixture/expected" "$HOST_MAKE_EVENTS"
        [[ ! -e "$HOST_MAKE_EXTERNAL" ]]
    done
done
unset SHARED_TOOLING_ROOT
# Outer Make must reject unsafe modes before any substituted effect can run.
for mode in -i --ignore-errors -n -t -q; do
    for selection in direct inherited; do
        for target in fmt-check release-patch; do
            : > "$HOST_MAKE_EVENTS"
            args=(--no-print-directory "$target" "FORMAT_CARGO=$PWD/cargo" "SHARED_TOOLING_ROOT=$fixture/external")
            status=0
            if [[ "$selection" == direct ]]; then
                make "$mode" "${args[@]}" > "$fixture/$selection-$mode-$target.log" 2>&1 || status=$?
            else
                MAKEFLAGS="$mode" make "${args[@]}" > "$fixture/$selection-$mode-$target.log" 2>&1 || status=$?
            fi
            [[ "$status" == 2 && ! -s "$HOST_MAKE_EVENTS" && ! -e "$HOST_MAKE_EXTERNAL" ]]
        done
    done
done
echo 'Host Make snapshot routing and execution admission passed (substitute effects)'
