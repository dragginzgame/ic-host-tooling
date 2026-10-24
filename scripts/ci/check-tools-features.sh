#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
cd "$root"

# Inspect all direct declarations (aliases, dev/build and target-specific edges)
# before inspecting the selected transitive graphs below.
bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$root" --external-ic

[[ $# -le 1 ]] || { echo 'usage: check-tools-features.sh [TARGET]' >&2; exit 2; }
target="${1:-${CARGO_BUILD_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}}"
case "$target" in
    x86_64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin) ;;
    *) printf 'unsupported Host graph target: %s\n' "$target" >&2; exit 2 ;;
esac
# Host owns the profile matrix; inspect transitive normal/build dependencies for
# the explicit target, excluding dev fixtures. This is not artifact attestation.
# Observe command failures separately from dependency violations.
perl scripts/ci/check-cargo-graph.pl --consumer "$root" --package ic-host-http \
    --target "$target" --require reqwest=1 --require ic-host-artifacts=1 \
    --forbid-pattern '^(ic-platform|candid|ic-agent|ic-host-(fs|process|tools)|rustls|native-tls|async-compression)$'
perl scripts/ci/check-cargo-graph.pl --consumer "$root" --package ic-host-http \
    --target wasm32-unknown-unknown --forbid-pattern '^(?!ic-host-http$)'
perl scripts/ci/check-cargo-graph.pl --consumer "$root" --package ic-build \
    --target "$target" --forbid clap
perl scripts/ci/check-cargo-graph.pl --consumer "$root" --package ic-build \
    --target "$target" --features cli --require clap=1
check_graph() {
    local label="$1" package="$2" defaults="$3" features="$4" required="$5" forbidden="$6" owner
    local arguments=(--consumer "$root" --package "$package" --target "${7:-$target}")
    arguments+=(--forbid ic-host-http)
    local owners=()
    [[ "$defaults" != none ]] || arguments+=(--no-default-features)
    [[ -z "$features" ]] || arguments+=(--features "$features")
    arguments+=(--forbid-pattern "${forbidden% }$" --forbid-pattern '^(ciborium|ciborium-io|ciborium-ll)$')
    # Platform's runtime is independently selectable; no Host profile owns it.
    arguments+=(--forbid-pattern '^(ic-cdk|ic-cdk-macros|ic-cdk-executor|ic-platform-macros|ic0)$')
    arguments+=(--forbid-pattern '^(serde_yaml_ng|serde_yml|serde-saphyr|granit-parser)$')
    case "$label" in
        icp-project|tools-all|text-yaml) ;;
        *) arguments+=(--forbid-pattern '^(serde_yaml|unsafe-libyaml)$') ;;
    esac
    if [[ "$label" != artifacts-text ]]; then
        arguments+=(--forbid-pattern '^(wat|wast)$')
    fi
    if [[ -n "$required" ]]; then
        read -r -a owners <<< "$required"
        for owner in "${owners[@]}"; do arguments+=(--require "$owner=1"); done
    fi
    perl "$root/scripts/ci/check-cargo-graph.pl" "${arguments[@]}" > /dev/null
}

check_graph tools ic-host-tools default '' '' \
    '^(ic-platform|candid|candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser|ic-certification|ic-agent|reqwest|url|serde_cbor|minicbor) '
check_graph tools-no-defaults ic-host-tools none '' '' \
    '^(ic-platform|candid|candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser|ic-certification|ic-agent|reqwest|url|serde_cbor|minicbor) '
check_graph encoding ic-host-tools none encoding '' \
    '^(ic-platform|candid|candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser|ic-certification|ic-agent|reqwest|serde_cbor|minicbor) '
check_graph icp-project ic-host-tools none icp-project 'ic-host-text serde_yaml unsafe-libyaml' \
    '^(ic-platform|candid|candid_parser|ic-host-(artifacts|fs|process)|wasmparser|ic-certification|ic-agent|reqwest|serde_cbor|minicbor) '
check_graph ledger-evidence ic-host-tools none ledger-evidence 'ic-platform candid icrc-ledger-types' \
    '^(candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser|ic-agent|reqwest|ic-stable-structures) '
check_graph icp-ledger ic-host-tools none icp-ledger prost \
    '^(ic-platform|candid|candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser|ic-certification|ic-agent|reqwest|serde_cbor|minicbor) '
check_graph registry ic-host-tools none registry 'ic-platform prost' \
    '^(candid|candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser|ic-certification|ic-agent|reqwest) '
check_graph ic-interface ic-host-tools none ic-interface \
    'ic-host-artifacts wasmparser' '^(ic-platform|candid|candid_parser|ic-host-(fs|process|text)|serde_cbor|minicbor|wasm-encoder) '
check_graph inspection ic-host-wasm-inspect default '' 'ic-host-artifacts ic-host-fs' \
    '^(ic-host-process|ic-platform|candid_parser|serde_cbor|minicbor|wasm-encoder|cap-std|cap-fs-ext|tempfile) '
check_graph certificate ic-host-tools none certificate 'ic-platform serde_cbor ic-certification' \
    '^(candid|candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser|ic-agent|reqwest|minicbor) '
check_graph tools-parser-free-combined ic-host-tools none ic-interface,certificate \
    'ic-platform serde_cbor ic-certification ic-host-artifacts wasmparser' \
    '^(candid|candid_parser|ic-host-(fs|process|text)|ic-agent|reqwest|minicbor) '
check_graph candid ic-host-tools none candid 'ic-platform candid candid_parser' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|ic-certification|ic-agent|reqwest|serde_cbor|minicbor) '
check_graph tools-combined ic-host-tools none candid,ic-interface,certificate \
    'ic-platform candid_parser serde_cbor ic-certification ic-host-artifacts wasmparser' \
    '^(ic-host-(fs|process|text)|ic-agent|reqwest|minicbor) '
check_graph tools-all ic-host-tools none agent,candid,certificate,encoding,ic-interface,icp-project,icp-ledger,ledger-evidence,registry \
    'ic-platform candid_parser serde_cbor ic-certification ic-host-artifacts wasmparser ic-agent reqwest icrc-ledger-types ic-host-text serde_yaml unsafe-libyaml prost' \
    '^(ic-host-(fs|process)|ic-stable-structures) '
check_graph text ic-host-text default '' unicode-width \
    '^(ic-host-(artifacts|fs|process|tools)|ic-platform|candid_parser|ic-agent|ic-certification|serde_cbor|minicbor|serde) '
check_graph text-yaml ic-host-text none yaml 'unicode-width serde serde_yaml unsafe-libyaml' \
    '^(ic-host-(artifacts|fs|process|tools)|ic-platform|candid|candid_parser|ic-agent|ic-certification|serde_cbor|minicbor) '
check_graph artifacts ic-host-artifacts default '' '' '^(ic-host-process|wasm-encoder|wasmparser|clap) '
check_graph artifacts-wasm ic-host-artifacts none wasm wasmparser '^wasm-encoder '
check_graph artifacts-encoding ic-host-artifacts none wasm-encoding wasm-encoder \
    '^(ic-host-(fs|process|text|tools)|ic-platform|candid|candid_parser|wasmparser) '
check_graph artifacts-text ic-host-artifacts none wasm-text 'wat wast wasm-encoder' \
    '^(ic-host-(fs|process|text|tools)|ic-platform|candid|candid_parser|wasmparser) '
for profile in wasm wasm-validation wasm-encoding wasm-text wasm-encoding,wasm-validation wasm-text,wasm-validation; do
    feature_graph="$(cargo tree -p ic-host-artifacts --no-default-features --features "$profile" \
        --target "$target" --edges normal,build --prefix none --format '{p} features={f}' --locked --offline)" || exit
    status=0
    printf '%s\n' "$feature_graph" | rg '^wasmparser [^ ]+ features=([^, ]+,)*validate(,|$)' > /dev/null || status=$?
    case "$profile:$status" in
        wasm:1|wasm-encoding:1|wasm-text:1|wasm-validation:0|wasm-encoding,wasm-validation:0|wasm-text,wasm-validation:0) ;;
        *) printf '%s has an unexpected Wasm validator feature selection (status %s)\n' "$profile" "$status" >&2; exit 1 ;;
    esac
    case "$profile" in
        *wasm-text*)
            status=0
            printf '%s\n' "$feature_graph" | rg '^(wat|wast|wasm-encoder) [^ ]+ features=([^, ]+,)*component-model(,|$)' > /dev/null || status=$?
            case "$status" in
                1) ;;
                *) printf '%s has an unexpected WAT component feature selection (status %s)\n' "$profile" "$status" >&2; exit 1 ;;
            esac
            ;;
    esac
done
check_graph filesystem ic-host-fs default '' ic-host-artifacts \
    '^(ic-host-(process|tools|text)|ic-platform|candid|candid_parser|ic-agent|ic-certification|reqwest|serde_cbor|minicbor|clap|cap-std|cap-fs-ext|tempfile) '
check_graph filesystem-capability ic-host-fs none capability 'ic-host-artifacts cap-std cap-fs-ext' \
    '^(tempfile|clap|ic-host-process|ic-platform|wasmparser|wasm-encoder) '
check_graph filesystem-temporary ic-host-fs none temporary-file 'ic-host-artifacts tempfile' \
    '^(cap-std|cap-fs-ext|clap|ic-host-process|ic-platform|wasmparser|wasm-encoder) '
check_graph filesystem-combined ic-host-fs none capability,temporary-file 'ic-host-artifacts cap-std cap-fs-ext tempfile' \
    '^(clap|ic-host-process|ic-platform|wasmparser|wasm-encoder) '
check_graph filesystem-cli ic-host-fs none cli 'clap ic-host-artifacts' \
    '^(ic-host-process|ic-platform|wasmparser|wasm-encoder|cap-std|cap-fs-ext|tempfile) '
check_graph process ic-host-process default '' 'ic-host-fs' '^(clap|ic-platform|wasmparser|wasm-encoder|cap-std|cap-fs-ext|tempfile) '
check_graph filesystem-non-unix ic-host-fs none capability,temporary-file ic-host-artifacts \
    '^(cap-std|cap-fs-ext|tempfile|rustix|ic-host-process|ic-platform) ' wasm32-unknown-unknown
check_graph agent ic-host-tools none agent 'ic-platform candid ic-agent reqwest url serde_cbor' \
    '^(candid_parser|ic-host-(artifacts|fs|process|text)|wasmparser) '
check_graph agent-wasm ic-host-tools none agent 'ic-platform' \
    '^(ic-host-(artifacts|fs|process|text)|wasmparser|candid|candid_parser|ic-agent|reqwest|url|serde_cbor|minicbor) ' wasm32-unknown-unknown
printf 'tools/text feature dependency graphs passed for %s (normal/build)\n' "$target"
