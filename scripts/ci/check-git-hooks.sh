#!/usr/bin/env bash
set -euo pipefail

# Consumer inputs only; Shared Tooling owns hook behavior and its fixture matrix.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-hook-input.XXXXXX")"
trap 'rm -rf -- "$fixture"' EXIT

# Perturb only ordering in the actual workspace dependency table.
awk '
    $0 == "[workspace.dependencies]" {
        print; getline first; getline second; print second; print first; next
    }
    { print }
' "$root/Cargo.toml" > "$fixture/Cargo.toml"

bash "$root/scripts/ci/check-formatting-hooks.sh" "$root" \
    crates/ic_testkit_perf_probe/src/lib.rs Cargo.toml "$fixture/Cargo.toml" \
    --rust-sources \
    make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk \
    make/execution.mk ci/tool-versions.env Cargo.lock \
    scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh
