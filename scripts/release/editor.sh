#!/usr/bin/env bash
# Shared companions: ci/release-editor.env scripts/dev/install-rust-tools.sh
set -euo pipefail
[[ $# == 1 && ( "$1" == install || "$1" == check ) ]] || {
    echo 'usage: editor.sh install|check' >&2; exit 2;
}
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=ci/release-editor.env
source "$root/ci/release-editor.env"
arguments=(--consumer "$root" --package ic-build --version "$HOST_RELEASE_EDITOR_VERSION" \
    --bin ic-build-release-metadata --profile debug)
[[ "$1" != check ]] || arguments+=(--check)
exec bash "$root/scripts/dev/install-rust-tools.sh" "${arguments[@]}"
