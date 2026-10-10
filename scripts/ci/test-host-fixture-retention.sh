#!/usr/bin/env bash
set -euo pipefail

# Inject failures into disposable copies of the real Host fixture entrypoints.
# Stop before setup, Cargo, Git or publication substitutes can run.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/host-fixture-retention.XXXXXX")"
completed=false
finish() {
    local status=$?
    [[ "$completed" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else echo "Host fixture retention evidence retained: $fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
for input in scripts/ci/test-make-snapshot.sh scripts/release/test-adapter.sh \
    scripts/release/test-tools.sh scripts/publish/test-workspace.sh \
    scripts/ci/test-host-fixture-retention.sh scripts/ci/check-host-formatting.sh; do
    name="${input##*/}"
    awk '
        { print }
        /^trap finish EXIT$/ {
            print "printf evidence > \"${fixture_parent:-$fixture}/failure-marker\""
            print "case \"$HOST_FIXTURE_FAILURE\" in"
            print "    nounset) unset HOST_FIXTURE_MISSING; printf '\''%s\\n'\'' \"$HOST_FIXTURE_MISSING\" ;;"
            print "    command) false ;;"
            print "    status) exit 17 ;;"
            print "    incomplete) exit 0 ;;"
            print "esac"
            exit
        }
    ' "$root/$input" > "$fixture/$name"
    for failure in nounset command status incomplete; do
        attempt="$fixture/$name-$failure"
        mkdir "$attempt"
        status=0
        TMPDIR="$attempt" HOST_FIXTURE_FAILURE="$failure" bash "$fixture/$name" \
            > "$attempt.log" 2>&1 || status=$?
        expected=1
        [[ "$failure" != status ]] || expected=17
        [[ "$status" == "$expected" ]] || exit 1
        retained=("$attempt/"*)
        [[ ${#retained[@]} == 1 && -d "${retained[0]}" ]] || exit 1
        [[ "$(cat "${retained[0]}/failure-marker")" == evidence ]] || exit 1
    done
done
# Contradict the real release-tool fixture's expected child status. Unlike an
# early exit, a skipped assertion can reach completion and delete its evidence.
assertion_root="$fixture/assertion-source"
for input in scripts/release/test-tools.sh scripts/release/tools.sh \
    scripts/dev/install-rust-tools.sh scripts/ci/verify-file-checksum.sh \
    ci/release-tools.env; do
    mkdir -p "$assertion_root/${input%/*}"
    cp "$root/$input" "$assertion_root/$input"
done
sed 's/RELEASE_TOOL_TEST_RUN_STATUS=37 /RELEASE_TOOL_TEST_RUN_STATUS=36 /' \
    "$root/scripts/release/test-tools.sh" > "$assertion_root/scripts/release/test-tools.sh"
attempt="$fixture/assertion-attempt"
mkdir "$attempt"
status=0
TMPDIR="$attempt" bash "$assertion_root/scripts/release/test-tools.sh" \
    > "$fixture/assertion.log" 2>&1 || status=$?
[[ "$status" == 1 ]] || exit 1
retained=("$attempt/"*)
[[ ${#retained[@]} == 1 && -f "${retained[0]}/arguments" ]] || exit 1
if grep -F 'Host release-tool selection' "$fixture/assertion.log"; then exit 1; fi
echo 'Host fixtures reject premature completion and wrong assertions, retaining failure evidence'
completed=true
