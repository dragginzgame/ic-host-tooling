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

perl "$root/scripts/ci/prepare-cargo-formatting-fixture.pl" \
    "$root/Cargo.toml" workspace.dependencies ic-host-artifacts ic-host-fs > "$fixture/unsorted.toml"
bash "$root/scripts/ci/check-formatting-hooks.sh" "$root" \
    crates/ic-host-fs/src/durable/mod.rs Cargo.toml "$fixture/unsorted.toml" --rust-sources \
    make/consumer.mk make/tools.mk make/rust-format.mk make/release.mk make/execution.mk \
    ci/tool-versions.env scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh \
    scripts/ci/select-rust-formatting-sources.sh scripts/ci/fixture-lifecycle.sh
fixture_complete=true
