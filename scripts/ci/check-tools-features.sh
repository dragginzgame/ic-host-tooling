#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
cd "$root"

# Host owns the profile matrix; Cargo supplies the normal graph, excluding dev
# fixtures. Observe command failures separately from dependency violations.
check_graph() {
    local label="$1" package="$2" defaults="$3" features="$4" required="$5" forbidden="$6"
    local graph status=0 owner
    local arguments=(tree -p "$package" --edges normal --prefix none --format '{p}' --locked --offline)
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

check_graph response ic-host-tools default '' '' \
    '^(ic-host-(artifacts|fs|process|text)|candid_parser|ciborium|ic-certification) '
check_graph extraction ic-host-tools default candid-extraction \
    'ic-host-artifacts ic-host-fs ic-host-process' '^candid_parser '
for feature in ic-interface ic-limits; do
    check_graph "$feature" ic-host-tools none "$feature" ic-host-artifacts \
        '^(ic-host-(fs|process)|candid_parser) '
done
check_graph service ic-host-tools none candid-service 'ic-host-artifacts candid_parser' \
    '^ic-host-(fs|process) '
check_graph inspection ic-host-wasm-inspect default '' 'ic-host-artifacts ic-host-fs' \
    '^(ic-host-process|candid_parser) '
check_graph certificate ic-host-tools none certificate 'ciborium ic-certification' \
    '^(ic-host-(artifacts|fs|process|text)|candid_parser|ic-agent|reqwest) '
check_graph encoding ic-host-tools none encoding '' \
    '^(ic-host-(artifacts|fs|process|text)|candid_parser|ciborium|ic-certification|ic-agent|reqwest) '
check_graph text ic-host-text default '' unicode-width \
    '^(ic-host-(artifacts|fs|process|tools)|candid_parser|ic-agent|ic-certification|ciborium) '
echo 'tools/text feature dependency graphs passed'
