#!/usr/bin/env bash
set -euo pipefail

# Select one Host-owned CLI through the shared installer and receipt checker.
# Installation is explicit; check and run never install or fall back to PATH.
mode="${1:-}"
[[ $# -ge 1 ]] || exit 2
shift
case "$mode" in
    install|check) [[ $# == 0 ]] || exit 2 ;;
    run) [[ $# -gt 0 ]] || exit 2 ;;
    *) echo 'usage: tools.sh install|check|run SET_VERSION_ARGUMENTS...' >&2; exit 2 ;;
esac
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=/dev/null
source "$root/ci/release-tools.env"
arguments=(--consumer "$PWD" --package cargo-edit --version "$HOST_CARGO_EDIT_VERSION"
    --bin cargo-set-version --profile release)
[[ "$mode" == install ]] || arguments+=(--check)
executable="$(bash "$root/scripts/dev/install-rust-tools.sh" "${arguments[@]}")" || exit "$?"
actual="$("$executable" set-version --version)" || exit "$?"
[[ "$actual" == "cargo-edit-set-version $HOST_CARGO_EDIT_VERSION" ]] || {
    echo 'release tool version differs from the selected cargo-edit pin' >&2; exit 1;
}
if [[ "$mode" == run ]]; then
    exec "$executable" set-version "$@"
fi
printf '%s\n' "$executable"
