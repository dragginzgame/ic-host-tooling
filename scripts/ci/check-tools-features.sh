#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
cd "$root"
# Cargo's actual normal dependency graph, excluding test-only artifact fixtures.
response="$(cargo tree -p ic-host-tools --no-default-features --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$response" | rg '^ic-host-(artifacts|fs|process) '; then
    echo 'response-only tools graph includes an extraction owner' >&2
    exit 1
fi
extraction="$(cargo tree -p ic-host-tools --edges normal --prefix none --format '{p}' --locked --offline)"
limits="$(cargo tree -p ic-host-tools --no-default-features --features ic-limits --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$limits" | rg '^ic-host-(fs|process) '; then
    echo 'IC limits graph includes a filesystem or process owner' >&2
    exit 1
fi
printf '%s\n' "$limits" | rg '^ic-host-artifacts ' >/dev/null || {
    echo 'IC limits graph lacks the canonical Wasm facts owner' >&2
    exit 1
}
for owner in artifacts fs process; do
    printf '%s\n' "$extraction" | rg "^ic-host-$owner " >/dev/null || {
        echo "default Candid extraction graph lacks ic-host-$owner" >&2
        exit 1
    }
done
echo 'tools feature dependency graphs passed'
