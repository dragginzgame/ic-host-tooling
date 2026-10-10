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
    scripts/ci/test-host-fixture-retention.sh; do
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
        [[ "$status" == "$expected" ]]
        retained=("$attempt/"*)
        [[ ${#retained[@]} == 1 && -d "${retained[0]}" ]]
        [[ "$(cat "${retained[0]}/failure-marker")" == evidence ]]
    done
done
echo 'Host fixtures reject premature completion and retain failure evidence'
completed=true
