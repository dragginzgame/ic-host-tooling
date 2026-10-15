#!/usr/bin/env bash
set -euo pipefail

# Host selects the inputs; the shared checker owns formatting and preservation
# assertions. All Git changes occur in its disposable consumer copies.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/host-formatting.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Host formatting input retained"' EXIT

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
    crates/ic-host-fs/src/durable/mod.rs Cargo.toml "$fixture/unsorted.toml" --rust-sources \
    make/consumer.mk make/tools.mk make/rust-format.mk make/release.mk make/execution.mk \
    ci/tool-versions.env scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh \
    scripts/ci/select-rust-formatting-sources.sh scripts/ci/fixture-lifecycle.sh
fixture_complete=true
