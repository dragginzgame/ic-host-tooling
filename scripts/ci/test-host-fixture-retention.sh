#!/usr/bin/env bash
set -euo pipefail

# Inject failures into disposable copies of the real Host fixture entrypoints.
# Stop before setup, Cargo, Git or publication substitutes can run.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/host-fixture-retention.XXXXXX")"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(CDPATH="" cd -P -- "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Host fixture retention evidence retained"' EXIT
export TMPDIR="${fixture%/*}"
probe_root="$fixture/probe-source"
mkdir -p "$probe_root/scripts/ci"
cp "$root/scripts/ci/fixture-lifecycle.sh" "$probe_root/scripts/ci/"
for input in scripts/ci/test-make-snapshot.sh scripts/release/test-adapter.sh \
    scripts/release/test-tools.sh scripts/publish/test-workspace.sh \
    scripts/ci/test-host-fixture-retention.sh scripts/ci/check-host-formatting.sh; do
    name="${input##*/}"
    mkdir -p "$probe_root/${input%/*}"
    awk '
        { print }
        /^trap .* EXIT$/ {
            print "printf evidence > \"${fixture_parent:-$fixture}/failure-marker\""
            print "case \"$HOST_FIXTURE_FAILURE\" in"
            print "    nounset) unset HOST_FIXTURE_MISSING; printf '\''%s\\n'\'' \"$HOST_FIXTURE_MISSING\" ;;"
            print "    command) false ;;"
            print "    status) exit 17 ;;"
            print "    incomplete) exit 0 ;;"
            print "    completed) fixture_complete=true; exit 0 ;;"
            print "    failed-completion) fixture_complete=true; exit 17 ;;"
            print "esac"
            exit
        }
    ' "$root/$input" > "$probe_root/$input"
    for failure in nounset command status incomplete completed failed-completion; do
        attempt="$fixture/$name-$failure"
        mkdir "$attempt"
        status=0
        TMPDIR="$attempt" HOST_FIXTURE_FAILURE="$failure" bash "$probe_root/$input" \
            > "$attempt.log" 2>&1 || status=$?
        expected=1
        case "$failure" in
            status|failed-completion) expected=17 ;;
            completed) expected=0 ;;
        esac
        [[ "$status" == "$expected" ]] || exit 1
        retained=("$attempt/"*)
        if [[ "$expected" == 0 ]]; then
            [[ ! -e "${retained[0]}" ]] || exit 1
        else
            [[ ${#retained[@]} == 1 && -d "${retained[0]}" ]] || exit 1
            [[ "$(cat "${retained[0]}/failure-marker")" == evidence ]] || exit 1
        fi
    done
done
# Each real fixture must survive changing directories with a relative aliased
# temporary parent. These use substitutes for release/publication effects.
mkdir "$fixture/relative-parent"
ln -s "$fixture/relative-parent" "$fixture/relative-alias"
for input in scripts/ci/test-make-snapshot.sh scripts/release/test-adapter.sh \
    scripts/publish/test-workspace.sh; do
    (cd "$fixture"; TMPDIR=relative-alias bash "$root/$input") \
        > "$fixture/relative-${input##*/}.log" 2>&1
    remaining=("$fixture/relative-parent/"*)
    [[ ! -e "${remaining[0]}" ]] || exit 1
done
# Exercise the real release-tool fixture through an aliased temporary directory
# and padded line counts, as on macOS, before contradicting its child status.
assertion_root="$fixture/assertion-source"
for input in scripts/release/test-tools.sh scripts/release/tools.sh ci/release-tools.env \
    scripts/ci/fixture-lifecycle.sh; do
    mkdir -p "$assertion_root/${input%/*}"
    cp "$root/$input" "$assertion_root/$input"
done
mkdir "$fixture/bin" "$fixture/physical-attempt"
HOST_FIXTURE_REAL_WC="$(command -v wc)"
export HOST_FIXTURE_REAL_WC
cat > "$fixture/bin/wc" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == -l ]]; then
    count="$("$HOST_FIXTURE_REAL_WC" "$@")" || exit "$?"
    printf '%8d\n' "$count"
else
    exec "$HOST_FIXTURE_REAL_WC" "$@"
fi
STUB
chmod +x "$fixture/bin/wc"
attempt="$fixture/assertion-attempt"
ln -s "$fixture/physical-attempt" "$attempt"
(cd "$fixture"; PATH="$fixture/bin:$PATH" TMPDIR=assertion-attempt \
    bash "$assertion_root/scripts/release/test-tools.sh") \
    > "$fixture/portable-tools.log" 2>&1
# Unlike an early exit, a skipped assertion can claim completion and delete
# its evidence. The same portable inputs must still reject a wrong status.
sed 's/RELEASE_TOOL_TEST_RUN_STATUS=37 /RELEASE_TOOL_TEST_RUN_STATUS=36 /' \
    "$root/scripts/release/test-tools.sh" > "$assertion_root/scripts/release/test-tools.sh"
status=0
(cd "$fixture"; PATH="$fixture/bin:$PATH" TMPDIR=assertion-attempt \
    bash "$assertion_root/scripts/release/test-tools.sh") \
    > "$fixture/assertion.log" 2>&1 || status=$?
[[ "$status" == 1 ]] || exit 1
retained=("$attempt/"*)
[[ ${#retained[@]} == 1 && -f "${retained[0]}/arguments" ]] || exit 1
if grep -F 'Host release-tool selection' "$fixture/assertion.log"; then exit 1; fi
echo 'Host fixtures reject premature completion and wrong assertions, retaining failure evidence'
fixture_complete=true
