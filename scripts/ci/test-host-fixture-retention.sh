#!/usr/bin/env bash
set -euo pipefail

# Inject failures into disposable copies of the real Host fixture entrypoints.
# Stop before setup, Cargo, Git or publication substitutes can run.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
# shellcheck source=scripts/ci/fixture-retention-probes.sh
source "$root/scripts/ci/fixture-retention-probes.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/host-fixture-retention.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Host fixture retention evidence retained"' EXIT
export TMPDIR="${fixture%/*}"
# The self-probe sources this companion before installing its EXIT trap.
mkdir -p "$fixture/exit-source/scripts/ci"
cp "$root/scripts/ci/fixture-retention-probes.sh" "$fixture/exit-source/scripts/ci/"
for input in scripts/ci/test-make-snapshot.sh scripts/release/test-adapter.sh \
    scripts/publish/test-workspace.sh \
    scripts/ci/test-host-fixture-retention.sh scripts/ci/check-host-formatting.sh; do
    name="${input##*/}"
    directory_variable=fixture
    [[ "$input" != scripts/publish/test-workspace.sh ]] || directory_variable=FIXTURE_ROOT
    FIXTURE_PROBE_ROOT_VARIABLE=root FIXTURE_PROBE_DIRECTORY_VARIABLE="$directory_variable" \
        fixture_probe_exits "$root/$input" "$fixture" "$root" "$name" || exit 1
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
echo 'Host fixtures reject exit failures and premature completion, retaining evidence'
fixture_complete=true
