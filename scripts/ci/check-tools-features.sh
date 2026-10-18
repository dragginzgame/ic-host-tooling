#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
cd "$root"

# Inspect all direct declarations (aliases, dev/build and target-specific edges)
# before inspecting the selected transitive graphs below.
bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$root"

[[ $# -le 1 ]] || { echo 'usage: check-tools-features.sh [TARGET]' >&2; exit 2; }
target="${1:-${CARGO_BUILD_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}}"
case "$target" in
    x86_64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin) ;;
    *) printf 'unsupported Host graph target: %s\n' "$target" >&2; exit 2 ;;
esac
# Host owns the profile matrix; inspect transitive normal/build dependencies for
# the explicit target, excluding dev fixtures. This is not artifact attestation.
# Observe command failures separately from dependency violations.
check_graph() {
    local label="$1" package="$2" defaults="$3" features="$4" required="$5" forbidden="$6"
    local graph status=0 owner
    local arguments=(tree -p "$package" --target "${7:-$target}" --edges 'normal,build' --prefix none --format '{p}' --locked --offline)
    local owners=()
    [[ "$defaults" != none ]] || arguments+=(--no-default-features)
    [[ -z "$features" ]] || arguments+=(--features "$features")
    graph="$(cargo "${arguments[@]}")" || {
        status=$?
        printf '%s graph observation failed\n' "$label" >&2
        return "$status"
    }
    printf '%s\n' "$graph" | rg "$forbidden" || status=$?
    case "$status" in
        0) printf '%s graph includes a forbidden dependency\n' "$label" >&2; return 1 ;;
        1) ;;
        *) printf '%s graph exclusion check failed\n' "$label" >&2; return "$status" ;;
    esac
    if [[ -n "$required" ]]; then
        read -r -a owners <<< "$required"
        for owner in "${owners[@]}"; do
            status=0
            printf '%s\n' "$graph" | rg "^$owner " > /dev/null || status=$?
            case "$status" in
                0) ;;
                1) printf '%s graph lacks %s\n' "$label" "$owner" >&2; return 1 ;;
                *) printf '%s graph inclusion check failed\n' "$label" >&2; return "$status" ;;
            esac
        done
    fi
}

check_graph tools ic-host-tools default '' 'ic-platform candid_parser candid' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ciborium|ic-certification|ic-agent|reqwest|url) '
check_graph tools-no-defaults ic-host-tools none '' 'ic-platform candid_parser candid' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ciborium|ic-certification|ic-agent|reqwest|url) '
check_graph encoding ic-host-tools none encoding 'ic-platform candid_parser candid' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ciborium|ic-certification|ic-agent|reqwest|serde_yaml_ng) '
check_graph icp-project ic-host-tools none icp-project 'ic-platform candid_parser serde_yaml_ng' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ciborium|ic-certification|ic-agent|reqwest) '
check_graph ledger-evidence ic-host-tools none ledger-evidence 'ic-platform icrc-ledger-types candid_parser' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ciborium|ic-agent|reqwest|ic-stable-structures|serde_yaml_ng) '
check_graph ic-interface ic-host-tools none ic-interface \
    'ic-platform ic-host-artifacts candid_parser wasmparser' '^ic-host-(fs|process|text) '
check_graph inspection ic-host-wasm-inspect default '' 'ic-host-artifacts ic-host-fs' \
    '^(ic-host-process|ic-platform|candid_parser) '
check_graph certificate ic-host-tools none certificate 'ic-platform candid_parser ciborium ic-certification' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ic-agent|reqwest) '
check_graph tools-combined ic-host-tools none ic-interface,certificate \
    'ic-platform candid_parser ciborium ic-certification ic-host-artifacts wasmparser' \
    '^(ic-host-(fs|process|text)|ic-agent|reqwest) '
check_graph text ic-host-text default '' unicode-width \
    '^(ic-host-(artifacts|fs|process|tools)|ic-platform|candid_parser|ic-agent|ic-certification|ciborium) '
check_graph artifacts ic-host-artifacts default '' '' '^ic-host-process '
check_graph filesystem ic-host-fs default '' '' '^ic-host-process '
check_graph agent ic-host-tools none agent 'ic-platform candid_parser ic-agent reqwest url' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ciborium) '
check_graph agent-wasm ic-host-tools none agent 'ic-platform candid' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|candid_parser|ic-agent|reqwest|url) ' wasm32-unknown-unknown
printf 'tools/text feature dependency graphs passed for %s (normal/build)\n' "$target"
