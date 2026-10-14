#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
cd "$root"
# Cargo's actual normal dependency graph, excluding test-only artifact fixtures.
response="$(cargo tree -p ic-host-tools --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$response" | rg '^(ic-host-(artifacts|fs|process|text)|candid_parser|ciborium|ic-certification) '; then
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
certificate="$(cargo tree -p ic-host-tools --no-default-features --features certificate --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$certificate" | rg '^(ic-host-(artifacts|fs|process|text)|candid_parser|ic-agent|reqwest) '; then
    echo 'certificate decoder graph includes execution, transport, text or other inspection owners' >&2
    exit 1
fi
for owner in ciborium ic-certification; do
    printf '%s\n' "$certificate" | rg "^$owner " >/dev/null || exit 1
done
text="$(cargo tree -p ic-host-text --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$text" | rg '^(ic-host-(artifacts|fs|process|tools)|candid_parser|ic-agent|ic-certification|ciborium) '; then
    echo 'text graph includes another Host or IC owner' >&2
    exit 1
fi
printf '%s\n' "$text" | rg '^unicode-width ' >/dev/null || exit 1

project="$(cargo tree -p ic-host-tools --no-default-features --features icp-project --edges normal --prefix none --format '{p}' --locked --offline)"
if printf '%s\n' "$project" | rg '^(ic-host-(artifacts|fs|process|text)|candid_parser|ic-certification) '; then
    echo 'ICP project projection includes an unrelated execution or inspection owner' >&2
    exit 1
fi
printf '%s\n' "$project" | rg '^serde_yaml_ng ' >/dev/null || {
    echo 'ICP project projection lacks the canonical YAML parser' >&2
    exit 1
}
if printf '%s\n' "$response" | rg '^serde_yaml_ng '; then
    echo 'default tools graph includes the optional project parser' >&2
    exit 1
fi

echo 'tools/text feature dependency graphs passed'
