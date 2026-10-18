#!/usr/bin/env bash
set -euo pipefail

# IC Build consumer adapter, adapted from IC Metrics (MIT). Dependencies: Cargo, jq, yq, Git, tar, awk and Unix utilities.
# Copyright (c) 2026 ic-metrics contributors; MIT terms in LICENSE.
operation="${1:-}"
[[ $# -eq 1 ]] || exit 2
# Standard entrypoints select cache preparation only after runner recovery/source
# admission. Consume the internal selection before any helper or Cargo child.
release_cache_prepare="${IC_BUILD_RELEASE_CACHE_PREPARE:-0}"
unset IC_BUILD_RELEASE_CACHE_PREPARE
if [[ "${RELEASE_DELIVERY-direct}" != direct ]]; then
    echo 'ic-build supports RELEASE_DELIVERY=direct only' >&2
    exit 2
fi
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
reader="$root/scripts/ci/read-cargo-workspace-version.sh"
case "$operation" in
    version) bash "$reader" --stable Cargo.toml ;;
    preflight)
        if ! bash "$root/scripts/ci/check-release-source.sh" \
            --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md; then
            echo 'release preflight refused; this attempt has not started validation or version preparation' >&2
            exit 1
        fi
        current_version="$(bash "$reader" --stable Cargo.toml)"
        [[ "$current_version" == "${RELEASE_PREVIOUS:?}" ]] || exit 1
        for path in Cargo.toml Cargo.lock CHANGELOG.md; do
            [[ -f "$path" && ! -L "$path" ]] || exit 1
        done
        # Refuse conflicting pending notes before a gate or preparation intent.
        awk -v version="${RELEASE_VERSION:?}" -v previous="$RELEASE_PREVIOUS" -v date="${RELEASE_DATE:?}" \
            -f scripts/ci/finalize-release-changelog.awk CHANGELOG.md > /dev/null
        bash "$root/scripts/ci/cargo-release-preflight.sh" "$root" "$release_cache_prepare" \
            --pin "$root/ci/tool-versions.env" \
            --check tools-check --prepare install-tools --prepare tools-check
        ;;
    prepare)
        bash "$root/scripts/ci/check-release-source.sh" \
            --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
        current_version="$(bash "$reader" --stable Cargo.toml)"
        # Preserve recovery after a completed preparation with a lost response.
        if [[ "$current_version" == "${RELEASE_VERSION:?}" ]]; then
            bash "$root/scripts/release/metadata.sh" check
            exit
        fi
        [[ "$current_version" == "${RELEASE_PREVIOUS:?}" ]] || exit 1
        bash "$root/scripts/ci/cargo-release-metadata.sh" prepare ic-build
        ;;
    check|commit-check)
        if [[ "$operation" == commit-check ]]; then
            [[ -z "${RELEASE_COMMIT:-}" ]] || exit 1
        fi
        bash "$root/scripts/ci/cargo-release-metadata.sh" check ic-build --manifest-order
        bash "$root/scripts/ci/check-release-source.sh" \
            --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
        if [[ "$operation" == commit-check ]]; then
            git diff --quiet -- Cargo.toml Cargo.lock CHANGELOG.md
        fi
        ;;
    *) echo 'usage: metadata.sh version|preflight|prepare|check|commit-check' >&2; exit 2 ;;
esac
