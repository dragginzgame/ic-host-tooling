#!/usr/bin/env bash
set -euo pipefail

# Host selects the inputs; the shared checker owns formatting and preservation
# assertions. All Git changes occur in its disposable consumer copies.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/host-formatting.XXXXXX")"
completed=false
finish() {
    local status=$?
    [[ "$completed" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else echo "Host formatting input retained: $fixture" >&2; fi
    exit "$status"
}
trap finish EXIT

# Swap adjacent workspace dependencies without moving their preceding comment
# or changing version/feature selections. Fail if those inputs no longer match.
awk '
    $1 == "ic-host-artifacts" { saved=$0; next }
    saved != "" {
        if ($1 != "ic-host-fs") exit 1
        print; print saved; saved=""; swapped=1; next
    }
    { print }
    END { if (!swapped || saved != "") exit 1 }
' "$root/Cargo.toml" > "$fixture/unsorted.toml"
bash "$root/scripts/ci/check-formatting-hooks.sh" "$root" \
    crates/ic-host-fs/src/durable/mod.rs Cargo.toml "$fixture/unsorted.toml" \
    make/tools.mk make/rust-format.mk make/release.mk make/execution.mk \
    ci/tool-versions.env scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh
completed=true
