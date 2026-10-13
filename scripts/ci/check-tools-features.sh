#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
cd "$root"
# Cargo's actual normal dependency graph, excluding test-only artifact fixtures.
response="$(cargo tree -p ic-host-tools --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$response" | rg '^(ic-host-(artifacts|fs|process)|candid_parser) '; then
    echo 'response/text tools graph includes an optional inspection/extraction owner' >&2
    exit 1
fi
extraction="$(cargo tree -p ic-host-tools --features candid-extraction --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$extraction" | rg '^candid_parser '; then
    echo 'extractor graph includes the optional service parser' >&2
    exit 1
fi
for feature in ic-interface ic-limits; do
    graph="$(cargo tree -p ic-host-tools --no-default-features --features "$feature" --edges normal --prefix none --format '{p}' --locked --offline)"
    if printf '%s\n' "$graph" | rg '^(ic-host-(fs|process)|candid_parser) '; then
        echo "$feature graph includes a filesystem, process or Candid parser owner" >&2
        exit 1
    fi
    printf '%s\n' "$graph" | rg '^ic-host-artifacts ' >/dev/null || {
        echo "$feature graph lacks the canonical Wasm facts owner" >&2
        exit 1
    }
done
service="$(cargo tree -p ic-host-tools --no-default-features --features candid-service --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$service" | rg '^ic-host-(fs|process) '; then
    echo 'Candid service graph includes a filesystem or process owner' >&2
    exit 1
fi
for owner in ic-host-artifacts candid_parser; do
    printf '%s\n' "$service" | rg "^$owner " >/dev/null || {
        echo "Candid service graph lacks $owner" >&2
        exit 1
    }
done
for owner in artifacts fs process; do
    printf '%s\n' "$extraction" | rg "^ic-host-$owner " >/dev/null || {
        echo "explicit Candid extraction graph lacks ic-host-$owner" >&2
        exit 1
    }
done
inspection="$(cargo tree -p ic-host-wasm-inspect --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$inspection" | rg '^(ic-host-process|candid_parser) '; then
    echo 'Wasm inspection CLI graph includes a process or Candid parser owner' >&2
    exit 1
fi
for owner in artifacts fs; do
    printf '%s\n' "$inspection" | rg "^ic-host-$owner " >/dev/null || {
        echo "Wasm inspection CLI graph lacks ic-host-$owner" >&2
        exit 1
    }
done
echo 'tools feature dependency graphs passed'
