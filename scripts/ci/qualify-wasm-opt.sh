#!/usr/bin/env bash
set -euo pipefail

# The caller admits the optimizer's archive/pin before selecting this executable.
[[ $# == 2 ]] || { echo 'usage: qualify-wasm-opt.sh ABSOLUTE-WASM-OPT NEW-ABSOLUTE-EVIDENCE-DIRECTORY' >&2; exit 2; }
optimizer="$1" evidence="$2"
for path in "$optimizer" "$evidence"; do
    case "$path" in /*) ;; *) exit 2 ;; esac
    case "$path" in *$'\r'*|*$'\n'*) exit 2 ;; esac
done
[[ -f "$optimizer" && -x "$optimizer" && ! -L "$optimizer" && ! -e "$evidence" && ! -L "$evidence" ]] || exit 2
optimizer_parent="$(CDPATH='' cd -P -- "${optimizer%/*}" && printf '%s/.' "$PWD")"
parent="$(CDPATH='' cd -P -- "${evidence%/*}" && printf '%s/.' "$PWD")"
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
case "$root" in *$'\r'*|*$'\n'*) exit 2 ;; esac
root="$(CDPATH='' cd -P -- "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
for path in "$optimizer_parent" "$parent" "$root"; do
    case "$path" in *$'\r'*|*$'\n'*) exit 2 ;; esac
done
optimizer="${optimizer_parent%/.}/${optimizer##*/}"
evidence="${parent%/.}/${evidence##*/}"
[[ "$(node --version)" == "v$(cat "$root/.node-version")" ]] || exit 1
mkdir "$evidence"
complete=false
trap 'status=$?; [[ "$complete" == true || "$status" != 0 ]] || status=1; printf "Wasm optimization evidence retained: %s\n" "$evidence" >&2 || :; exit "$status"' EXIT
{
    git -C "$root" rev-parse HEAD
    uname -sm
    node --version
    "$optimizer" --version
    for path in "$optimizer" "$root/scripts/ci/qualify-wasm-opt.sh" \
        "$root/crates/ic-build/tests/wasm_optimizer.rs" \
        "$root/crates/ic-build/tests/fixtures/wasm-optimizer/"*; do
        bash "$root/scripts/ci/verify-file-checksum.sh" --print sha256 "$path"
        printf '%s\n' "$path"
    done
} > "$evidence/source.txt"
cp "$root/crates/ic-build/tests/fixtures/wasm-optimizer/"* "$evidence/"
"$optimizer" "$evidence/input.wat" -O0 -o "$evidence/input.wasm" > "$evidence/input.log" 2>&1
export IC_BUILD_WASM_OPT="$optimizer" IC_BUILD_WASM_EVIDENCE="$evidence"
IC_BUILD_WASM_NODE="$(command -v node)"
export IC_BUILD_WASM_NODE
cd "$root"
status=0
cargo test --locked --offline -p ic-build --test wasm_optimizer -- --ignored --nocapture \
    > "$evidence/qualification.log" 2>&1 || status=$?
cat "$evidence/qualification.log" || :
[[ "$status" == 0 ]] || exit "$status"
cat "$evidence/execution.log"
echo 'Native Wasm optimization, retained publication, reuse and execution passed'
complete=true
