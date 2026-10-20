#!/usr/bin/env bash
set -euo pipefail

# Consumer-owned selections; Shared owns archive safety. No build caches.
[[ $# == 3 ]] || { echo 'usage: archive-product-evidence.sh REPOSITORY RUNNER-TEMP NEW-ARCHIVE' >&2; exit 2; }
repository="$1" temporary="$2" output="$3"
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
inputs=()
shopt -s nullglob
for path in "$temporary/wasm-qualification/optimizer" \
    "$temporary/wasm-qualification/dependencies-fetch.log"; do
    if [[ -e "$path" || -L "$path" ]]; then
        inputs+=("$temporary" "${path#"$temporary/"}")
    fi
done
if [[ -e "$repository/target/publish" || -L "$repository/target/publish" ]]; then
    inputs+=("$repository" target/publish)
fi
# Package qualification owns an isolated Cargo target inside each attempt.
# Retain source/lock snapshots, extracted packages and logs, excluding that cache.
shopt -s dotglob
for attempt in "$repository"/target/package-qualification-*; do
    if [[ -L "$attempt" || ! -d "$attempt" ]]; then
        inputs+=("$repository" "${attempt#"$repository/"}")
        continue
    fi
    for path in "$attempt"/*; do
        [[ "${path##*/}" != target ]] || continue
        inputs+=("$repository" "${path#"$repository/"}")
    done
done
if [[ ${#inputs[@]} == 0 ]]; then
    echo 'No product qualification evidence was produced.' >&2
    exit 0
fi
bash "$root/scripts/ci/archive-evidence.sh" "$output" "${inputs[@]}"
